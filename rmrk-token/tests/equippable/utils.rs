use catalog_io::*;
use gstd::{prelude::*, ActorId};
use gtest::{Program, System};
use rmrk_io::*;
use types::primitives::{TokenId, PartId};
const CATALOG_ID: u64 = 100;
const PATH_TO_CATALOG: &str = "../target/wasm32-unknown-unknown/release/rmrk_catalog.opt.wasm";
const ADMIN: u64 = 200;
const KANARIA_ID: u64 = 10;
const GEM_ID: u64 = 11;

pub fn setup_catalog(system: &System) {
    let mut parts = BTreeMap::new();

    let catalog = Program::from_file_with_id(system, CATALOG_ID, PATH_TO_CATALOG);
    let res = catalog.send(
        ADMIN,
        InitCatalog {
            catalog_type: "svg".to_string(),
            symbol: "CatalogSymbol".to_string(),
        },
    );
    assert!(!res.main_failed());

    let part_id_for_back_1 = 1;
    let part_for_back_1 = Part::Fixed(FixedPart {
        z: Some(0),
        metadata_uri: String::from("ipfs://backgrounds/1.svg"),
    });
    parts.insert(part_id_for_back_1, part_for_back_1);

    let part_id_for_back_2 = 2;
    let part_for_back_2 = Part::Fixed(FixedPart {
        z: Some(0),
        metadata_uri: String::from("ipfs://backgrounds/2.svg"),
    });
    parts.insert(part_id_for_back_2, part_for_back_2);

    let part_id_for_head_1 = 3;
    let part_for_head_1 = Part::Fixed(FixedPart {
        z: Some(3),
        metadata_uri: String::from("ipfs://heads/1.svg"),
    });
    parts.insert(part_id_for_head_1, part_for_head_1);

    let part_id_for_head_2 = 4;
    let part_for_head_2 = Part::Fixed(FixedPart {
        z: Some(3),
        metadata_uri: String::from("ipfs://heads/2.svg"),
    });
    parts.insert(part_id_for_head_2, part_for_head_2);

    let part_id_for_body_1 = 5;
    let part_for_body_1 = Part::Fixed(FixedPart {
        z: Some(2),
        metadata_uri: String::from("ipfs://body/1.svg"),
    });
    parts.insert(part_id_for_body_1, part_for_body_1);

    let part_id_for_body_2 = 6;
    let part_for_body_2 = Part::Fixed(FixedPart {
        z: Some(2),
        metadata_uri: String::from("ipfs://body/2.svg"),
    });
    parts.insert(part_id_for_body_2, part_for_body_2);

    let part_id_for_wings_1 = 5;
    let part_for_wings_1 = Part::Fixed(FixedPart {
        z: Some(4),
        metadata_uri: String::from("ipfs://wings/1.svg"),
    });
    parts.insert(part_id_for_wings_1, part_for_wings_1);

    let part_id_for_wings_2 = 6;
    let part_for_wings_2 = Part::Fixed(FixedPart {
        z: Some(4),
        metadata_uri: String::from("ipfs://wings/2.svg"),
    });
    parts.insert(part_id_for_wings_2, part_for_wings_2);

    let part_id_for_gem_slot_1 = 7;
    let part_for_gem_slot_1 = Part::Slot(SlotPart {
        equippable: vec![GEM_ID.into()],
        z: Some(4),
        metadata_uri: String::from(""),
    });
    parts.insert(part_id_for_gem_slot_1, part_for_gem_slot_1);

    let part_id_for_gem_slot_2 = 8;
    let part_for_gem_slot_2 = Part::Slot(SlotPart {
        equippable: vec![GEM_ID.into()],
        z: Some(4),
        metadata_uri: String::from(""),
    });
    parts.insert(part_id_for_gem_slot_1, part_for_gem_slot_2);

    let part_id_for_gem_slot_3 = 9;
    let part_for_gem_slot_3 = Part::Slot(SlotPart {
        equippable: vec![GEM_ID.into()],
        z: Some(4),
        metadata_uri: String::from(""),
    });
    parts.insert(part_id_for_gem_slot_3, part_for_gem_slot_3);

    let result = catalog.send(ADMIN, CatalogAction::AddParts(parts.clone()));
    let expected_reply: Result<CatalogReply, CatalogError> = Ok(CatalogReply::PartsAdded(parts));
    assert!(result.contains(&(ADMIN, expected_reply.encode())));
}

