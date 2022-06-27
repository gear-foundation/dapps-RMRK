use crate::token_tests::utils::*;
use codec::Encode;
use gstd::{ActorId, BTreeMap, BTreeSet};
use gtest::System;
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

    // check that token does not exist 
    check_rmrk_owner(&rmrk, token_id, None, ZERO_ID);
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
        parent_token_id,
        child_pending_token_id,
    )
    .main_failed());

    // mint child_token_id to parent_token_id
    assert!(!mint_to_nft(
        &rmrk_child,
        USERS[1],
        PARENT_NFT_CONTRACT,
        parent_token_id,
        child_accepted_token_id,
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
    let res = burn(&rmrk_child, USERS[0], child_pending_token_id);
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::Transfer {
            to: ZERO_ID.into(),
            token_id: child_pending_token_id.into(),
        }
        .encode()
    )));
    let res = burn(&rmrk_child, USERS[0], child_accepted_token_id);
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::Transfer {
            to: ZERO_ID.into(),
            token_id: child_accepted_token_id.into(),
        }
        .encode()
    )));

    // check that parent contract has no pending children
    check_pending_children(&rmrk_parent, parent_token_id, BTreeMap::new());

    // check that parent contract has no accepted children
    check_accepted_children(&rmrk_parent, parent_token_id, BTreeMap::new());
}

#[test]
fn burn_nested_token_failures() {
    let sys = System::new();
    before_test(&sys);
    let rmrk_child = sys.get_program(1);
    let child_token_id: u64 = 9;
    let parent_token_id: u64 = 10;

    // mint child_token_id to parent_token_id
    assert!(!mint_to_nft(
        &rmrk_child,
        USERS[1],
        PARENT_NFT_CONTRACT,
        child_token_id,
        parent_token_id,
    )
    .main_failed());

    // must fail since caller is not root owner of the nested child token
    assert!(burn(&rmrk_child, USERS[3], child_token_id).main_failed());
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
    let mut accepted_children: BTreeMap<ActorId, BTreeSet<TokenId>> = BTreeMap::new();
    accepted_children.insert(
        CHILD_NFT_CONTRACT.into(),
        BTreeSet::from([child_token_id.into()]),
    );
    check_accepted_children(&rmrk_parent, parent_token_id, accepted_children);

    // check accepted children of child_token_id
    let mut accepted_children: BTreeMap<ActorId, BTreeSet<TokenId>> = BTreeMap::new();
    accepted_children.insert(3.into(), BTreeSet::from([grand_token_id.into()]));
    check_accepted_children(&rmrk_child, child_token_id, accepted_children);

    // burn child
    assert!(!burn(&rmrk_child, USERS[0], child_token_id).main_failed());
    
    // check that parent_token_id has no accepted children
    check_accepted_children(&rmrk_parent, parent_token_id, BTreeMap::new());

    // check that child_token_id does not exist
    check_rmrk_owner(&rmrk_child, child_token_id, None, ZERO_ID);

    // check that grand_token_id does not exist
    check_rmrk_owner(&rmrk_grand, grand_token_id, None, ZERO_ID);

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
    assert!(!burn(&rmrk_parent, USERS[0], parent_token_id).main_failed());

    // check that child_token_id does not exist
    check_rmrk_owner(&rmrk_child, child_token_id, None, ZERO_ID);

    // check that grand_token_id does not exist
    check_rmrk_owner(&rmrk_grand, grand_token_id, None, ZERO_ID);

}
