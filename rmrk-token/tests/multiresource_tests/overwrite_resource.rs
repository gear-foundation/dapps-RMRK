use crate::multiresource_tests::utils::*;
use gtest::System;
use gstd::BTreeSet;
use resource_io::{Resource, ResourceId};

#[test]
fn overwrite_resource() {
    let sys = System::new();
    before_test(&sys);
    let rmrk = sys.get_program(1);

    let resource = Resource {
        id: 1,
        ..Default::default()
    };
    let new_resource = Resource {
        id: 2,
        ..Default::default()
    };
    // add resource entry to storage contract
    add_resource_entry(&rmrk, USERS[0], resource);
    // add overwrite resource to storage contract
    add_resource_entry(&rmrk, USERS[0], new_resource);

    let token_id: u64 = 10;
    let resource_id: u8 = 1;
    let new_resource_id: u8 = 2;

    // add and accept resource_id
    assert!(!add_resource(&rmrk, USERS[0], token_id, resource_id, 0).main_failed());
    assert!(!accept_resource(&rmrk, USERS[0], token_id, resource_id).main_failed());

    // add resource to overwrite
    assert!(!add_resource(&rmrk, USERS[0], token_id, new_resource_id, resource_id).main_failed());
    // check pending resources
    let mut resources: BTreeSet<ResourceId> = BTreeSet::new();
    resources.insert(new_resource_id);
    check_pending_resources(&rmrk, token_id, resources.clone());

    // accept new resource instead of previous one
    assert!(!accept_resource(&rmrk, USERS[0], token_id, new_resource_id).main_failed());
    // check active resources
    check_active_resources(&rmrk, token_id, resources);
}

#[test]
fn overwrite_resource_failures() {
    let sys = System::new();
    before_test(&sys);
    let rmrk = sys.get_program(1);

    let resource_1 = Resource {
        id: 1,
        ..Default::default()
    };
    let resource_2 = Resource {
        id: 2,
        ..Default::default()
    };
    let resource_3 = Resource {
        id: 3,
        ..Default::default()
    };
    // add resources entry to storage contract
    add_resource_entry(&rmrk, USERS[0], resource_1);
    add_resource_entry(&rmrk, USERS[0], resource_2);
    add_resource_entry(&rmrk, USERS[0], resource_3);

    let token_id: u64 = 10;
    let resource_id_1: u8 = 1;
    let resource_id_2: u8 = 2;
    let resource_id_3: u8 = 3;

    // must fail since no resource to overwrite
    assert!(add_resource(&rmrk, USERS[0], token_id, resource_id_2, resource_id_1).main_failed());

    // add and accept resource_id
    assert!(!add_resource(&rmrk, USERS[0], token_id, resource_id_1, 0).main_failed());
    assert!(!accept_resource(&rmrk, USERS[0], token_id, resource_id_1).main_failed());

    // must fail since Proposed overwritten resource must exist on token
    assert!(add_resource(&rmrk, USERS[0], token_id, resource_id_3, resource_id_2).main_failed());
}
