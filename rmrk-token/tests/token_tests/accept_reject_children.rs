use crate::token_tests::utils::*;
use codec::Encode;
use gstd::{ActorId, BTreeMap, BTreeSet};
use gtest::System;
use rmrk_io::*;

#[test]
fn accept_child_simple() {
    let sys = System::new();
    before_test(&sys);
    let rmrk_child = sys.get_program(CHILD_NFT_CONTRACT);
    let rmrk_parent = sys.get_program(PARENT_NFT_CONTRACT);

    let child_token_id: u64 = 1;
    let parent_token_id: u64 = 10;

    // mint RMRK child token to RMRK parent token
    assert!(!mint_to_nft(
        &rmrk_child,
        USERS[3],
        PARENT_NFT_CONTRACT,
        parent_token_id,
        child_token_id,
    )
    .main_failed());

    let res = accept_child(
        &rmrk_parent,
        USERS[0],
        parent_token_id,
        CHILD_NFT_CONTRACT,
        child_token_id,
    );
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::AcceptedChild {
            child_token_address: CHILD_NFT_CONTRACT.into(),
            child_token_id: child_token_id.into(),
            parent_token_id: parent_token_id.into(),
        }
        .encode()
    )));

    // check that parent_token_id has no pending children
    check_pending_children(&rmrk_parent, parent_token_id, BTreeMap::new());

    // check accepted children
    let mut accepted_children: BTreeMap<ActorId, BTreeSet<TokenId>> = BTreeMap::new();
    accepted_children.insert(
        CHILD_NFT_CONTRACT.into(),
        BTreeSet::from([child_token_id.into()]),
    );
    check_accepted_children(&rmrk_parent, parent_token_id, accepted_children);
}

#[test]
fn accept_child_from_approved_address() {
    let sys = System::new();
    before_test(&sys);
    let rmrk_child = sys.get_program(CHILD_NFT_CONTRACT);
    let rmrk_parent = sys.get_program(PARENT_NFT_CONTRACT);

    let child_token_id: u64 = 1;
    let parent_token_id: u64 = 10;

    // mint RMRK child token to RMRK parent token
    assert!(!mint_to_nft(
        &rmrk_child,
        USERS[3],
        PARENT_NFT_CONTRACT,
        parent_token_id,
        child_token_id,
    )
    .main_failed());

    assert!(!approve(&rmrk_parent, USERS[0], USERS[3], parent_token_id.into()).main_failed());
    let res = accept_child(
        &rmrk_parent,
        USERS[0],
        parent_token_id,
        CHILD_NFT_CONTRACT,
        child_token_id,
    );
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::AcceptedChild {
            child_token_address: CHILD_NFT_CONTRACT.into(),
            child_token_id: child_token_id.into(),
            parent_token_id: parent_token_id.into(),
        }
        .encode()
    )));
}

#[test]
fn accept_child_failures() {
    let sys = System::new();
    before_test(&sys);
    let rmrk_child = sys.get_program(CHILD_NFT_CONTRACT);
    let rmrk_parent = sys.get_program(PARENT_NFT_CONTRACT);

    let child_token_id: u64 = 1;
    let parent_token_id: u64 = 10;

    // mint RMRK child token to RMRK parent token
    assert!(!mint_to_nft(
        &rmrk_child,
        USERS[3],
        PARENT_NFT_CONTRACT,
        parent_token_id,
        child_token_id,
    )
    .main_failed());

    // fail since the caller is not the owner
    assert!(accept_child(
        &rmrk_parent,
        USERS[3],
        parent_token_id,
        CHILD_NFT_CONTRACT,
        child_token_id
    )
    .main_failed());

    // fail since the child with that ID does not exist
    assert!(accept_child(
        &rmrk_parent,
        USERS[0],
        parent_token_id,
        CHILD_NFT_CONTRACT,
        2
    )
    .main_failed());

    // accept child
    assert!(!accept_child(
        &rmrk_parent,
        USERS[0],
        parent_token_id,
        CHILD_NFT_CONTRACT,
        child_token_id,
    )
    .main_failed());

    // fail since child has alredy been accepted
    assert!(accept_child(
        &rmrk_parent,
        USERS[0],
        parent_token_id,
        CHILD_NFT_CONTRACT,
        child_token_id,
    )
    .main_failed());
}

