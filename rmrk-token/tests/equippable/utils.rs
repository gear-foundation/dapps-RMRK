use base_io::*;
use codec::Encode;
use gstd::{ActorId, BTreeMap, BTreeSet};
use gtest::{Program, RunResult, System};
use resource_io::*;
use rmrk_io::*;
pub const USERS: &[u64] = &[10, 11, 12, 13];
pub const ZERO_ID: u64 = 0;
pub const BASE_ID: u64 = 1;
pub const SOLDIER_ID: u64 = 2;
pub const SOLDIER_TOKEN_ID: u64 = 20;
pub const WEAPON_ID: u64 = 3;
pub const WEAPON_TOKEN_ID: u64 = 30;
pub const WEAPON_GEM_ID: u64 = 4;
pub const BACKGROUND_ID: u64 = 5;

pub fn init_rmrk(sys: &System, resource_hash: Option<[u8; 32]>) {
    sys.init_logger();
    let rmrk = Program::from_file(sys, "../target/wasm32-unknown-unknown/release/rmrk.wasm");
    let res = rmrk.send(
        USERS[0],
        InitRMRK {
            name: "RMRKToken".to_string(),
            symbol: "RMRKSymbol".to_string(),
            resource_hash,
            resource_name: "ResourceName".to_string(),
        },
    );
    if resource_hash.is_some() {
        println!("{:?}", res.decoded_log::<RMRKEvent>());
        assert!(!res.log().is_empty());
    } else {
        assert!(res.log().is_empty());
    }
}

pub fn init_base(sys: &System) {
    let base = Program::from_file(
        sys,
        "../target/wasm32-unknown-unknown/release/rmrk_base.wasm",
    );
    let res = base.send(
        USERS[0],
        InitBase {
            base_type: "svg".to_string(),
            symbol: "".to_string(),
        },
    );
    assert!(res.log().is_empty());
}


pub fn add_resource_entry(rmrk: &Program, user: u64, resource_id: ResourceId, resource: Resource) {
    let res = rmrk.send(
        user,
        RMRKAction::AddResourceEntry{resource_id, resource: resource.clone()},
    );

    assert!(res.contains(&(
        user,
        RMRKEvent::ResourceEntryAdded (resource).encode()
    )));
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
