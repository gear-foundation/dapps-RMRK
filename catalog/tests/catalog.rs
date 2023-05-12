use catalog_io::*;
use gstd::{prelude::*, BTreeMap};
use gtest::{Program, System};
pub const ADMIN: u64 = 10;

pub fn init_catalog(sys: &System, admin: u64) {
    sys.init_logger();
    let catalog = Program::current(sys);
    let res = catalog.send(
        admin,
        InitCatalog {
            catalog_type: "svg".to_string(),
            symbol: "BaseSymbol".to_string(),
        },
    );
    assert!(!res.main_failed());
}

#[test]
fn add_parts() {
    let system = System::new();
    init_catalog(&system, ADMIN);
    let catalog = system.get_program(1);

    // Add fixed part
    let fixed_part_data = Part::Fixed(FixedPart {
        z: Some(0),
        metadata_uri: String::from("src"),
    });
    let part_id = 1;

    let added_part = BTreeMap::from([(part_id, fixed_part_data.clone())]);

    let result = catalog.send(ADMIN, CatalogAction::AddParts(added_part.clone()));
    let expected_reply: Result<CatalogReply, CatalogError> =
        Ok(CatalogReply::PartsAdded(added_part));
    assert!(result.contains(&(ADMIN, expected_reply.encode())));

    // check that fixed part is in the state
    let state: CatalogState = catalog.read_state().expect("Failed to decode the state");
    let fixed_part_in_state = state
        .parts
        .iter()
        .any(|part| part == &(part_id, fixed_part_data.clone()));
    assert!(fixed_part_in_state);

    // Add slot part
    let slot_part_data = Part::Slot(SlotPart {
        equippable: vec![],
        z: Some(0),
        metadata_uri: String::from("src"),
    });
    let part_id = 2;

    let added_part = BTreeMap::from([(part_id, slot_part_data.clone())]);
    let result = catalog.send(ADMIN, CatalogAction::AddParts(added_part.clone()));
    let expected_reply: Result<CatalogReply, CatalogError> =
        Ok(CatalogReply::PartsAdded(added_part));
    assert!(result.contains(&(ADMIN, expected_reply.encode())));

    // Check that slot part is in the state
    let state: CatalogState = catalog.read_state().expect("Failed to decode the state");

    let slot_part_in_state = state
        .parts
        .iter()
        .any(|part| part == &(part_id, slot_part_data.clone()));
    assert!(slot_part_in_state);

    // Add part list
    let slot_part_id = 10;
    let fixed_part_id_1 = 20;
    let fixed_part_id_2 = 21;

    let fixed_part_data_1 = Part::Fixed(FixedPart {
        z: Some(1),
        metadata_uri: String::from("src1"),
    });
    let fixed_part_data_2 = Part::Fixed(FixedPart {
        z: Some(1),
        metadata_uri: String::from("src2"),
    });
    let slot_part_data = Part::Slot(SlotPart {
        equippable: vec![],
        z: Some(2),
        metadata_uri: String::from("src3"),
    });
    let mut parts = BTreeMap::new();
    parts.insert(slot_part_id, slot_part_data.clone());
    parts.insert(fixed_part_id_1, fixed_part_data_1.clone());
    parts.insert(fixed_part_id_2, fixed_part_data_2.clone());

    let result = catalog.send(ADMIN, CatalogAction::AddParts(parts.clone()));
    let expected_reply: Result<CatalogReply, CatalogError> = Ok(CatalogReply::PartsAdded(parts));
    assert!(result.contains(&(ADMIN, expected_reply.encode())));

    let state: CatalogState = catalog.read_state().expect("Failed to decode the state");

    // check that fixed part_1 is in the state
    let fixed_part_1_in_state = state
        .parts
        .iter()
        .any(|part| part == &(fixed_part_id_1, fixed_part_data_1.clone()));
    assert!(fixed_part_1_in_state);

    // check that fixed part_2 is in the state
    let fixed_part_2_in_state = state
        .parts
        .iter()
        .any(|part| part == &(fixed_part_id_2, fixed_part_data_2.clone()));
    assert!(fixed_part_2_in_state);

    // check that slot part_1 is in the state
    let slot_part_in_state = state
        .parts
        .iter()
        .any(|part| part == &(slot_part_id, slot_part_data.clone()));
    assert!(slot_part_in_state);

    // Remove parts
    let removed_parts = vec![fixed_part_id_1, slot_part_id];
    let result = catalog.send(ADMIN, CatalogAction::RemoveParts(removed_parts.clone()));
    let expected_reply: Result<CatalogReply, CatalogError> =
        Ok(CatalogReply::PartsRemoved(removed_parts));
    assert!(result.contains(&(ADMIN, expected_reply.encode())));

    // check that fixed part_1 is NOT in the state
    let state: CatalogState = catalog.read_state().expect("Failed to decode the state");
    let fixed_part_1_in_state = state
        .parts
        .iter()
        .any(|part| part == &(fixed_part_id_1, fixed_part_data_1.clone()));
    assert!(!fixed_part_1_in_state);

    // check that slot part_1 is NOT in the state
    let slot_part_in_state = state
        .parts
        .iter()
        .any(|part| part == &(slot_part_id, slot_part_data.clone()));
    assert!(!slot_part_in_state);

    // Zero length array of parts
    let result = catalog.send(ADMIN, CatalogAction::RemoveParts(vec![]));
    let expected_reply: Result<CatalogReply, CatalogError> = Err(CatalogError::ZeroLengthPassed);
    assert!(result.contains(&(ADMIN, expected_reply.encode())));

    // Cannot remove non-existing part
    let result = catalog.send(ADMIN, CatalogAction::RemoveParts(vec![fixed_part_id_1]));
    let expected_reply: Result<CatalogReply, CatalogError> = Err(CatalogError::PartDoesNotExist);
    assert!(result.contains(&(ADMIN, expected_reply.encode())));
}