#[test]
fn reject_child_simple() {
    let sys = System::new();
    before_test(&sys);
    let rmrk_child = sys.get_program(1);
    let rmrk_parent = sys.get_program(2);

    let child_token_id: u64 = 1;
    let parent_token_id: u64 = 10;

    // mint RMRK child token to RMRK parent token
    assert!(!mint_to_nft(
        &rmrk_child,
        USERS[3],
        PARENT_NFT_CONTRACT,
        parent_token_id,
        child_token_id,
    )
    .main_failed());

    // reject child
    let res = reject_child(
        &rmrk_parent,
        USERS[0],
        parent_token_id,
        CHILD_NFT_CONTRACT,
        child_token_id,
    );
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::RejectedChild {
            child_token_address: CHILD_NFT_CONTRACT.into(),
            child_token_id: child_token_id.into(),
            parent_token_id: parent_token_id.into(),
        }
        .encode()
    )));

    // check that parent_token_id has no pending children
    check_pending_children(&rmrk_parent, parent_token_id, BTreeMap::new());

    // check that child token in rmrk_child does not exist
    check_rmrk_owner(&rmrk_child, child_token_id, None, ZERO_ID);

}

#[test]
fn reject_child_from_approved_address() {
    let sys = System::new();
    before_test(&sys);
    let rmrk_child = sys.get_program(1);
    let rmrk_parent = sys.get_program(2);

    let child_token_id: u64 = 1;
    let parent_token_id: u64 = 10;

    // mint RMRK child token to RMRK parent token
    assert!(!mint_to_nft(
        &rmrk_child,
        USERS[3],
        PARENT_NFT_CONTRACT,
        parent_token_id,
        child_token_id,
    )
    .main_failed());

    // approve to USERS[3]
    assert!(!approve(&rmrk_parent, USERS[0], USERS[3], parent_token_id.into()).main_failed());
    // reject child from USERSS[3]
    let res = reject_child(
        &rmrk_parent,
        USERS[0],
        parent_token_id,
        CHILD_NFT_CONTRACT,
        child_token_id,
    );
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::RejectedChild {
            child_token_address: CHILD_NFT_CONTRACT.into(),
            child_token_id: child_token_id.into(),
            parent_token_id: parent_token_id.into(),
        }
        .encode()
    )));
}

#[test]
fn reject_child_failures() {
    let sys = System::new();
    before_test(&sys);
    let rmrk_child = sys.get_program(1);
    let rmrk_parent = sys.get_program(2);

    let child_token_id: u64 = 1;
    let parent_token_id: u64 = 10;

    // mint RMRK child token to RMRK parent token
    assert!(!mint_to_nft(
        &rmrk_child,
        USERS[3],
        PARENT_NFT_CONTRACT,
        child_token_id,
        parent_token_id,
    )
    .main_failed());

    // must fail since the caller is not owner or not approved account
    assert!(reject_child(
        &rmrk_parent,
        USERS[3],
        parent_token_id,
        CHILD_NFT_CONTRACT,
        child_token_id
    )
    .main_failed());

    // must fail since the child with indicated id does not exist
    assert!(reject_child(
        &rmrk_parent,
        USERS[0],
        parent_token_id,
        CHILD_NFT_CONTRACT,
        100
    )
    .main_failed());
}

