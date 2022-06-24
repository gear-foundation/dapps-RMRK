use gtest::{Program, RunResult, System};
use rmrk_io::*;
pub const USERS: &[u64] = &[5, 6, 7, 8];
pub const ZERO_ID: u64 = 0;
pub const PARENT_NFT_CONTRACT: u64 = 2;
pub const CHILD_NFT_CONTRACT: u64 = 1;

pub fn init_rmrk(sys: &System) {
    sys.init_logger();
    let rmrk = Program::from_file(sys, "../target/wasm32-unknown-unknown/release/rmrk.wasm");
    let res = rmrk.send(
        USERS[0],
        InitRMRK {
            name: "RMRKToken".to_string(),
            symbol: "RMRKSymbol".to_string(),
            resource_hash: Some([0; 32]),
            resource_name: "ResourceName".to_string(),
        },
    );
    assert!(res.log().is_empty());
}

pub fn before_test(sys: &System) {
    // child contract
    init_rmrk(sys);
    // parent contract
    init_rmrk(sys);
    let rmrk_parent = sys.get_program(2);
    // mint parents tokens
    for i in 1..11 {
        mint_to_root_owner(&rmrk_parent, USERS[0], USERS[0], i as u64);
    }
    for i in 11..20 {
        mint_to_root_owner(&rmrk_parent, USERS[1], USERS[1], i as u64);
    }
}
pub fn mint_to_root_owner(rmrk: &Program, user: u64, root_owner: u64, token_id: u64) -> RunResult {
    rmrk.send(
        user,
        RMRKAction::MintToRootOwner {
            root_owner: root_owner.into(),
            token_id: token_id.into(),
        },
    )
}

pub fn mint_to_nft(
    rmrk: &Program,
    user: u64,
    parent_id: u64,
    parent_token_id: u64,
    token_id: u64,
) -> RunResult {
    rmrk.send(
        user,
        RMRKAction::MintToNft {
            parent_id: parent_id.into(),
            parent_token_id: parent_token_id.into(),
            token_id: token_id.into(),
        },
    )
}

pub fn accept_child(
    rmrk: &Program,
    user: u64,
    parent_token_id: u64,
    child_contract_id: u64,
    child_token_id: u64,
) -> RunResult {
    rmrk.send(
        user,
        RMRKAction::AcceptChild {
            parent_token_id: parent_token_id.into(),
            child_contract_id: child_contract_id.into(),
            child_token_id: child_token_id.into(),
        },
    )
}

pub fn reject_child(
    rmrk: &Program,
    user: u64,
    parent_token_id: u64,
    child_contract_id: u64,
    child_token_id: u64,
) -> RunResult {
    rmrk.send(
        user,
        RMRKAction::RejectChild {
            parent_token_id: parent_token_id.into(),
            child_contract_id: child_contract_id.into(),
            child_token_id: child_token_id.into(),
        },
    )
}

pub fn remove_child(
    rmrk: &Program,
    user: u64,
    parent_token_id: u64,
    child_contract_id: u64,
    child_token_id: u64,
) -> RunResult {
    rmrk.send(
        user,
        RMRKAction::RemoveChild {
            parent_token_id: parent_token_id.into(),
            child_contract_id: child_contract_id.into(),
            child_token_id: child_token_id.into(),
        },
    )
}
pub fn approve(rmrk: &Program, user: u64, to: u64, token_id: TokenId) -> RunResult {
    rmrk.send(
        user,
        RMRKAction::Approve {
            to: to.into(),
            token_id,
        },
    )
}

pub fn burn(rmrk: &Program, user: u64, token_id: u64) -> RunResult {
    rmrk.send(user, RMRKAction::Burn(token_id.into()))
}

pub fn transfer(rmrk: &Program, from: u64, to: u64, token_id: TokenId) -> RunResult {
    rmrk.send(
        from,
        RMRKAction::Transfer {
            to: to.into(),
            token_id,
        },
    )
}

pub fn transfer_to_nft(
    rmrk: &Program,
    from: u64,
    to: u64,
    token_id: u64,
    destination_id: u64,
) -> RunResult {
    rmrk.send(
        from,
        RMRKAction::TransferToNft {
            to: to.into(),
            token_id: token_id.into(),
            destination_id: destination_id.into(),
        },
    )
}

pub fn get_root_owner(rmrk: &Program, token_id: TokenId) -> RunResult {
    rmrk.send(10, RMRKAction::RootOwner(token_id))
}

// ownership chain is  USERS[0] > parent_token_id > child_token_id > grand_token_id
pub fn rmrk_chain(
    rmrk_grand: &Program,
    rmrk_child: &Program,
    rmrk_parent: &Program,
    grand_token_id: u64,
    child_token_id: u64,
    parent_token_id: u64,
) {
    // mint child_token_id to parent_token_id
    assert!(!mint_to_nft(
        rmrk_child,
        USERS[1],
        PARENT_NFT_CONTRACT,
        parent_token_id,
        child_token_id,
    )
    .main_failed());

    // accept child
    assert!(!accept_child(
        rmrk_parent,
        USERS[0],
        parent_token_id,
        CHILD_NFT_CONTRACT,
        child_token_id,
    )
    .main_failed());

    // mint grand_token_id to child_token_id
    assert!(!mint_to_nft(
        rmrk_grand,
        USERS[1],
        CHILD_NFT_CONTRACT,
        child_token_id,
        grand_token_id,
    )
    .main_failed());

    // accept child
    assert!(!accept_child(rmrk_child, USERS[0], child_token_id, 3, grand_token_id,).main_failed());
}

// reading the token owner
pub fn owner(rmrk: &Program, token_id: u64) -> RMRKStateReply {
    rmrk.meta_state(&RMRKState::Owner(token_id.into()))
}

// reading the account balance
pub fn balance(rmrk: &Program, account: u64) -> RMRKStateReply {
    rmrk.meta_state(RMRKState::Balance(account.into()))
}

// reading the pending children of token
pub fn get_pending_children(rmrk: &Program, token_id: u64) -> RMRKStateReply {
    rmrk.meta_state(RMRKState::PendingChildren(token_id.into()))
}

// reading the accepted children of token
pub fn get_accepted_children(rmrk: &Program, token_id: u64) -> RMRKStateReply {
    rmrk.meta_state(RMRKState::AcceptedChildren(token_id.into()))
}