#[test]
fn add_parts_error_cases() {
    let system = System::new();
    init_catalog(&system, ADMIN);
    let catalog = system.get_program(1);

    let fixed_part_data = Part::Fixed(FixedPart {
        z: Some(0),
        metadata_uri: String::from("src"),
    });
    let part_id = 0;

    let added_part = BTreeMap::from([(part_id, fixed_part_data.clone())]);

    // Cannot add part with zero id
    let result = catalog.send(ADMIN, CatalogAction::AddParts(added_part.clone()));
    let expected_reply: Result<CatalogReply, CatalogError> = Err(CatalogError::PartIdCantBeZero);
    assert!(result.contains(&(ADMIN, expected_reply.encode())));

    // check that fixed part is in the state
    let state: CatalogState = catalog.read_state().expect("Failed to decode the state");
    assert_eq!(state.parts, vec![]);

    // Add part
    let part_id = 1;

    let added_part = BTreeMap::from([(part_id, fixed_part_data.clone())]);

    let result = catalog.send(ADMIN, CatalogAction::AddParts(added_part.clone()));
    let expected_reply: Result<CatalogReply, CatalogError> =
        Ok(CatalogReply::PartsAdded(added_part));
    assert!(result.contains(&(ADMIN, expected_reply.encode())));

    // Cannot add part with already existing id
    let added_part = BTreeMap::from([(part_id, fixed_part_data.clone())]);
    let result = catalog.send(ADMIN, CatalogAction::AddParts(added_part.clone()));
    let expected_reply: Result<CatalogReply, CatalogError> = Err(CatalogError::PartAlreadyExists);
    assert!(result.contains(&(ADMIN, expected_reply.encode())));

    // Zero length BTreeMap
    let result = catalog.send(ADMIN, CatalogAction::AddParts(BTreeMap::new()));
    let expected_reply: Result<CatalogReply, CatalogError> = Err(CatalogError::ZeroLengthPassed);
    assert!(result.contains(&(ADMIN, expected_reply.encode())));
}

