use crate::token_tests::utils::*;
use codec::Encode;
use gstd::{ActorId, BTreeMap, BTreeSet};
use gtest::System;
use rmrk_io::*;

// Root owner transfers accepted child token to between his RMRK tokens inside one contract
#[test]
fn transfer_accepted_child_to_token_with_same_owner() {
    let sys = System::new();
    before_test(&sys);
    let rmrk_child = sys.get_program(1);
    let rmrk_parent = sys.get_program(2);

    let child_token_id: u64 = 1;
    let parent_token_id: u64 = 10;
    let new_parent_token_id: u64 = 8;

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

    // USERS[0] transfer child to another his token
    let res = transfer_to_nft(
        &rmrk_child,
        USERS[0],
        PARENT_NFT_CONTRACT,
        child_token_id,
        new_parent_token_id,
    );
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::TransferToNft {
            to: PARENT_NFT_CONTRACT.into(),
            token_id: child_token_id.into(),
            destination_id: new_parent_token_id.into(),
        }
        .encode()
    )));

    // check owner
    let rmrk_owner = owner(&rmrk_child, child_token_id);
    assert_eq!(
        rmrk_owner,
        RMRKStateReply::Owner {
            token_id: Some(new_parent_token_id.into()),
            owner_id: PARENT_NFT_CONTRACT.into(),
        }
    );

    // check accepted children of parent_token_id
    let accepted_children_reply = get_accepted_children(&rmrk_parent, parent_token_id);
    assert_eq!(
        accepted_children_reply,
        RMRKStateReply::AcceptedChildren(BTreeMap::new())
    );

    // check accepted children of new_parent_token_id
    let accepted_children_reply = get_accepted_children(&rmrk_parent, new_parent_token_id);
    let mut accepted_children: BTreeMap<ActorId, BTreeSet<TokenId>> = BTreeMap::new();
    accepted_children.insert(
        CHILD_NFT_CONTRACT.into(),
        BTreeSet::from([child_token_id.into()]),
    );
    assert_eq!(
        accepted_children_reply,
        RMRKStateReply::AcceptedChildren(accepted_children)
    );
}

// Root owner transfers pending child token to between his RMRK tokens inside one contract
#[test]
fn transfer_pending_child_to_token_with_same_owner() {
    let sys = System::new();
    before_test(&sys);
    let rmrk_child = sys.get_program(1);
    let rmrk_parent = sys.get_program(2);

    let child_token_id: u64 = 1;
    let parent_token_id: u64 = 10;
    let new_parent_token_id: u64 = 8;

    // mint RMRK child token to RMRK parent token
    assert!(!mint_to_nft(
        &rmrk_child,
        USERS[3],
        PARENT_NFT_CONTRACT,
        parent_token_id,
        child_token_id,
    )
    .main_failed());

    // USERS[0] transfer child to another his token
    let res = transfer_to_nft(
        &rmrk_child,
        USERS[0],
        PARENT_NFT_CONTRACT,
        child_token_id,
        new_parent_token_id,
    );
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::TransferToNft {
            to: PARENT_NFT_CONTRACT.into(),
            token_id: child_token_id.into(),
            destination_id: new_parent_token_id.into(),
        }
        .encode()
    )));

    // check owner
    let rmrk_owner = owner(&rmrk_child, child_token_id);
    assert_eq!(
        rmrk_owner,
        RMRKStateReply::Owner {
            token_id: Some(new_parent_token_id.into()),
            owner_id: PARENT_NFT_CONTRACT.into(),
        }
    );

    // check pending children of parent_token_id
    let pending_children_reply = get_pending_children(&rmrk_parent, parent_token_id);
    assert_eq!(
        pending_children_reply,
        RMRKStateReply::PendingChildren(BTreeMap::new())
    );
    // check pending children of new_parent_token_id
    let pending_children_reply = get_pending_children(&rmrk_parent, new_parent_token_id);
    let mut pending_children: BTreeMap<ActorId, BTreeSet<TokenId>> = BTreeMap::new();
    pending_children.insert(
        CHILD_NFT_CONTRACT.into(),
        BTreeSet::from([child_token_id.into()]),
    );
    assert_eq!(
        pending_children_reply,
        RMRKStateReply::PendingChildren(pending_children)
    );
}