#[test]
fn remove_child_simple() {
    let sys = System::new();
    before_test(&sys);
    let rmrk_child = sys.get_program(CHILD_NFT_CONTRACT);
    let rmrk_parent = sys.get_program(PARENT_NFT_CONTRACT);

    let child_token_id: u64 = 1;
    let parent_token_id: u64 = 10;

    // mint RMRK child token to RMRK parent token
    assert!(!mint_to_nft(
        &rmrk_child,
        USERS[3],
        PARENT_NFT_CONTRACT,
        parent_token_id,
        child_token_id,
    )
    .main_failed());

    // accept child
    assert!(!accept_child(
        &rmrk_parent,
        USERS[0],
        parent_token_id,
        CHILD_NFT_CONTRACT,
        child_token_id,
    )
    .main_failed());

    // remove child
    let res = remove_child(
        &rmrk_parent,
        USERS[0],
        parent_token_id,
        CHILD_NFT_CONTRACT,
        child_token_id,
    );

    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::RemovedChild {
            child_token_address: CHILD_NFT_CONTRACT.into(),
            child_token_id: child_token_id.into(),
            parent_token_id: parent_token_id.into(),
        }
        .encode()
    )));

    // check that parent_token_id has no accepted children
    check_accepted_children(&rmrk_parent, parent_token_id, BTreeMap::new());

    // check that child token in rmrk_child does not exist
    check_rmrk_owner(&rmrk_child, child_token_id, None, ZERO_ID);

}

#[test]
fn remove_child_from_approved_account() {
    let sys = System::new();
    before_test(&sys);
    let rmrk_child = sys.get_program(CHILD_NFT_CONTRACT);
    let rmrk_parent = sys.get_program(PARENT_NFT_CONTRACT);

    let child_token_id: u64 = 1;
    let parent_token_id: u64 = 10;

    // mint RMRK child token to RMRK parent token
    assert!(!mint_to_nft(
        &rmrk_child,
        USERS[3],
        PARENT_NFT_CONTRACT,
        parent_token_id,
        child_token_id,
    )
    .main_failed());

    // accept child
    assert!(!accept_child(
        &rmrk_parent,
        USERS[0],
        parent_token_id,
        CHILD_NFT_CONTRACT,
        child_token_id,
    )
    .main_failed());

    assert!(!approve(&rmrk_parent, USERS[0], USERS[3], parent_token_id.into()).main_failed());

    let res = remove_child(
        &rmrk_parent,
        USERS[3],
        parent_token_id,
        CHILD_NFT_CONTRACT,
        child_token_id,
    );

    assert!(res.contains(&(
        USERS[3],
        RMRKEvent::RemovedChild {
            child_token_address: CHILD_NFT_CONTRACT.into(),
            child_token_id: child_token_id.into(),
            parent_token_id: parent_token_id.into(),
        }
        .encode()
    )));
    // check that parent_token_id has no accepted children
    check_accepted_children(&rmrk_parent, parent_token_id, BTreeMap::new());
}

#[test]
fn remove_child_failures() {
    let sys = System::new();
    before_test(&sys);
    let rmrk_child = sys.get_program(1);
    let rmrk_parent = sys.get_program(2);

    let child_token_id: u64 = 1;
    let parent_token_id: u64 = 10;

    // mint RMRK child token to RMRK parent token
    assert!(!mint_to_nft(
        &rmrk_child,
        USERS[3],
        PARENT_NFT_CONTRACT,
        parent_token_id,
        child_token_id
    )
    .main_failed());

    // accept child
    assert!(!accept_child(
        &rmrk_parent,
        USERS[0],
        parent_token_id,
        CHILD_NFT_CONTRACT,
        child_token_id,
    )
    .main_failed());
    // must fail since the caller is not owner or not approved account
    assert!(remove_child(
        &rmrk_parent,
        USERS[3],
        parent_token_id,
        CHILD_NFT_CONTRACT,
        child_token_id
    )
    .main_failed());

    // must fail since the child with indicated id does not exist
    assert!(remove_child(
        &rmrk_parent,
        USERS[0],
        parent_token_id,
        CHILD_NFT_CONTRACT,
        100
    )
    .main_failed());
}