#[test]
fn equippable() {
    let system = System::new();
    init_catalog(&system, ADMIN);
    let catalog = system.get_program(1);

    // Add fixed part
    let fixed_part_id = 1;
    let fixed_part_data = Part::Fixed(FixedPart {
        z: Some(0),
        metadata_uri: String::from("src"),
    });

    let slot_part_id_1 = 2;
    let slot_part_data_1 = Part::Slot(SlotPart {
        equippable: vec![100.into()],
        z: Some(0),
        metadata_uri: String::from("src"),
    });

    let slot_part_id_2 = 3;
    let slot_part_data_2 = Part::Slot(SlotPart {
        equippable: vec![],
        z: Some(0),
        metadata_uri: String::from("src"),
    });

    let mut parts = BTreeMap::new();
    parts.insert(fixed_part_id, fixed_part_data);
    parts.insert(slot_part_id_1, slot_part_data_1);
    parts.insert(slot_part_id_2, slot_part_data_2);

    let result = catalog.send(ADMIN, CatalogAction::AddParts(parts.clone()));
    let expected_reply: Result<CatalogReply, CatalogError> = Ok(CatalogReply::PartsAdded(parts));
    assert!(result.contains(&(ADMIN, expected_reply.encode())));

    // Is not equippable if address was not added
    let result = catalog.send(
        ADMIN,
        CatalogAction::CheckEquippable {
            part_id: slot_part_id_2,
            collection_id: 100.into(),
        },
    );
    let expected_reply: Result<CatalogReply, CatalogError> = Ok(CatalogReply::NotInEquippableList);
    assert!(result.contains(&(ADMIN, expected_reply.encode())));

    // Is equippable if added in the part definition
    let result = catalog.send(
        ADMIN,
        CatalogAction::CheckEquippable {
            part_id: slot_part_id_1,
            collection_id: 100.into(),
        },
    );
    let expected_reply: Result<CatalogReply, CatalogError> = Ok(CatalogReply::InEquippableList);
    assert!(result.contains(&(ADMIN, expected_reply.encode())));

    // Is equippable if added afterward
    let result = catalog.send(
        ADMIN,
        CatalogAction::AddEquippableAddresses {
            part_id: slot_part_id_2,
            collection_ids: vec![100.into()],
        },
    );
    let expected_reply: Result<CatalogReply, CatalogError> = Ok(CatalogReply::EquippablesAdded {
        part_id: slot_part_id_2,
        collection_ids: vec![100.into()],
    });
    assert!(result.contains(&(ADMIN, expected_reply.encode())));
    let result = catalog.send(
        ADMIN,
        CatalogAction::CheckEquippable {
            part_id: slot_part_id_2,
            collection_id: 100.into(),
        },
    );
    let expected_reply: Result<CatalogReply, CatalogError> = Ok(CatalogReply::InEquippableList);
    assert!(result.contains(&(ADMIN, expected_reply.encode())));

    // Is equippable if set to all
    let result = catalog.send(
        ADMIN,
        CatalogAction::SetEquippableToAll {
            part_id: slot_part_id_1,
        },
    );
    let expected_reply: Result<CatalogReply, CatalogError> = Ok(CatalogReply::EquippableToAllSet);
    assert!(result.contains(&(ADMIN, expected_reply.encode())));

    let result = catalog.send(
        ADMIN,
        CatalogAction::CheckEquippable {
            part_id: slot_part_id_1,
            collection_id: 200.into(),
        },
    );
    let expected_reply: Result<CatalogReply, CatalogError> = Ok(CatalogReply::InEquippableList);
    assert!(result.contains(&(ADMIN, expected_reply.encode())));

    // Can reset equippable addresses
    // Reset the slot that is equippable to all
    let result = catalog.send(
        ADMIN,
        CatalogAction::ResetEquippableAddress {
            part_id: slot_part_id_1,
        },
    );
    let expected_reply: Result<CatalogReply, CatalogError> =
        Ok(CatalogReply::EqippableAddressesReset);
    assert!(result.contains(&(ADMIN, expected_reply.encode())));

    let result = catalog.send(
        ADMIN,
        CatalogAction::CheckEquippable {
            part_id: slot_part_id_1,
            collection_id: 200.into(),
        },
    );
    let expected_reply: Result<CatalogReply, CatalogError> = Ok(CatalogReply::NotInEquippableList);
    assert!(result.contains(&(ADMIN, expected_reply.encode())));

    // Reset the slot that is equippable to indixated addresses
    let result = catalog.send(
        ADMIN,
        CatalogAction::ResetEquippableAddress {
            part_id: slot_part_id_2,
        },
    );
    let expected_reply: Result<CatalogReply, CatalogError> =
        Ok(CatalogReply::EqippableAddressesReset);
    assert!(result.contains(&(ADMIN, expected_reply.encode())));

    let result = catalog.send(
        ADMIN,
        CatalogAction::CheckEquippable {
            part_id: slot_part_id_2,
            collection_id: 100.into(),
        },
    );
    let expected_reply: Result<CatalogReply, CatalogError> = Ok(CatalogReply::NotInEquippableList);
    assert!(result.contains(&(ADMIN, expected_reply.encode())));

    // Cannot add equippable addresses for non existing part
    let result = catalog.send(
        ADMIN,
        CatalogAction::AddEquippableAddresses {
            part_id: 100,
            collection_ids: vec![100.into()],
        },
    );
    let expected_reply: Result<CatalogReply, CatalogError> = Err(CatalogError::PartDoesNotExist);
    assert!(result.contains(&(ADMIN, expected_reply.encode())));

    // Cannot add empty list of equippable addresses
    let result = catalog.send(
        ADMIN,
        CatalogAction::AddEquippableAddresses {
            part_id: slot_part_id_1,
            collection_ids: vec![],
        },
    );
    let expected_reply: Result<CatalogReply, CatalogError> = Err(CatalogError::ZeroLengthPassed);
    assert!(result.contains(&(ADMIN, expected_reply.encode())));

    // Cannot add equippable addresses to non slot part
    let result = catalog.send(
        ADMIN,
        CatalogAction::AddEquippableAddresses {
            part_id: fixed_part_id,
            collection_ids: vec![200.into()],
        },
    );
    let expected_reply: Result<CatalogReply, CatalogError> = Err(CatalogError::WrongPartFormat);
    assert!(result.contains(&(ADMIN, expected_reply.encode())));

    // Cannot reset equippable for non existing part
    let result = catalog.send(
        ADMIN,
        CatalogAction::ResetEquippableAddress { part_id: 1000 },
    );
    let expected_reply: Result<CatalogReply, CatalogError> = Err(CatalogError::PartDoesNotExist);
    assert!(result.contains(&(ADMIN, expected_reply.encode())));

    // Cannot reset equippable for fixed part
    let result = catalog.send(
        ADMIN,
        CatalogAction::ResetEquippableAddress {
            part_id: fixed_part_id,
        },
    );
    let expected_reply: Result<CatalogReply, CatalogError> = Err(CatalogError::WrongPartFormat);
    assert!(result.contains(&(ADMIN, expected_reply.encode())));
}
// fn get_parts() -> BTreeMap<PartId, Part> {
//     let mut parts: BTreeMap<PartId, Part> = BTreeMap::new();
//     let fixed_part_id = 100;
//     let fixed_part = Part::Fixed(FixedPart {
//         z: Some(3),
//         src: "fixed_part_src".to_string(),
//     });
//     parts.insert(fixed_part_id, fixed_part);

