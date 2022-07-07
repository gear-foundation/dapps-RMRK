use crate::utils::*;
use gstd::BTreeSet;
use gtest::System;
use resource_io::Resource;
use types::primitives::ResourceId;

#[test]
fn overwrite_resource() {
    let sys = System::new();
    before_multiresource_test(&sys);
    let rmrk = sys.get_program(1);

    let resource_id: ResourceId = 1;
    let new_resource_id: ResourceId = 2;
    let resource = Resource::Basic(Default::default());
    let new_resource = Resource::Basic(Default::default());
    // add resource entry to storage contract
    add_resource_entry(&rmrk, USERS[0], resource_id, resource);
    // add overwrite resource to storage contract
    add_resource_entry(&rmrk, USERS[0], new_resource_id, new_resource);

    let token_id: u64 = 10;

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
    before_multiresource_test(&sys);
    let rmrk = sys.get_program(1);

    let resource_id_1: u8 = 1;
    let resource_id_2: u8 = 2;
    let resource_id_3: u8 = 3;

    let resource_1 = Resource::Basic(Default::default());
    let resource_2 = Resource::Basic(Default::default());
    let resource_3 = Resource::Basic(Default::default());
    // add resources entry to storage contract
    add_resource_entry(&rmrk, USERS[0], resource_id_1, resource_1);
    add_resource_entry(&rmrk, USERS[0], resource_id_2, resource_2);
    add_resource_entry(&rmrk, USERS[0], resource_id_3, resource_3);

    let token_id: u64 = 10;

    // must fail since no resource to overwrite
    assert!(add_resource(&rmrk, USERS[0], token_id, resource_id_2, resource_id_1).main_failed());

    // add and accept resource_id
    assert!(!add_resource(&rmrk, USERS[0], token_id, resource_id_1, 0).main_failed());
    assert!(!accept_resource(&rmrk, USERS[0], token_id, resource_id_1).main_failed());

    // must fail since Proposed overwritten resource must exist on token
    assert!(add_resource(&rmrk, USERS[0], token_id, resource_id_3, resource_id_2).main_failed());
}