// Root owner transfers accepted child token to RMRK token that he does not own inside one contract
#[test]
fn transfer_accepted_child_to_token_with_different_owner() {
    let sys = System::new();
    before_test(&sys);
    let rmrk_child = sys.get_program(1);
    let rmrk_parent = sys.get_program(2);

    let child_token_id: u64 = 1;
    let parent_token_id: u64 = 10;
    let new_parent_token_id: u64 = 12;

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

    // USERS[0] transfer child to token that he does not own
    let res = transfer_to_nft(
        &rmrk_child,
        USERS[0],
        PARENT_NFT_CONTRACT,
        child_token_id,
        new_parent_token_id,
    );
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::TransferToNft {
            to: PARENT_NFT_CONTRACT.into(),
            token_id: child_token_id.into(),
            destination_id: new_parent_token_id.into(),
        }
        .encode()
    )));

    let rmrk_owner = owner(&rmrk_child, child_token_id);
    assert_eq!(
        rmrk_owner,
        RMRKStateReply::Owner {
            token_id: Some(new_parent_token_id.into()),
            owner_id: PARENT_NFT_CONTRACT.into(),
        }
    );

    // check accepted children of parent_token_id
    let accepted_children_reply = get_accepted_children(&rmrk_parent, parent_token_id);
    assert_eq!(
        accepted_children_reply,
        RMRKStateReply::AcceptedChildren(BTreeMap::new())
    );

    // check pending children of new_parent_token_id
    let pending_children_reply = get_pending_children(&rmrk_parent, new_parent_token_id);
    let mut pending_children: BTreeMap<ActorId, BTreeSet<TokenId>> = BTreeMap::new();
    pending_children.insert(
        CHILD_NFT_CONTRACT.into(),
        BTreeSet::from([child_token_id.into()]),
    );
    assert_eq!(
        pending_children_reply,
        RMRKStateReply::PendingChildren(pending_children)
    );
}

// Root owner transfers pending child token to  RMRK token that he does not own inside one contract
#[test]
fn transfer_pending_child_to_token_with_different_owner() {
    let sys = System::new();
    before_test(&sys);
    let rmrk_child = sys.get_program(1);
    let rmrk_parent = sys.get_program(2);

    let child_token_id: u64 = 1;
    let parent_token_id: u64 = 10;
    let new_parent_token_id: u64 = 12;

    // mint RMRK child token to RMRK parent token
    assert!(!mint_to_nft(
        &rmrk_child,
        USERS[3],
        PARENT_NFT_CONTRACT,
        parent_token_id,
        child_token_id,
    )
    .main_failed());

    // USERS[0] transfer child to another his token
    let res = transfer_to_nft(
        &rmrk_child,
        USERS[0],
        PARENT_NFT_CONTRACT,
        child_token_id,
        new_parent_token_id,
    );
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::TransferToNft {
            to: PARENT_NFT_CONTRACT.into(),
            token_id: child_token_id.into(),
            destination_id: new_parent_token_id.into(),
        }
        .encode()
    )));

    // check owner
    let rmrk_owner = owner(&rmrk_child, child_token_id);
    assert_eq!(
        rmrk_owner,
        RMRKStateReply::Owner {
            token_id: Some(new_parent_token_id.into()),
            owner_id: PARENT_NFT_CONTRACT.into(),
        }
    );

    // check accepted children of parent_token_id
    let accepted_children_reply = get_accepted_children(&rmrk_parent, parent_token_id);
    assert_eq!(
        accepted_children_reply,
        RMRKStateReply::AcceptedChildren(BTreeMap::new())
    );

    // check pending children of new_parent_token_id
    let pending_children_reply = get_pending_children(&rmrk_parent, new_parent_token_id);
    let mut pending_children: BTreeMap<ActorId, BTreeSet<TokenId>> = BTreeMap::new();
    pending_children.insert(
        CHILD_NFT_CONTRACT.into(),
        BTreeSet::from([child_token_id.into()]),
    );
    assert_eq!(
        pending_children_reply,
        RMRKStateReply::PendingChildren(pending_children)
    );
}