//     let slot_part_id = 102;
//     let slot_part = Part::Slot(SlotPart {
//         equippable: vec![],
//         z: Some(3),
//         src: "slot_part_src".to_string(),
//     });
//     parts.insert(slot_part_id, slot_part);

//     let slot_part_id = 103;
//     let slot_part = Part::Slot(SlotPart {
//         equippable: vec![],
//         z: Some(2),
//         src: "slot_part_src".to_string(),
//     });

//     parts.insert(slot_part_id, slot_part);
//     parts
// }
// fn add_parts(catalog: &Program, issuer: u64, parts: BTreeMap<PartId, Part>) {
//     let res = catalog.send(issuer, CatalogAction::AddParts(parts.clone()));
//     assert!(res.contains(&(issuer, CatalogReply::PartsAdded(parts).encode())));
// }

// fn check_part(catalog: &Program, part_id: PartId) -> RunResult {
//     catalog.send(ISSUER, CatalogAction::CheckPart(part_id))
// }

// fn remove_parts(catalog: &Program, issuer: u64, parts: Vec<PartId>) -> RunResult {
//     catalog.send(issuer, CatalogAction::RemoveParts(parts))
// }

// fn add_equippable(
//     catalog: &Program,
//     issuer: u64,
//     part_id: PartId,
//     collection_id: u64,
// ) -> RunResult {
//     catalog.send(
//         issuer,
//         CatalogAction::AddEquippable {
//             part_id,
//             collection_id: collection_id.into(),
//         },
//     )
// }

// fn remove_equippable(
//     catalog: &Program,
//     admin: u64,
//     part_id: PartId,
//     collection_id: u64,
// ) -> RunResult {
//     catalog.send(
//         admin,
//         CatalogAction::RemoveEquippable {
//             part_id,
//             collection_id: collection_id.into(),
//         },
//     )
// }

