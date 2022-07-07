use crate::utils::*;
use codec::Encode;
use gstd::BTreeSet;
use gtest::System;
use resource_io::Resource;
use rmrk_io::*;
use types::primitives::ResourceId;

// adds resource entry to the resource storage contract through the rmrk token contract
#[test]
fn add_resource_entry_simple() {
    let sys = System::new();
    before_multiresource_test(&sys);
    let rmrk = sys.get_program(1);
    let resource_id: ResourceId = 1;
    let resource = Resource::Basic(Default::default());
    add_resource_entry(&rmrk, USERS[0], resource_id, resource);
}

#[test]
fn add_resource_entry_failures() {
    let sys = System::new();
    before_multiresource_test(&sys);
    let resource_id: ResourceId = 1;
    let resource = Resource::Basic(Default::default());

    let rmrk = sys.get_program(1);

    // add resource
    add_resource_entry(&rmrk, USERS[0], resource_id, resource.clone());

    // must fail since resource already exists
    assert!(rmrk
        .send(
            USERS[0],
            RMRKAction::AddResourceEntry {
                resource_id,
                resource: resource.clone(),
            },
        )
        .main_failed());

    // must fail since resource id is zero
    assert!(rmrk
        .send(
            USERS[0],
            RMRKAction::AddResourceEntry {
                resource_id: 0,
                resource,
            },
        )
        .main_failed());
}

// propose resource for the token
#[test]
fn add_resource_to_token() {
    let sys = System::new();
    before_multiresource_test(&sys);
    let rmrk = sys.get_program(1);
    let resource_id: ResourceId = 1;
    let resource = Resource::Basic(Default::default());
    // add resource entry to the storage contract
    add_resource_entry(&rmrk, USERS[0], resource_id, resource);

    let token_id: u64 = 10;
    let resource_id: u8 = 1;

    let res = add_resource(&rmrk, USERS[0], token_id, resource_id, 0);
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::ResourceAdded {
            token_id: token_id.into(),
            resource_id,
            overwrite_id: 0,
        }
        .encode()
    )));

    // check pending resources
    let mut pending_resources: BTreeSet<ResourceId> = BTreeSet::new();
    pending_resources.insert(resource_id);
    check_pending_resources(&rmrk, token_id, pending_resources);

    // check active resources
    check_active_resources(&rmrk, token_id, BTreeSet::new());
}

#[test]
fn add_resource_to_token_failures() {
    let sys = System::new();
    before_multiresource_test(&sys);
    let rmrk = sys.get_program(1);
    let resource = Resource::Basic(Default::default());
    let token_id: u64 = 10;
    let mut pending_resources: BTreeSet<ResourceId> = BTreeSet::new();

    // must fail since cannot add resource with not added resource id to a token
    assert!(add_resource(&rmrk, USERS[0], token_id, 1, 0).main_failed());

    // must fail since cannot since token does not exist
    assert!(add_resource(&rmrk, USERS[0], 11, 1, 0).main_failed());

    for resource_id in 1..129 {
        add_resource_entry(&rmrk, USERS[0], resource_id, resource.clone());
        pending_resources.insert(resource_id);
        assert!(!add_resource(&rmrk, USERS[0], token_id, resource_id, 0).main_failed());
    }

    // must fail since too many resources have already been added
    let resource_id: ResourceId = 129;
    add_resource_entry(&rmrk, USERS[0], resource_id, resource);
    assert!(add_resource(&rmrk, USERS[0], token_id, 129, 0).main_failed());

    // check pending resources
    check_pending_resources(&rmrk, token_id, pending_resources);

    // must fail since that resource has already been added
    assert!(add_resource(&rmrk, USERS[0], token_id, 100, 0).main_failed());
}

#[test]
fn add_resource_to_different_tokens() {
    let sys = System::new();
    before_multiresource_test(&sys);
    let rmrk = sys.get_program(1);
    let resource_id: u8 = 1;
    let resource = Resource::Basic(Default::default());
    add_resource_entry(&rmrk, USERS[0], resource_id, resource);

    let token_id_0: u64 = 10;
    let token_id_1: u64 = 11;

    // add resource to token_id_0
    assert!(!add_resource(&rmrk, USERS[0], token_id_0, resource_id, 0).main_failed());
    // add the same resource to token_id_1
    assert!(!add_resource(&rmrk, USERS[0], token_id_1, resource_id, 0).main_failed());

    let mut pending_resources: BTreeSet<ResourceId> = BTreeSet::new();
    pending_resources.insert(resource_id);

    // check pending resources of token_id_0
    check_pending_resources(&rmrk, token_id_0, pending_resources.clone());

    // check pending resources of token_id_1
    check_pending_resources(&rmrk, token_id_1, pending_resources);
}
