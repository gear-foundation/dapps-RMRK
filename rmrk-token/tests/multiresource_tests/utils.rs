use codec::Encode;
use gtest::{Program, RunResult, System};
use resource_io::{Resource, ResourceId};
use rmrk_io::*;
use gstd::BTreeSet;
pub const USERS: &[u64] = &[5, 6, 7, 8];

pub fn init_rmrk(sys: &System, code_hash: [u8; 32]) {
    sys.init_logger();
    let rmrk = Program::current(sys);
    let res = rmrk.send(
        USERS[0],
        InitRMRK {
            name: "RMRKToken".to_string(),
            symbol: "RMRKSymbol".to_string(),
            resource_name: "ResourceName".to_string(),
            resource_hash: Some(code_hash),
        },
    );
    //println!("{:?}", res.decoded_log::<RMRKEvent>());
    assert!(!res.log().is_empty());
}

pub fn before_test(sys: &System) {
    // Prepare resource
    let code_hash_stored =
        sys.submit_code("../target/wasm32-unknown-unknown/release/rmrk_resource.wasm");
    // rmrk contract
    init_rmrk(sys, code_hash_stored.into());
    let rmrk = sys.get_program(1);
    // mint parents tokens
    assert!(!mint_to_root_owner(&rmrk, USERS[0], USERS[0], 10).main_failed());
    assert!(!mint_to_root_owner(&rmrk, USERS[0], USERS[0], 11).main_failed());
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

pub fn add_resource_entry(rmrk: &Program, user: u64, resource: Resource) {
    let res = rmrk.send(
        user,
        RMRKAction::AddResourceEntry {
            id: resource.id,
            src: resource.src,
            thumb: resource.thumb,
            metadata_uri: resource.metadata_uri,
        },
    );

    assert!(res.contains(&(
        user,
        RMRKEvent::ResourceEntryAdded { id: resource.id }.encode()
    )));
}

// pub fn get_resource(storage: &Program, id: u8) -> RunResult {
//     storage.send(10, ResourceAction::GetResource { id })
// }

pub fn add_resource(
    rmrk: &Program,
    user: u64,
    token_id: u64,
    resource_id: u8,
    overwrite_id: u8,
) -> RunResult {
    rmrk.send(
        user,
        RMRKAction::AddResource {
            token_id: token_id.into(),
            resource_id,
            overwrite_id,
        },
    )
}

pub fn accept_resource(rmrk: &Program, user: u64, token_id: u64, resource_id: u8) -> RunResult {
    rmrk.send(
        user,
        RMRKAction::AcceptResource {
            token_id: token_id.into(),
            resource_id,
        },
    )
}

pub fn reject_resource(rmrk: &Program, user: u64, token_id: u64, resource_id: u8) -> RunResult {
    rmrk.send(
        user,
        RMRKAction::RejectResource {
            token_id: token_id.into(),
            resource_id,
        },
    )
}

// reading the pending resources of token
pub fn get_pending_resources(rmrk: &Program, token_id: u64) -> RMRKStateReply {
    rmrk.meta_state(RMRKState::PendingResources(token_id.into()))
}

// reading the active resource of token
pub fn get_active_resources(rmrk: &Program, token_id: u64) -> RMRKStateReply {
    rmrk.meta_state(RMRKState::ActiveResources(token_id.into()))
}

pub fn set_priority(rmrk: &Program, user: u64, token_id: u64, priorities: Vec<u8>) -> RunResult {
    rmrk.send(
        user,
        RMRKAction::SetPriority {
            token_id: token_id.into(),
            priorities,
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

pub fn check_pending_resources(rmrk: &Program, token_id: u64, expected_pending_resources: BTreeSet<ResourceId>) {
    let pending_resources = get_pending_resources(rmrk, token_id);
    assert_eq!(
        pending_resources,
        RMRKStateReply::PendingResources(expected_pending_resources)
    );
}

pub fn check_active_resources(rmrk: &Program, token_id: u64, expected_active_resources: BTreeSet<ResourceId>) {
    let active_resources = get_active_resources(rmrk, token_id);
    assert_eq!(
        active_resources,
        RMRKStateReply::ActiveResources(expected_active_resources)
    );
}