// fn check_equippable(
//     base: &Program,
//     part_id: PartId,
//     collection_id: u64,
// ) -> RunResult {
//     base.send(
//         ISSUER,
//         CatalogAction::CheckEquippable {
//             part_id,
//             collection_id: collection_id.into(),
//         },
//     )
// }

// #[test]
// fn add_parts_test() {
//     let sys = System::new();
//     init_catalog(&sys, ISSUER);
//     let catalog = sys.get_program(1);
//     let parts = get_parts();
//     add_parts(&base, ISSUER, parts);

//     l
//     // // meta state (parts)
//     // let parts_reply: BaseStateReply = base
//     //     .meta_state(BaseState::Parts)
//     //     .expect("Meta_state failed");
//     // let expected_reply: Vec<Part> = parts.values().cloned().collect();
//     // assert_eq!(parts_reply, BaseStateReply::Parts(expected_reply));

//     // // meta state (part)
//     // for (part_id, part) in parts {
//     //     let part_reply: BaseStateReply = base
//     //         .meta_state(BaseState::Part(part_id))
//     //         .expect("Meta_state failed");
//     //     assert_eq!(part_reply, BaseStateReply::Part(Some(part.clone())));
//     //     // message: check parts
//     //     let res = check_part(&base, part_id);
//     //     assert!(res.contains(&(ISSUER, BaseEvent::Part(part).encode())));
//     // }

//     // // meta state for non-existing part
//     // let part_reply: BaseStateReply = base
//     //     .meta_state(BaseState::Part(1000))
//     //     .expect("Meta_state failed");
//     // assert_eq!(part_reply, BaseStateReply::Part(None));
// }

// #[test]
// #[should_panic]
// fn add_parts_wrong_issuer() {
//     let sys = System::new();
//     init_base(&sys, 1000);
//     let base = sys.get_program(1);
//     add_parts(&base, ISSUER, BTreeMap::new());
// }

// #[test]
// fn remove_parts_test() {
//     let sys = System::new();
//     init_base(&sys, ISSUER);
//     let base = sys.get_program(1);
//     let mut parts = get_parts();
//     add_parts(&base, ISSUER, parts.clone());

//     let removed_parts: Vec<PartId> = vec![100, 102];
//     let res = remove_parts(&base, ISSUER, removed_parts.clone());
//     assert!(res.contains(&(ISSUER, CatalogReply::PartsRemoved(removed_parts).encode())));
//     parts.remove(&100);
//     parts.remove(&102);
//     // // meta state (parts)
//     // let parts_reply: BaseStateReply = base
//     //     .meta_state(BaseState::Parts)
//     //     .expect("Meta_state failed");
//     // assert_eq!(
//     //     parts_reply,
//     //     BaseStateReply::Parts(vec![parts[&103].clone()])
//     // );

//     // // check that removed parts are None
//     // let part_reply: BaseStateReply = base
//     //     .meta_state(BaseState::Part(100))
//     //     .expect("Meta_state failed");
//     // assert_eq!(part_reply, BaseStateReply::Part(None));

//     // let part_reply: BaseStateReply = base
//     //     .meta_state(BaseState::Part(102))
//     //     .expect("Meta_state failed");
//     // assert_eq!(part_reply, BaseStateReply::Part(None));
// }

// #[test]
// fn remove_parts_failures() {
//     let sys = System::new();
//     init_base(&sys, ISSUER);
//     let base = sys.get_program(1);
//     let parts = get_parts();
//     add_parts(&base, ISSUER, parts);

//     let mut removed_parts: Vec<PartId> = vec![100, 102];

//     // wrong issuer
//     assert!(remove_parts(&base, 1000, removed_parts.clone()).main_failed());

//     removed_parts[0] = 500;
//     // Part with indicated ID does not exist
//     assert!(remove_parts(&base, ISSUER, removed_parts).main_failed());
// }

// #[test]
// fn add_remove_equippable_test() {
//     let sys = System::new();
//     init_base(&sys, ISSUER);
//     let base = sys.get_program(1);
//     let mut parts = get_parts();
//     add_parts(&base, ISSUER, parts.clone());

//     let part_id: PartId = 102;
//     let collection_id: u64 = 300;
//     let token_id: u64 = 250;

