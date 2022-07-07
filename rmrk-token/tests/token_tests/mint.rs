use crate::utils::*;
use codec::Encode;
use gstd::BTreeSet;
use gtest::System;
use rmrk_io::*;
use types::primitives::{CollectionId, TokenId};

#[test]
fn mint_to_root_owner_success() {
    let sys = System::new();
    before_token_test(&sys);
    let rmrk = sys.get_program(1);
    let token_id: u64 = 100;
    let res = mint_to_root_owner(&rmrk, USERS[0], USERS[2], token_id);
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::MintToRootOwner {
            root_owner: USERS[2].into(),
            token_id: token_id.into(),
        }
        .encode()
    )));

    // check the rmrk owner
    check_rmrk_owner(&rmrk, token_id, None, USERS[2]);

    // check the owner balance
    let balance = balance(&rmrk, USERS[2]);
    assert_eq!(balance, RMRKStateReply::Balance(1.into()));
}

#[test]
fn mint_to_root_owner_failures() {
    let sys = System::new();
    before_token_test(&sys);
    let rmrk_parent = sys.get_program(PARENT_NFT_CONTRACT);
    // mints already minted token
    assert!(mint_to_root_owner(&rmrk_parent, USERS[1], USERS[1], 1).main_failed());
    // mints to zero address
    assert!(mint_to_root_owner(&rmrk_parent, USERS[1], ZERO_ID, 1).main_failed());
}

#[test]
fn mint_to_nft_failures() {
    let sys = System::new();
    before_token_test(&sys);
    let rmrk_child = sys.get_program(CHILD_NFT_CONTRACT);

    let child_token_id: u64 = 1;
    let parent_token_id: u64 = 10;
    let wrong_parent_token_id: u64 = 100;
    // nest mint to a non-existent token
    assert!(mint_to_nft(
        &rmrk_child,
        USERS[1],
        PARENT_NFT_CONTRACT,
        wrong_parent_token_id,
        parent_token_id,
    )
    .main_failed());

    // mint RMRK child token to RMRK parent token
    assert!(!mint_to_nft(
        &rmrk_child,
        USERS[1],
        PARENT_NFT_CONTRACT,
        parent_token_id,
        child_token_id
    )
    .main_failed());
    // nest mint already minted token
    assert!(mint_to_nft(
        &rmrk_child,
        USERS[1],
        PARENT_NFT_CONTRACT,
        parent_token_id,
        child_token_id
    )
    .main_failed());
    // nest mint already minted token to a different parent
    assert!(mint_to_nft(
        &rmrk_child,
        USERS[1],
        PARENT_NFT_CONTRACT,
        12,
        child_token_id
    )
    .main_failed());
    // nest mint to zero address (TO DO)
    // assert!(mint_to_nft(&rmrk_child, USERS[1], ZERO_ID, 2.into(), 12.into()).main_failed());
}

#[test]
fn mint_to_nft_success() {
    let sys = System::new();
    before_token_test(&sys);
    let rmrk_child = sys.get_program(CHILD_NFT_CONTRACT);
    let rmrk_parent = sys.get_program(PARENT_NFT_CONTRACT);
    let parent_token_id: u64 = 10;

    let mut pending_children: BTreeSet<(CollectionId, TokenId)> = BTreeSet::new();
    // mint  RMRK children
    for child_token_id in 0..10_u64 {
        let res = mint_to_nft(
            &rmrk_child,
            USERS[1],
            PARENT_NFT_CONTRACT,
            parent_token_id,
            child_token_id,
        );
        assert!(res.contains(&(
            USERS[1],
            RMRKEvent::MintToNft {
                parent_id: PARENT_NFT_CONTRACT.into(),
                parent_token_id: parent_token_id.into(),
                token_id: child_token_id.into(),
            }
            .encode()
        )));

        // check that owner is another NFT in parent token contract
        check_rmrk_owner(
            &rmrk_child,
            child_token_id,
            Some(parent_token_id.into()),
            PARENT_NFT_CONTRACT,
        );

        // add to pending children
        pending_children.insert((CHILD_NFT_CONTRACT.into(), child_token_id.into()));
    }
    // another RMRK child contract
    init_rmrk(&sys, None);
    let rmrk_child_2_id: u64 = 3;
    let rmrk_child_2 = sys.get_program(rmrk_child_2_id);
    for child_token_id in 0..20_u64 {
        let res = mint_to_nft(
            &rmrk_child_2,
            USERS[1],
            PARENT_NFT_CONTRACT,
            parent_token_id,
            child_token_id,
        );
        assert!(res.contains(&(
            USERS[1],
            RMRKEvent::MintToNft {
                parent_id: PARENT_NFT_CONTRACT.into(),
                parent_token_id: parent_token_id.into(),
                token_id: child_token_id.into(),
            }
            .encode()
        )));

        // check that owner is NFT in parent contract
        check_rmrk_owner(
            &rmrk_child_2,
            child_token_id,
            Some(parent_token_id.into()),
            PARENT_NFT_CONTRACT,
        );

        //insert pending children
        pending_children.insert((rmrk_child_2_id.into(), child_token_id.into()));
    }
    // check pending children
    check_pending_children(&rmrk_parent, parent_token_id, pending_children);
}

#[test]
fn mint_child_to_child() {
    let sys = System::new();
    before_token_test(&sys);
    let rmrk_child = sys.get_program(CHILD_NFT_CONTRACT);
    // grand child contract
    init_rmrk(&sys, None);
    let rmrk_grand_child = sys.get_program(3);
    let parent_token_id: u64 = 10;
    let child_token_id: u64 = 1;
    let grand_child_id: u64 = 2;
    // mint child_token_id to parent_token_id
    let res = mint_to_nft(
        &rmrk_child,
        USERS[1],
        PARENT_NFT_CONTRACT,
        parent_token_id,
        child_token_id,
    );
    assert!(res.contains(&(
        USERS[1],
        RMRKEvent::MintToNft {
            parent_id: PARENT_NFT_CONTRACT.into(),
            parent_token_id: parent_token_id.into(),
            token_id: child_token_id.into(),
        }
        .encode()
    )));

    // mint grand_token_id to child_token_id
    let res = mint_to_nft(
        &rmrk_grand_child,
        USERS[1],
        CHILD_NFT_CONTRACT,
        child_token_id,
        grand_child_id,
    );
    assert!(res.contains(&(
        USERS[1],
        RMRKEvent::MintToNft {
            parent_id: CHILD_NFT_CONTRACT.into(),
            parent_token_id: child_token_id.into(),
            token_id: grand_child_id.into(),
        }
        .encode()
    )));
    // root owner of grand_token_id must be USERS[0]
    let res = get_root_owner(&rmrk_grand_child, grand_child_id.into());
    assert!(res.contains(&(10, RMRKEvent::RootOwner(USERS[0].into(),).encode())));
}