// Root owner transfers accepted child token to his RMRK token in another RMRK contract
#[test]
fn transfer_accepted_child_to_token_with_same_owner_another_contract() {
    let sys = System::new();
    before_test(&sys);
    init_rmrk(&sys);
    let rmrk_child = sys.get_program(1);
    let rmrk_parent = sys.get_program(2);
    let new_rmrk_parent = sys.get_program(3);

    let child_token_id: u64 = 1;
    let parent_token_id: u64 = 10;
    let new_parent_token_id: u64 = 8;
    let new_parent_contract_id: u64 = 3;

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

    // mint new parent rmrk token
    assert!(
        !mint_to_root_owner(&new_rmrk_parent, USERS[0], USERS[0], new_parent_token_id,)
            .main_failed()
    );

    // USERS[0] transfer child to another his token in another rmrk contract
    let res = transfer_to_nft(
        &rmrk_child,
        USERS[0],
        new_parent_contract_id,
        child_token_id,
        new_parent_token_id,
    );
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::TransferToNft {
            to: new_parent_contract_id.into(),
            token_id: child_token_id.into(),
            destination_id: new_parent_token_id.into(),
        }
        .encode()
    )));

    // check owner
    let rmrk_owner = owner(&rmrk_child, child_token_id);
    assert_eq!(
        rmrk_owner,
        RMRKStateReply::Owner {
            token_id: Some(new_parent_token_id.into()),
            owner_id: new_parent_contract_id.into(),
        }
    );

    // check accepted children of parent_token_id
    let accepted_children_reply = get_accepted_children(&rmrk_parent, parent_token_id);
    assert_eq!(
        accepted_children_reply,
        RMRKStateReply::AcceptedChildren(BTreeMap::new())
    );
    // check accepted children of new_parent_token_id
    let accepted_children_reply = get_accepted_children(&new_rmrk_parent, new_parent_token_id);
    let mut accepted_children: BTreeMap<ActorId, BTreeSet<TokenId>> = BTreeMap::new();
    accepted_children.insert(
        CHILD_NFT_CONTRACT.into(),
        BTreeSet::from([child_token_id.into()]),
    );
    assert_eq!(
        accepted_children_reply,
        RMRKStateReply::AcceptedChildren(accepted_children)
    );
}

// Root owner transfers accepted child token to  RMRK token with different owner in another RMRK contract
#[test]
fn transfer_accepted_child_to_token_with_different_owner_another_contract() {
    let sys = System::new();
    before_test(&sys);
    init_rmrk(&sys);
    let rmrk_child = sys.get_program(1);
    let rmrk_parent = sys.get_program(2);
    let new_rmrk_parent = sys.get_program(3);

    let child_token_id: u64 = 1;
    let parent_token_id: u64 = 10;
    let new_parent_token_id: u64 = 8;
    let new_parent_contract_id: u64 = 3;

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

    // mint new parent rmrk token
    assert!(
        !mint_to_root_owner(&new_rmrk_parent, USERS[1], USERS[1], new_parent_token_id,)
            .main_failed()
    );

    // USERS[0] transfer child to another his token in another rmrk contract
    let res = transfer_to_nft(
        &rmrk_child,
        USERS[0],
        new_parent_contract_id,
        child_token_id,
        new_parent_token_id,
    );
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::TransferToNft {
            to: new_parent_contract_id.into(),
            token_id: child_token_id.into(),
            destination_id: new_parent_token_id.into(),
        }
        .encode()
    )));

    // check owner
    let rmrk_owner = owner(&rmrk_child, child_token_id);
    assert_eq!(
        rmrk_owner,
        RMRKStateReply::Owner {
            token_id: Some(new_parent_token_id.into()),
            owner_id: new_parent_contract_id.into(),
        }
    );

    // check accepted children of parent_token_id
    let accepted_children_reply = get_accepted_children(&rmrk_parent, parent_token_id);
    assert_eq!(
        accepted_children_reply,
        RMRKStateReply::AcceptedChildren(BTreeMap::new())
    );

    // check pending children of new_parent_token_id
    let pending_children_reply = get_pending_children(&new_rmrk_parent, new_parent_token_id);
    let mut pending_children: BTreeMap<ActorId, BTreeSet<TokenId>> = BTreeMap::new();
    pending_children.insert(
        CHILD_NFT_CONTRACT.into(),
        BTreeSet::from([child_token_id.into()]),
    );
    assert_eq!(
        pending_children_reply,
        RMRKStateReply::PendingChildren(pending_children)
    );
}

