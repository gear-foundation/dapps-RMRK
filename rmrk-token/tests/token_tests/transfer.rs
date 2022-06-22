use crate::token_tests::utils::*;
use codec::Encode;
use gtest::{Program, System};
use rmrk_io::*;

#[test]
fn transfer_simple() {
    let sys = System::new();
    before_test(&sys);
    let rmrk = sys.get_program(2);
    let token_id: u64 = 9;

    let res = transfer(&rmrk, USERS[0], USERS[3], token_id.into());
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::Transfer {
            to: USERS[3].into(),
            token_id: token_id.into(),
        }
        .encode()
    )));

    // check that RMRK owner
    let rmrk_owner = owner(&rmrk, token_id);
    assert_eq!(
        rmrk_owner,
        RMRKStateReply::Owner {
            token_id: None,
            owner_id: USERS[3].into(),
        }
    );
}

#[test]
fn transfer_parent_with_child() {
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

    assert!(!transfer(&rmrk_parent, USERS[0], USERS[3], parent_token_id.into()).main_failed());

    // check root_owner of child_token_id
    let res = get_root_owner(&rmrk_child, child_token_id.into());
    assert!(res.contains(&(10, RMRKEvent::RootOwner(USERS[3].into(),).encode())));

    // check root_owner of grand_token_id
    let res = get_root_owner(&rmrk_grand, grand_token_id.into());
    assert!(res.contains(&(10, RMRKEvent::RootOwner(USERS[3].into(),).encode())));
}