pub fn mint_tokens(system: &System) {
    let kanaria = Program::current_with_id(system, KANARIA_ID);

    let res = kanaria.send(
        ADMIN,
        InitRMRK {
            name: "Kanaria".to_string(),
            symbol: "KAN".to_string(),
            resource_hash: None,
            resource_name: "".to_string(),
        },
    );
    assert!(!res.main_failed());

    let gem = Program::current_with_id(system, GEM_ID);
    let res = gem.send(
        ADMIN,
        InitRMRK {
            name: "Gem".to_string(),
            symbol: "GEM".to_string(),
            resource_hash: None,
            resource_name: "".to_string(),
        },
    );
    assert!(!res.main_failed());

    // mint 5 birds
    for token_id in 0..5 {
        let res = kanaria.send(
            ADMIN,
            RMRKAction::MintToRootOwner {
                root_owner: ADMIN.into(),
                token_id: token_id.into(),
            },
        );
        let reply = RMRKReply::MintToRootOwner {
            root_owner: ADMIN.into(),
            token_id: token_id.into(),
        }
        .encode();
        assert!(res.contains(&(ADMIN, reply)));
    }

    // Mint 3 gems into each kanaria
    let mut gem_token_id = 0;
    for token_id in 0..5 {
        for _i in 0..3 {
            let res = gem.send(
                ADMIN,
                RMRKAction::MintToNft {
                    parent_id: KANARIA_ID.into(),
                    parent_token_id: token_id.into(),
                    token_id: gem_token_id.into(),
                },
            );
            let reply: Result<RMRKReply, RMRKError> = Ok(RMRKReply::MintToNft {
                parent_id: KANARIA_ID.into(),
                parent_token_id: token_id.into(),
                token_id: gem_token_id.into(),
            });
            assert!(res.contains(&(ADMIN, reply.encode())));

            let res = kanaria.send(
                ADMIN,
                RMRKAction::AcceptChild {
                    parent_token_id: token_id.into(),
                    child_contract_id: GEM_ID.into(),
                    child_token_id: gem_token_id.into(),
                },
            );

            let reply: Result<RMRKReply, RMRKError> = Ok(RMRKReply::AcceptedChild {
                child_contract_id: GEM_ID.into(),
                child_token_id: gem_token_id.into(),
                parent_token_id: token_id.into(),
            });
            assert!(res.contains(&(ADMIN, reply.encode())));

            gem_token_id += 1;
        }
    }
}

pub fn add_kanaria_assets(system: &System) {
    let kanaria = system.get_program(KANARIA_ID);
    let default_asset_id = 1;
    let composed_asset_id = 2;

    let res = kanaria.send(
        ADMIN,
        RMRKAction::AddEquippableAssetEntry {
            equippable_group_id: 0,
            catalog_address: None,
            metadata_uri: String::from("ipfs://default.png"),
            part_ids: vec![],
        },
    );

    let reply: Result<RMRKReply, RMRKError> = Ok(RMRKReply::EquippableAssetEntryAdded {
        id: default_asset_id,
        equippable_group_id: 0,
        catalog_address: None,
        metadata_uri: String::from("ipfs://default.png"),
        part_ids: vec![],
    });

    assert!(res.contains(&(ADMIN, reply.encode())));

    let res = kanaria.send(
        ADMIN,
        RMRKAction::AddEquippableAssetEntry {
            equippable_group_id: 0,
            catalog_address: Some(CATALOG_ID.into()),
            metadata_uri: String::from("ipfs://meta1.json"),
            part_ids: vec![1, 3, 5, 7, 9, 10, 11],
        },
    );

    let reply: Result<RMRKReply, RMRKError> = Ok(RMRKReply::EquippableAssetEntryAdded {
        id: composed_asset_id,
        equippable_group_id: 0,
        catalog_address: Some(CATALOG_ID.into()),
        metadata_uri: String::from("ipfs://meta1.json"),
        part_ids: vec![1, 3, 5, 7, 9, 10, 11],
    });

    assert!(res.contains(&(ADMIN, reply.encode())));

    let token_id: TokenId = 1.into();

    let res = kanaria.send(
        ADMIN,
        RMRKAction::AddAssetToToken {
            token_id,
            asset_id: default_asset_id,
            replaces_asset_with_id: 0,
        },
    );
    let reply: Result<RMRKReply, RMRKError> = Ok(RMRKReply::AssetAddedToToken {
        token_id,
        asset_id: default_asset_id,
        replaces_asset_with_id: 0,
    });
    assert!(res.contains(&(ADMIN, reply.encode())));

    let res = kanaria.send(
        ADMIN,
        RMRKAction::AddAssetToToken {
            token_id,
            asset_id: composed_asset_id,
            replaces_asset_with_id: 0,
        },
    );
    let reply: Result<RMRKReply, RMRKError> = Ok(RMRKReply::AssetAddedToToken {
        token_id,
        asset_id: composed_asset_id,
        replaces_asset_with_id: 0,
    });
    assert!(res.contains(&(ADMIN, reply.encode())));

    let res = kanaria.send(
        ADMIN,
        RMRKAction::AcceptAsset {
            token_id,
            asset_id: default_asset_id,
        },
    );
    let reply: Result<RMRKReply, RMRKError> = Ok(RMRKReply::AssetAccepted {
        token_id,
        asset_id: default_asset_id,
    });
    assert!(res.contains(&(ADMIN, reply.encode())));

    let res = kanaria.send(
        ADMIN,
        RMRKAction::AcceptAsset {
            token_id,
            asset_id: composed_asset_id,
        },
    );
    let reply: Result<RMRKReply, RMRKError> = Ok(RMRKReply::AssetAccepted {
        token_id,
        asset_id: composed_asset_id,
    });
    assert!(res.contains(&(ADMIN, reply.encode())));
}