// Root owner transfers usual token to his RMRK token
#[test]
fn transfer_token_to_token_with_same_owner() {
    let sys = System::new();
    before_test(&sys);
    init_rmrk(&sys);
    let rmrk_child = sys.get_program(1);
    let rmrk_parent = sys.get_program(2);

    let child_token_id: u64 = 1;
    let parent_token_id: u64 = 10;

    // mint future child token
    assert!(!mint_to_root_owner(&rmrk_child, USERS[0], USERS[0], child_token_id).main_failed());

    // USERS[0] transfer child to another his token in another rmrk contract
    let res = transfer_to_nft(
        &rmrk_child,
        USERS[0],
        PARENT_NFT_CONTRACT,
        child_token_id,
        parent_token_id,
    );
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::TransferToNft {
            to: PARENT_NFT_CONTRACT.into(),
            token_id: child_token_id.into(),
            destination_id: parent_token_id.into(),
        }
        .encode()
    )));

    // check owner
    let rmrk_owner = owner(&rmrk_child, child_token_id);
    assert_eq!(
        rmrk_owner,
        RMRKStateReply::Owner {
            token_id: Some(parent_token_id.into()),
            owner_id: PARENT_NFT_CONTRACT.into(),
        }
    );

    // check accepted children
    let accepted_children_reply = get_accepted_children(&rmrk_parent, parent_token_id);
    let mut accepted_children: BTreeMap<ActorId, BTreeSet<TokenId>> = BTreeMap::new();
    accepted_children.insert(
        CHILD_NFT_CONTRACT.into(),
        BTreeSet::from([child_token_id.into()]),
    );
    assert_eq!(
        accepted_children_reply,
        RMRKStateReply::AcceptedChildren(accepted_children)
    );
}

// Root owner transfers usual token to  RMRK token with different owner
#[test]
fn transfer_usual_token_to_token_with_different_owner() {
    let sys = System::new();
    before_test(&sys);
    init_rmrk(&sys);
    let rmrk_child = sys.get_program(1);
    let rmrk_parent = sys.get_program(2);

    let child_token_id: u64 = 1;
    let parent_token_id: u64 = 12;

    // mint future child token
    assert!(!mint_to_root_owner(&rmrk_child, USERS[0], USERS[0], child_token_id).main_failed());

    // USERS[0] transfer child to another his token in another rmrk contract
    let res = transfer_to_nft(
        &rmrk_child,
        USERS[0],
        PARENT_NFT_CONTRACT,
        child_token_id,
        parent_token_id,
    );
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::TransferToNft {
            to: PARENT_NFT_CONTRACT.into(),
            token_id: child_token_id.into(),
            destination_id: parent_token_id.into(),
        }
        .encode()
    )));

    // check owner
    let rmrk_owner = owner(&rmrk_child, child_token_id);
    assert_eq!(
        rmrk_owner,
        RMRKStateReply::Owner {
            token_id: Some(parent_token_id.into()),
            owner_id: PARENT_NFT_CONTRACT.into(),
        }
    );

    // check pending children of new_parent_token_id
    let pending_children_reply = get_pending_children(&rmrk_parent, parent_token_id);
    let mut pending_children: BTreeMap<ActorId, BTreeSet<TokenId>> = BTreeMap::new();
    pending_children.insert(
        CHILD_NFT_CONTRACT.into(),
        BTreeSet::from([child_token_id.into()]),
    );
    assert_eq!(
        pending_children_reply,
        RMRKStateReply::PendingChildren(pending_children)
    );
}

#[test]
fn transfer_to_token_failures() {
    let sys = System::new();
    before_test(&sys);
    let rmrk_child = sys.get_program(1);
    let rmrk_parent = sys.get_program(2);

    let child_token_id: u64 = 1;
    let parent_token_id: u64 = 10;
    let new_parent_token_id: u64 = 8;

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

    // must fail since USERS[1] is not root owner
    assert!(transfer_to_nft(
        &rmrk_child,
        USERS[1],
        PARENT_NFT_CONTRACT,
        new_parent_token_id,
        child_token_id,
    )
    .main_failed());

    // must fail since token does not exist
    assert!(transfer_to_nft(
        &rmrk_child,
        USERS[0],
        PARENT_NFT_CONTRACT,
        new_parent_token_id,
        child_token_id + 1
    )
    .main_failed());

    // must fail since destination token does not exist
    assert!(transfer_to_nft(
        &rmrk_child,
        USERS[0],
        PARENT_NFT_CONTRACT,
        new_parent_token_id + 100,
        child_token_id,
    )
    .main_failed());
}
