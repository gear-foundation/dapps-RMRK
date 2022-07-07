use crate::utils::*;
use codec::Encode;
use gtest::{Program, System};
use resource_io::*;
use rmrk_io::*;
use types::primitives::{CollectionAndToken, ResourceId};

#[test]
fn equip_test() {
    let sys = System::new();
    // Prepare resource
    let code_hash_stored = sys.submit_code("../target/wasm-projects/release/rmrk_resource.wasm");
    // init child contract with resource
    init_rmrk(&sys, Some(code_hash_stored.into()));
    // init parent contract with resource
    init_rmrk(&sys, Some(code_hash_stored.into()));

    // init base contract
    init_base(&sys);

    let rmrk_child = sys.get_program(CHILD_NFT_CONTRACT);
    let rmrk_parent = sys.get_program(PARENT_NFT_CONTRACT);
    let parent_token_id: u64 = 200;
    let child_token_id: u64 = 205;
    let slot_part_id = 400;

    let equippable: CollectionAndToken = (PARENT_NFT_CONTRACT.into(), parent_token_id.into());
    // mint parent token
    assert!(!mint_to_root_owner(&rmrk_parent, USERS[0], USERS[0], parent_token_id).main_failed());

    // mint child token
    assert!(!mint_to_root_owner(&rmrk_child, USERS[0], USERS[0], child_token_id).main_failed());

    // equip child token: fail since token has no resource
    assert!(equip(
        &rmrk_child,
        child_token_id,
        CHILD_RESOURCE_ID,
        equippable,
        PARENT_RESOURCE_ID
    )
    .main_failed());

    // add basic resource to child token
    let basic_resource_id: ResourceId = 10;
    let basic_resource = Resource::Basic(Default::default());
    add_resource_to_token(
        &rmrk_child,
        child_token_id,
        basic_resource_id,
        basic_resource.clone(),
    );

    // equip child token: fail since the indicated resource is not slot
    assert!(equip(
        &rmrk_child,
        child_token_id,
        basic_resource_id,
        equippable,
        PARENT_RESOURCE_ID
    )
    .main_failed());

    // add slot resource for child token
    let slot_resource_id: ResourceId = 11;
    let resource = Resource::Slot(SlotResource {
        base: BASE_ID.into(),
        slot: slot_part_id,
        ..Default::default()
    });
    add_resource_to_token(
        &rmrk_child,
        child_token_id,
        slot_resource_id,
        resource,
    );

    // equip child token: must fail token is not owned by another token
    assert!(equip(
        &rmrk_child,
        child_token_id,
        slot_resource_id,
        equippable,
        PARENT_RESOURCE_ID
    )
    .main_failed());

    // transfer child token to parent token
    assert!(!transfer_to_nft(
        &rmrk_child,
        USERS[0],
        PARENT_NFT_CONTRACT,
        child_token_id,
        parent_token_id,
    )
    .main_failed());

    // add basic resource to parent token
    add_resource_to_token(
        &rmrk_parent,
        parent_token_id,
        basic_resource_id,
        basic_resource,
    );

    // equip child token: must fail since parent's resource is not composed
    assert!(equip(
        &rmrk_child,
        child_token_id,
        slot_resource_id,
        equippable,
        basic_resource_id
    )
    .main_failed());

    // add composed resource to parent token
    let composed_resource_id: ResourceId = 11;
    let resource = Resource::Composed(ComposedResource {
        base: BASE_ID.into(),
        ..Default::default()
    });

    add_resource_to_token(
        &rmrk_parent,
        parent_token_id,
        composed_resource_id,
        resource,
    );

    // should equip
    let res = equip(
        &rmrk_child,
        child_token_id,
        slot_resource_id,
        equippable,
        composed_resource_id,
    );

    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::TokenEquipped {
            token_id: child_token_id.into(),
            resource_id: slot_resource_id,
            slot_id: slot_part_id,
            equippable,
        }
        .encode()
    )));

    // must fail since token is already equipped
    assert!(equip(
        &rmrk_child,
        child_token_id,
        slot_resource_id,
        equippable,
        basic_resource_id
    )
    .main_failed());
}

fn add_resource_to_token(
    rmrk: &Program,
    token_id: u64,
    resource_id: ResourceId,
    resource: Resource,
) {
    add_resource_entry(rmrk, USERS[0], resource_id, resource);
    assert!(!add_resource(rmrk, USERS[0], token_id, resource_id, 0).main_failed());
    assert!(!accept_resource(rmrk, USERS[0], token_id, resource_id).main_failed());
}