pub fn add_gem_assets(system: &System) {
    let gem_version = 4;
    let gem = system.get_program(GEM_ID);

    // These refIds are used from the child's perspective, to group assets that can be equipped into a parent
    // With it, we avoid the need to do set it asset by asset
    let equippable_ref_id_left_gem = 1;
    let equippable_ref_id_mid_gem = 2;
    let equippable_ref_id_right_gem = 3;

    add_equippable_asset_entry(
        &gem,
        0,
        Some(CATALOG_ID.into()),
        String::from("ipfs://gems/typeA/full.svg"),
        vec![],
        1,
    );

    add_equippable_asset_entry(
        &gem,
        equippable_ref_id_left_gem,
        Some(CATALOG_ID.into()),
        String::from("ipfs://gems/typeA/left.svg"),
        vec![],
        2,
    );

    add_equippable_asset_entry(
        &gem,
        equippable_ref_id_mid_gem,
        Some(CATALOG_ID.into()),
        String::from("ipfs://gems/typeA/mid.svg"),
        vec![],
        3,
    );

    add_equippable_asset_entry(
        &gem,
        equippable_ref_id_right_gem,
        Some(CATALOG_ID.into()),
        String::from("ipfs://gems/typeA/right.svg"),
        vec![],
        4,
    );

    add_equippable_asset_entry(
        &gem,
        0,
        Some(CATALOG_ID.into()),
        String::from("ipfs://gems/typeB/full.svg"),
        vec![],
        5,
    );

    add_equippable_asset_entry(
        &gem,
        equippable_ref_id_left_gem,
        Some(CATALOG_ID.into()),
        String::from("ipfs://gems/typeB/left.svg"),
        vec![],
        6,
    );

    add_equippable_asset_entry(
        &gem,
        equippable_ref_id_mid_gem,
        Some(CATALOG_ID.into()),
        String::from("ipfs://gems/typeB/mid.svg"),
        vec![],
        7,
    );

    add_equippable_asset_entry(
        &gem,
        equippable_ref_id_right_gem,
        Some(CATALOG_ID.into()),
        String::from("ipfs://gems/typeB/right.svg"),
        vec![],
        8,
    );

}

fn add_equippable_asset_entry(
    program: &Program,
    equippable_group_id: u64,
    catalog_address: Option<ActorId>,
    metadata_uri: String,
    part_ids: Vec<PartId>,
    id: u64,
) {
    let result = program.send(
        ADMIN,
        RMRKAction::AddEquippableAssetEntry {
            equippable_group_id,
            catalog_address,
            metadata_uri: metadata_uri.clone(),
            part_ids: part_ids.clone(),
        },
    );

    let reply: Result<RMRKReply, RMRKError> = Ok(RMRKReply::EquippableAssetEntryAdded {
        id,
        equippable_group_id,
        catalog_address,
        metadata_uri,
        part_ids,
    });
    assert!(result.contains(&(ADMIN, reply.encode())));
}
// fn setup() {
//     let mut parts = BTreeMap::new();

