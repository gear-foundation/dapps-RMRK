use base_io::{BaseAction, EquippableList, FixedPart, InitBase, Part, SlotPart};
use codec::Encode;
use gstd::{BTreeMap, BTreeSet};
use gtest::{Program, RunResult, System};
use resource_io::Resource;
use rmrk_io::*;
use types::primitives::{CollectionAndToken, CollectionId, PartId, ResourceId, TokenId};
pub const USERS: &[u64] = &[10, 11, 12, 13];
pub const ZERO_ID: u64 = 0;
pub const PARENT_NFT_CONTRACT: u64 = 2;
pub const CHILD_NFT_CONTRACT: u64 = 1;

pub const BASE_ID: u64 = 3;
pub const CHILD_RESOURCE_ID: ResourceId = 150;
pub const PARENT_RESOURCE_ID: ResourceId = 151;

pub fn init_rmrk(sys: &System, resource_hash: Option<[u8; 32]>) {
    sys.init_logger();
    let rmrk = Program::current(sys);
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
    let base = Program::from_file(sys, "../target/wasm-projects/release/rmrk_base.wasm");
    let res = base.send(
        USERS[0],
        InitBase {
            base_type: "svg".to_string(),
            symbol: "".to_string(),
        },
    );
    assert!(res.log().is_empty());

    let mut parts: BTreeMap<PartId, Part> = BTreeMap::new();
    // setup base
    let fixed_part_body_id = 100;
    let fixed_part_body = FixedPart {
        z: Some(0),
        src: "body-1".to_string(),
    };
    parts.insert(fixed_part_body_id, Part::Fixed(fixed_part_body));

    // Slot part left hand can equip items from collections 0 or 1
    let slot_part_left_hand_id = 400;
    let slot_part_left_hand = SlotPart {
        z: Some(0),
        src: "left-hand".to_string(),
        equippable: EquippableList::All,
    };
    parts.insert(slot_part_left_hand_id, Part::Slot(slot_part_left_hand));
    // add parts to base
    assert!(!base
        .send(USERS[0], BaseAction::AddParts(parts))
        .main_failed());
}

pub fn before_token_test(sys: &System) {
    // child contract
    init_rmrk(sys, None);
    // parent contract
    init_rmrk(sys, None);
    let rmrk_parent = sys.get_program(2);
    // mint parents tokens
    for i in 1..11 {
        mint_to_root_owner(&rmrk_parent, USERS[0], USERS[0], i as u64);
    }
    for i in 11..20 {
        mint_to_root_owner(&rmrk_parent, USERS[1], USERS[1], i as u64);
    }
}

pub fn before_multiresource_test(sys: &System) {
    // Prepare resource
    let code_hash_stored = sys.submit_code("../target/wasm-projects/release/rmrk_resource.wasm");
    // rmrk contract
    init_rmrk(sys, Some(code_hash_stored.into()));
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
        .expect("Meta_state failed")
}

// reading the account balance
pub fn balance(rmrk: &Program, account: u64) -> RMRKStateReply {
    rmrk.meta_state(RMRKState::Balance(account.into()))
        .expect("Meta_state failed")
}

// reading the pending children of token
pub fn get_pending_children(rmrk: &Program, token_id: u64) -> RMRKStateReply {
    rmrk.meta_state(RMRKState::PendingChildren(token_id.into()))
        .expect("Meta_state failed")
}

// reading the accepted children of token
pub fn get_accepted_children(rmrk: &Program, token_id: u64) -> RMRKStateReply {
    rmrk.meta_state(RMRKState::AcceptedChildren(token_id.into()))
        .expect("Meta_state failed")
}

pub fn check_rmrk_owner(
    rmrk: &Program,
    token_id: u64,
    expected_token_id: Option<TokenId>,
    owner_id: u64,
) {
    let rmrk_owner = owner(rmrk, token_id);
    assert_eq!(
        rmrk_owner,
        RMRKStateReply::Owner {
            token_id: expected_token_id,
            owner_id: owner_id.into(),
        }
    );
}

pub fn check_pending_children(
    rmrk: &Program,
    token_id: u64,
    expected_pending_children: BTreeSet<(CollectionId, TokenId)>,
) {
    let pending_children = get_pending_children(rmrk, token_id);
    assert_eq!(
        pending_children,
        RMRKStateReply::PendingChildren(expected_pending_children),
    );
}

pub fn check_accepted_children(
    rmrk: &Program,
    token_id: u64,
    expected_accepted_children: BTreeSet<(CollectionId, TokenId)>,
) {
    let accepted_children = get_accepted_children(rmrk, token_id);
    assert_eq!(
        accepted_children,
        RMRKStateReply::AcceptedChildren(expected_accepted_children),
    );
}

pub fn add_resource_entry(rmrk: &Program, user: u64, resource_id: ResourceId, resource: Resource) {
    let res = rmrk.send(
        user,
        RMRKAction::AddResourceEntry {
            resource_id,
            resource: resource.clone(),
        },
    );
    assert!(res.contains(&(user, RMRKEvent::ResourceEntryAdded(resource).encode())));
}

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
        .expect("Meta_state failed")
}

// reading the active resource of token
pub fn get_active_resources(rmrk: &Program, token_id: u64) -> RMRKStateReply {
    rmrk.meta_state(RMRKState::ActiveResources(token_id.into()))
        .expect("Meta_state failed")
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

pub fn check_pending_resources(
    rmrk: &Program,
    token_id: u64,
    expected_pending_resources: BTreeSet<ResourceId>,
) {
    let pending_resources = get_pending_resources(rmrk, token_id);
    assert_eq!(
        pending_resources,
        RMRKStateReply::PendingResources(expected_pending_resources)
    );
}

pub fn check_active_resources(
    rmrk: &Program,
    token_id: u64,
    expected_active_resources: BTreeSet<ResourceId>,
) {
    let active_resources = get_active_resources(rmrk, token_id);
    assert_eq!(
        active_resources,
        RMRKStateReply::ActiveResources(expected_active_resources)
    );
}

pub fn equip(
    rmrk: &Program,
    token_id: u64,
    resource_id: ResourceId,
    equippable: CollectionAndToken,
    equippable_resource_id: ResourceId,
) -> RunResult {
    rmrk.send(
        USERS[0],
        RMRKAction::Equip {
            token_id: token_id.into(),
            resource_id,
            equippable,
            equippable_resource_id,
        },
    )
}
