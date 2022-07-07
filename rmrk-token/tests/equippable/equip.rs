use crate::utils::*;
use codec::Encode;
use gtest::System;
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
    add_resource_entry(
        &rmrk_child,
        USERS[0],
        basic_resource_id,
        basic_resource.clone(),
    );
    assert!(
        !add_resource(&rmrk_child, USERS[0], child_token_id, basic_resource_id, 0).main_failed()
    );
    assert!(
        !accept_resource(&rmrk_child, USERS[0], child_token_id, basic_resource_id).main_failed()
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
    add_resource_entry(&rmrk_child, USERS[0], slot_resource_id, resource);
    assert!(
        !add_resource(&rmrk_child, USERS[0], child_token_id, slot_resource_id, 0).main_failed()
    );
    assert!(
        !accept_resource(&rmrk_child, USERS[0], child_token_id, slot_resource_id).main_failed()
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
    add_resource_entry(&rmrk_parent, USERS[0], basic_resource_id, basic_resource);
    assert!(!add_resource(
        &rmrk_parent,
        USERS[0],
        parent_token_id,
        basic_resource_id,
        0
    )
    .main_failed());
    assert!(
        !accept_resource(&rmrk_parent, USERS[0], parent_token_id, basic_resource_id).main_failed()
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

    add_resource_entry(&rmrk_parent, USERS[0], composed_resource_id, resource);
    assert!(!add_resource(
        &rmrk_parent,
        USERS[0],
        parent_token_id,
        composed_resource_id,
        0
    )
    .main_failed());
    assert!(!accept_resource(
        &rmrk_parent,
        USERS[0],
        parent_token_id,
        composed_resource_id
    )
    .main_failed());

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

//     let sys = System::new();
//     // Prepare resource
//     let code_hash_stored =
//         sys.submit_code("../target/wasm32-unknown-unknown/release/rmrk_resource.wasm");
//     // base
//     init_base(&sys);
//     let base = sys.get_program(BASE_ID);

//     // Soldier contract
//     init_soldier(&sys, code_hash_stored.into());
//     let soldier = sys.get_program(SOLDIER_ID);

//     // Weapon contract
//     init_weapon(&sys, code_hash_stored.into());
//     let weapon = sys.get_program(WEAPON_ID);

//     equip(
//         &soldier,
//         USERS[0],
//         SOLDIER_TOKEN_ID,
//         RESOURCE_COMPOSED_ID,
//         WEAPON_ID,
//         WEAPON_TOKEN_ID,
//         RESOURCE_SLOT_ID_1,
//         BASE_ID,
//         SLOT_ID_1,
//     );
// }

// fn init_soldier(system: &System, code_hash_stored: [u8; 32]) {
//     // Soldier contract
//     init_rmrk(&system, Some(code_hash_stored.into()));
//     let soldier = system.get_program(SOLDIER_ID);

//     // mint soldier
//     assert!(!mint_to_root_owner(&soldier, USERS[0], USERS[0], SOLDIER_TOKEN_ID).main_failed());

//     // create composed resource for soldire
//     let resource = Resource::Composed(ComposedResource {
//         base: BASE_ID.into(),
//         parts: vec![SLOT_ID_1, SLOT_ID_2],
//         ..Default::default()
//     });
//     // add resource to soldier contract
//     add_resource_entry(&soldier, USERS[0], RESOURCE_COMPOSED_ID, resource.clone());
//     // add resource to token
//     add_resource(
//         &soldier,
//         USERS[0],
//         SOLDIER_TOKEN_ID,
//         RESOURCE_COMPOSED_ID,
//         0,
//     );
//     accept_resource(&soldier, USERS[0], SOLDIER_TOKEN_ID, RESOURCE_COMPOSED_ID);
// }

// fn init_weapon(system: &System, code_hash_stored: [u8; 32]) {
//     // Weapon contract
//     init_rmrk(&system, Some(code_hash_stored.into()));
//     let soldier = system.get_program(SOLDIER_ID);
//     let weapon = system.get_program(WEAPON_ID);

//     // mint weapon to soldier
//     mint_to_nft(
//         &weapon,
//         USERS[0],
//         SOLDIER_ID,
//         SOLDIER_TOKEN_ID,
//         WEAPON_TOKEN_ID,
//     );
//     accept_child(
//         &soldier,
//         USERS[0],
//         SOLDIER_TOKEN_ID,
//         WEAPON_ID,
//         WEAPON_TOKEN_ID,
//     );

//     // create slot resource for weapon
//     let resource_1 = Resource::Slot(SlotResource {
//         base: BASE_ID.into(),
//         slot: SLOT_ID_1,
//         ..Default::default()
//     });
//     let resource_2 = Resource::Slot(SlotResource {
//         base: BASE_ID.into(),
//         slot: SLOT_ID_2,
//         ..Default::default()
//     });
//     // add slot resources to weapon contract
//     add_resource_entry(&weapon, USERS[0], RESOURCE_SLOT_ID_1, resource_1.clone());
//     add_resource_entry(&weapon, USERS[0], RESOURCE_SLOT_ID_2, resource_2.clone());

//     // add slot resources to token
//     add_resource(&weapon, USERS[0], WEAPON_TOKEN_ID, RESOURCE_SLOT_ID_1, 0);
//     accept_resource(&weapon, USERS[0], WEAPON_TOKEN_ID, RESOURCE_SLOT_ID_1);
//     add_resource(&weapon, USERS[0], WEAPON_TOKEN_ID, RESOURCE_SLOT_ID_2, 0);
//     accept_resource(&weapon, USERS[0], WEAPON_TOKEN_ID, RESOURCE_SLOT_ID_2);
// }

// fn equip(
//     soldier: &Program,
//     user: u64,
//     token_id: u64,
//     resource_id: ResourceId,
//     child_contract_id: u64,
//     child_token_id: u64,
//     child_resource_id: ResourceId,
//     base_id: u64,
//     slot_id: SlotId,
// ) {
//     let res = soldier.send(
//         user,
//         RMRKAction::Equip {
//             token_id: token_id.into(),
//             resource_id,
//             child_contract_id: child_contract_id.into(),
//             child_token_id: child_token_id.into(),
//             child_resource_id,
//             base_id: base_id.into(),
//             slot_id,
//         },
//     );
//     assert!(res.contains(&(
//         user,
//         RMRKEvent::TokenEquip {
//             token_id: token_id.into(),
//             resource_id: resource_id,
//             child_contract_id: child_contract_id.into(),
//             child_token_id: child_token_id.into(),
//             child_resource_id: child_resource_id,
//             base_id: base_id.into(),
//             slot_id: slot_id,
//         }
//         .encode()
//     )));
// }