//     let res = add_equippable(&base, ISSUER, part_id, collection_id, token_id);
//     assert!(res.contains(&(
//         ISSUER,
//         CatalogReply::EquippableAdded {
//             part_id,
//             collection_id: collection_id.into(),
//         }
//         .encode()
//     )));

//     // check that part contains equippable
//     let part = parts
//         .get_mut(&part_id)
//         .expect("Part with that id does not exist");
//     if let Part::Slot(SlotPart {
//         equippable: EquippableList::Custom(collection_and_token),
//         ..
//     }) = part
//     {
//         collection_and_token.insert((collection_id.into(), token_id.into()));
//     }
//     // let part_reply: BaseStateReply = base
//     //     .meta_state(BaseState::Part(102))
//     //     .expect("Meta_state failed");
//     // assert_eq!(part_reply, BaseStateReply::Part(Some(part.clone())));

//     // // check if token from the collection in the equippable list
//     // let is_equippable_reply: BaseStateReply = base
//     //     .meta_state(BaseState::IsEquippable {
//     //         part_id,
//     //         collection_id: collection_id.into(),
//     //         token_id: token_id.into(),
//     //     })
//     //     .expect("Meta_state failed");
//     // assert_eq!(is_equippable_reply, BaseStateReply::IsEquippable(true));

//     // check if token from the collection in the equippable list through the message
//     let res = check_equippable(&base, part_id, collection_id, token_id);
//     assert!(res.contains(&(ISSUER, CatalogReply::InEquippableList.encode())));

//     // // check that `is_equippable` is true if equippableList = EquippableList::All
//     // let is_equippable_reply: BaseStateReply = base
//     //     .meta_state(BaseState::IsEquippable {
//     //         part_id: 103,
//     //         collection_id: collection_id.into(),
//     //         token_id: token_id.into(),
//     //     })
//     //     .expect("Meta_state failed");
//     // assert_eq!(is_equippable_reply, BaseStateReply::IsEquippable(true));

//     let res = remove_equippable(&base, ISSUER, part_id, collection_id, token_id);
//     assert!(res.contains(&(
//         ISSUER,
//         CatalogReply::EquippableRemoved {
//             part_id,
//             collection_id: collection_id.into(),
//         }
//         .encode()
//     )));

//     // // check if token from the collection is not in the equippable list
//     // let is_equippable_reply: BaseStateReply = base
//     //     .meta_state(BaseState::IsEquippable {
//     //         part_id,
//     //         collection_id: collection_id.into(),
//     //         token_id: token_id.into(),
//     //     })
//     //     .expect("Meta_state failed");
//     // assert_eq!(is_equippable_reply, BaseStateReply::IsEquippable(false));
// }

// #[test]
// fn add_remove_equippable_failures() {
//     let sys = System::new();
//     init_base(&sys, ISSUER);
//     let base = sys.get_program(1);
//     let parts = get_parts();
//     add_parts(&base, ISSUER, parts);

//     let part_id: PartId = 102;
//     let collection_id: u64 = 300;
//     let token_id: u64 = 250;

//     // must fail since part does not exist
//     assert!(add_equippable(&base, ISSUER, 500, collection_id).main_failed());

//     // must fail since wrong issuer
//     assert!(add_equippable(&base, 500, part_id, collection_id).main_failed());

//     // must fail since equippable is added to FixedPart
//     assert!(add_equippable(&base, ISSUER, 100, collection_id).main_failed());

//     // must fail since part does not exist
//     assert!(remove_equippable(&base, ISSUER, 500, collection_id).main_failed());

//     // must fail since wrong issuer
//     assert!(remove_equippable(&base, 500, part_id, collection_id).main_failed());

//     // must fail since equippable is removed from FixedPart
//     assert!(remove_equippable(&base, ISSUER, 100, collection_id).main_failed());
// }

// #[test]
// fn add_check_failures() {
//     let sys = System::new();
//     init_base(&sys, ISSUER);
//     let base = sys.get_program(1);
//     let parts = get_parts();
//     add_parts(&base, ISSUER, parts);
//     // must fail since part does not exist
//     assert!(check_part(&base, 300).main_failed());

//     // must fail since token is not in equippable list
//     assert!(check_equippable(&base, 102, 100, 100).main_failed());
// }
