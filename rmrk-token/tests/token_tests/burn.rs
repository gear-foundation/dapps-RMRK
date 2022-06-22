use crate::token_tests::utils::*;
use codec::Encode;
use gstd::{ActorId, BTreeMap, BTreeSet};
use gtest::{Program, System};
use rmrk_io::*;

#[test]
fn burn_simple() {
    let sys = System::new();
    before_test(&sys);
    let rmrk = sys.get_program(PARENT_NFT_CONTRACT);
    let token_id: u64 = 5;
    let res = burn(&rmrk, USERS[0], token_id);
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::Transfer {
            to: ZERO_ID.into(),
            token_id: token_id.into(),
        }
        .encode()
    )));

    // check that token does not exist (must fail)
    let owner = owner(&rmrk, token_id.into());
    assert_eq!(
        owner,
        RMRKStateReply::Owner {
            token_id: None,
            owner_id: ZERO_ID.into()
        }
    );
}

#[test]
fn burn_simple_failures() {
    let sys = System::new();
    before_test(&sys);
    let rmrk = sys.get_program(2);
    // must fail since caller is not owner and not approved
    assert!(burn(&rmrk, USERS[3], 5).main_failed());
}

#[test]
fn burn_nested_token() {
    let sys = System::new();
    before_test(&sys);
    let rmrk_child = sys.get_program(1);
    let rmrk_parent = sys.get_program(2);
    let child_accepted_token_id: u64 = 8;
    let child_pending_token_id: u64 = 9;
    let parent_token_id: u64 = 10;

    // mint child_token_id to parent_token_id
    assert!(!mint_to_nft(
        &rmrk_child,
        USERS[1],
        PARENT_NFT_CONTRACT,
        parent_token_id.into(),
        child_pending_token_id.into(),
    )
    .main_failed());

    // mint child_token_id to parent_token_id
    assert!(!mint_to_nft(
        &rmrk_child,
        USERS[1],
        PARENT_NFT_CONTRACT,
        parent_token_id.into(),
        child_accepted_token_id.into(),
    )
    .main_failed());

    // accept one child
    assert!(!accept_child(
        &rmrk_parent,
        USERS[0],
        parent_token_id,
        CHILD_NFT_CONTRACT,
        child_accepted_token_id,
    )
    .main_failed());
    let res = burn(&rmrk_child, USERS[0], child_pending_token_id.into());
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::Transfer {
            to: ZERO_ID.into(),
            token_id: child_pending_token_id.into(),
        }
        .encode()
    )));
    let res = burn(&rmrk_child, USERS[0], child_accepted_token_id.into());
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::Transfer {
            to: ZERO_ID.into(),
            token_id: child_accepted_token_id.into(),
        }
        .encode()
    )));

    // check that parent contract has no pending children
    let pending_children_reply = get_pending_children(&rmrk_parent, parent_token_id.into());
    assert_eq!(
        pending_children_reply,
        RMRKStateReply::PendingChildren(BTreeMap::new())
    );

    // check that parent contract has no accepted children
    let accepted_children_reply = get_accepted_children(&rmrk_parent, parent_token_id.into());
    assert_eq!(
        accepted_children_reply,
        RMRKStateReply::AcceptedChildren(BTreeMap::new())
    );
}

#[test]
fn burn_nested_token_failures() {
    let sys = System::new();
    before_test(&sys);
    let rmrk_child = sys.get_program(1);
    let rmrk_parent = sys.get_program(2);
    let child_token_id: u64 = 9;
    let parent_token_id: u64 = 10;

    // mint child_token_id to parent_token_id
    assert!(!mint_to_nft(
        &rmrk_child,
        USERS[1],
        PARENT_NFT_CONTRACT,
        child_token_id.into(),
        parent_token_id.into(),
    )
    .main_failed());

    // must fail since caller is not root owner of the nested child token
    assert!(burn(&rmrk_child, USERS[3], child_token_id.into()).main_failed());
}