//     let part_id_for_head_1 = 1;
//     let part_for_head_1 = Part::Fixed(FixedPart {
//         z: Some(1),
//         metadata_uri: String::from("ipfs://head1.png"),
//     });
//     parts.insert(part_id_for_head_1, part_for_head_1);

//     let part_id_for_head_2 = 2;
//     let part_for_head_2 = Part::Fixed(FixedPart {
//         z: Some(1),
//         metadata_uri: String::from("ipfs://head2.png"),
//     });
//     parts.insert(part_id_for_head_2, part_for_head_2);

//     let part_id_for_head_3 = 3;
//     let part_for_head_3 = Part::Fixed(FixedPart {
//         z: Some(1),
//         metadata_uri: String::from("ipfs://head3.png"),
//     });
//     parts.insert(part_id_for_head_3, part_for_head_3);

//     let part_id_for_body_1 = 4;
//     let part_for_body_1 = Part::Fixed(FixedPart {
//         z: Some(1),
//         metadata_uri: String::from("ipfs://body1.png"),
//     });
//     parts.insert(part_id_for_body_1, part_for_body_1);

//     let part_id_for_body_2 = 5;
//     let part_for_body_2 = Part::Fixed(FixedPart {
//         z: Some(1),
//         metadata_uri: String::from("ipfs://body2.png"),
//     });
//     parts.insert(part_id_for_body_2, part_for_body_2);

//     let part_id_for_hair_1 = 6;
//     let part_for_hair_1 = Part::Fixed(FixedPart {
//         z: Some(2),
//         metadata_uri: String::from("ipfs://hair1.png"),
//     });
//     parts.insert(part_id_for_hair_1, part_for_hair_1);

//     let part_id_for_hair_2 = 7;
//     let part_for_hair_2 = Part::Fixed(FixedPart {
//         z: Some(2),
//         metadata_uri: String::from("ipfs://hair2.png"),
//     });
//     parts.insert(part_id_for_hair_2, part_for_hair_2);

//     let part_id_for_hair_3 = 8;
//     let part_for_hair_3 = Part::Fixed(FixedPart {
//         z: Some(2),
//         metadata_uri: String::from("ipfs://hair3.png"),
//     });
//     parts.insert(part_id_for_hair_3, part_for_hair_3);

//     let part_id_for_mask_catalog_1 = 9;
//     let part_for_mask_catalog_1 = Part::Fixed(FixedPart {
//         z: Some(3),
//         metadata_uri: String::from("ipfs://maskCatalog1.png"),
//     });
//     parts.insert(part_id_for_mask_catalog_1, part_for_mask_catalog_1);

//     let part_id_for_mask_catalog_2 = 10;
//     let part_for_mask_catalog_2 = Part::Fixed(FixedPart {
//         z: Some(3),
//         metadata_uri: String::from("ipfs://maskCatalog2.png"),
//     });
//     parts.insert(part_id_for_mask_catalog_2, part_for_mask_catalog_2);

//     let part_id_for_mask_catalog_3 = 11;
//     let part_for_mask_catalog_3 = Part::Fixed(FixedPart {
//         z: Some(3),
//         metadata_uri: String::from("ipfs://maskCatalog3.png"),
//     });
//     parts.insert(part_id_for_mask_catalog_3, part_for_mask_catalog_3);

//     let part_id_for_ears_1 = 12;
//     let part_for_mask_ears_1 = Part::Fixed(FixedPart {
//         z: Some(4),
//         metadata_uri: String::from("ipfs://ears1.png"),
//     });
//     parts.insert(part_id_for_mask_ears_1, part_for_mask_ears_1);

//     let part_id_for_ears_2 = 13;
//     let part_for_mask_ears_2 = Part::Fixed(FixedPart {
//         z: Some(4),
//         metadata_uri: String::from("ipfs://ears2.png"),
//     });
//     parts.insert(part_id_for_mask_ears_2, part_for_mask_ears_2);

//     let part_id_for_horns_1 = 14;
//     let part_for_horns_1 = Part::Fixed(FixedPart {
//         z: Some(5),
//         metadata_uri: String::from("ipfs://horns1.png"),
//     });
//     parts.insert(part_id_for_horns_1, part_for_horns_1);

//     let part_id_for_horns_2 = 15;
//     let part_for_horns_2 = Part::Fixed(FixedPart {
//         z: Some(5),
//         metadata_uri: String::from("ipfs://horns2.png"),
//     });
//     parts.insert(part_id_for_horns_2, part_for_horns_2);

//     let part_id_for_horns_3 = 16;
//     let part_for_horns_3 = Part::Fixed(FixedPart {
//         z: Some(5),
//         metadata_uri: String::from("ipfs://horns3.png"),
//     });
//     parts.insert(part_id_for_horns_3, part_for_horns_3);

//     let part_id_for_mask_catalog_equipped_1 = 17;
//     let part_for_mask_catalog_equipped_1 = Part::Fixed(FixedPart {
//         z: Some(3),
//         metadata_uri: String::from("ipfs://maskCatalogEquipped1.png"),
//     });
//     parts.insert(part_id_for_mask_catalog_equipped_1,  part_for_mask_catalog_equipped_1);

//     let part_id_for_mask_catalog_equipped_2 = 18;
//     let part_for_mask_catalog_equipped_2 = Part::Fixed(FixedPart {
//         z: Some(3),
//         metadata_uri: String::from("ipfs://maskCatalogEquipped2.png"),
//     });
//     parts.insert(part_id_for_mask_catalog_equipped_2,  part_for_mask_catalog_equipped_2);

//     let part_id_for_mask_catalog_equipped_3 = 19;
//     let part_for_mask_catalog_equipped_3 = Part::Fixed(FixedPart {
//         z: Some(3),
//         metadata_uri: String::from("ipfs://maskCatalogEquipped3.png"),
//     });
//     parts.insert(part_id_for_mask_catalog_equipped_3,  part_for_mask_catalog_equipped_3);

//     let part_id_for_ears_equipped_1 = 20;
//     let part_for_ears_equipped_1 = Part::Fixed(FixedPart {
//         z: Some(5),
//         metadata_uri: String::from("ipfs://maskEarsEquipped1.png"),
//     });
//     parts.insert(part_id_for_ears_equipped_1,  part_for_ears_equipped_1);

//     let part_id_for_ears_equipped_2 = 21;
//     let part_for_ears_equipped_2 = Part::Fixed(FixedPart {
//         z: Some(5),
//         metadata_uri: String::from("ipfs://maskEarsEquipped2.png"),
//     });
//     parts.insert(part_id_for_ears_equipped_2,  part_for_ears_equipped_2);

//     let part_id_for_horns_equipped_1 = 22;
//     let part_for_horns_equipped_1 = Part::Fixed(FixedPart {
//         z: Some(5),
//         metadata_uri: String::from("ipfs://maskHornsEquipped1.png"),
//     });
//     parts.insert(part_id_for_horns_equipped_1,  part_for_horns_equipped_1);

//     let part_id_for_horns_equipped_2 = 23;
//     let part_for_horns_equipped_2 = Part::Fixed(FixedPart {
//         z: Some(5),
//         metadata_uri: String::from("ipfs://maskHornsEquipped2.png"),
//     });
//     parts.insert(part_id_for_horns_equipped_2,  part_for_horns_equipped_2);

//     let part_id_for_horns_equipped_3 = 24;
//     let part_for_horns_equipped_3 = Part::Fixed(FixedPart {
//         z: Some(5),
//         metadata_uri: String::from("ipfs://maskHornsEquipped3.png"),
//     });
//     parts.insert(part_id_for_horns_equipped_3,  part_for_horns_equipped_3);

//     let part_id_for_mask = 25;
//     let part_for_mask = Part::Slot(SlotPart { equippable: vec![MASK_EQUIP_ADDRESS.into()], z: Some(2), metadata_uri: String::from("") });
//     parts.insert(part_id_for_mask,  part_for_mask);

//     let unique_neons = 10;
//     let unique_masks = 4;

// }

// fn mint_neons() {}

// fn mint_masks() {}