// ownership chain is now USERS[0] > parent_token_id > child_token_id > grand_token_id
// in that test child_token_id is burning
// rmrk_child contract must also burn grand_token_id and must be removed from parent_token_id
#[test]
fn recursive_burn_nested_token() {
    let sys = System::new();
    before_test(&sys);
    init_rmrk(&sys);
    let rmrk_child = sys.get_program(1);
    let rmrk_parent = sys.get_program(2);
    let rmrk_grand = sys.get_program(3);
    let child_token_id: u64 = 9;
    let parent_token_id: u64 = 10;
    let grand_token_id: u64 = 11;

    // ownership chain is  USERS[0] > parent_token_id > child_token_id > grand_token_id
    rmrk_chain(
        &rmrk_grand,
        &rmrk_child,
        &rmrk_parent,
        grand_token_id,
        child_token_id,
        parent_token_id,
    );

    // check accepted children of parent_token_id
    let accepted_children_reply = get_accepted_children(&rmrk_parent, parent_token_id.into());
    let mut accepted_children: BTreeMap<ActorId, BTreeSet<TokenId>> = BTreeMap::new();
    accepted_children.insert(
        CHILD_NFT_CONTRACT.into(),
        BTreeSet::from([child_token_id.into()]),
    );
    assert_eq!(
        accepted_children_reply,
        RMRKStateReply::AcceptedChildren(accepted_children)
    );

    // check accepted children of child_token_id
    let accepted_children_reply = get_accepted_children(&rmrk_child, child_token_id.into());
    let res = get_accepted_children(&rmrk_child, child_token_id.into());
    let mut accepted_children: BTreeMap<ActorId, BTreeSet<TokenId>> = BTreeMap::new();
    accepted_children.insert(3.into(), BTreeSet::from([grand_token_id.into()]));
    assert_eq!(
        accepted_children_reply,
        RMRKStateReply::AcceptedChildren(accepted_children)
    );

    // burn child
    assert!(!burn(&rmrk_child, USERS[0], child_token_id.into()).main_failed());
    // check that parent_token_id has no accepted children
    let accepted_children_reply = get_accepted_children(&rmrk_parent, parent_token_id.into());
    assert_eq!(
        accepted_children_reply,
        RMRKStateReply::AcceptedChildren(BTreeMap::new())
    );

    // check that child_token_id does not exist
    let rmrk_owner = owner(&rmrk_child, child_token_id);
    assert_eq!(
        rmrk_owner,
        RMRKStateReply::Owner {
            token_id: None,
            owner_id: ZERO_ID.into()
        }
    );
    // check that grand_token_id does not exist
    let rmrk_owner = owner(&rmrk_grand, grand_token_id);
    assert_eq!(
        rmrk_owner,
        RMRKStateReply::Owner {
            token_id: None,
            owner_id: ZERO_ID.into()
        }
    );
}

// ownership chain is now USERS[0] > parent_token_id > child_token_id > grand_token_id
// in that test parent_token_id is burning
// rmrk_child contract must also burn child_token_id and grand_token_id
#[test]
fn recursive_burn_parent_token() {
    let sys = System::new();
    before_test(&sys);
    init_rmrk(&sys);
    let rmrk_child = sys.get_program(1);
    let rmrk_parent = sys.get_program(2);
    let rmrk_grand = sys.get_program(3);
    let child_token_id: u64 = 9;
    let parent_token_id: u64 = 10;
    let grand_token_id: u64 = 11;
    let grand_grand_token_id: u64 = 12;

    // ownership chain is  USERS[0] > parent_token_id > child_token_id > grand_token_id
    rmrk_chain(
        &rmrk_grand,
        &rmrk_child,
        &rmrk_parent,
        grand_token_id,
        child_token_id,
        parent_token_id,
    );

    // burn parent_token_id
    assert!(!burn(&rmrk_parent, USERS[0], parent_token_id.into()).main_failed());

    // check that child_token_id does not exist
    let rmrk_owner = owner(&rmrk_child, child_token_id);
    assert_eq!(
        rmrk_owner,
        RMRKStateReply::Owner {
            token_id: None,
            owner_id: ZERO_ID.into()
        }
    );
    // check that grand_token_id does not exist
    let rmrk_owner = owner(&rmrk_grand, grand_token_id);
    assert_eq!(
        rmrk_owner,
        RMRKStateReply::Owner {
            token_id: None,
            owner_id: ZERO_ID.into()
        }
    );
}
