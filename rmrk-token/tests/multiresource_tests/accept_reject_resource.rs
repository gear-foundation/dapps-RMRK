use crate::utils::*;
use codec::Encode;
use gstd::BTreeSet;
use gtest::System;
use resource_io::Resource;
use rmrk_io::*;
use types::primitives::ResourceId;

#[test]
fn accept_resource_simple() {
    let sys = System::new();
    before_multiresource_test(&sys);
    let rmrk = sys.get_program(1);
    let resource_id: ResourceId = 1;
    let resource = Resource::Basic(Default::default());

    add_resource_entry(&rmrk, USERS[0], resource_id, resource);

    let token_id: u64 = 10;

    assert!(!add_resource(&rmrk, USERS[0], token_id, resource_id, 0).main_failed());

    let res = accept_resource(&rmrk, USERS[0], token_id, resource_id);
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::ResourceAccepted {
            token_id: token_id.into(),
            resource_id,
        }
        .encode()
    )));

    // check pending resources
    check_pending_resources(&rmrk, token_id, BTreeSet::new());

    // check active resources
    let mut active_resources: BTreeSet<ResourceId> = BTreeSet::new();
    active_resources.insert(resource_id);
    check_active_resources(&rmrk, token_id, active_resources);
}

#[test]
fn accept_resource_from_approved_address() {
    let sys = System::new();
    before_multiresource_test(&sys);
    let rmrk = sys.get_program(1);
    let resource_id: ResourceId = 1;
    let resource = Resource::Basic(Default::default());
    add_resource_entry(&rmrk, USERS[0], resource_id, resource);

    let token_id: u64 = 10;

    assert!(!add_resource(&rmrk, USERS[0], token_id, resource_id, 0).main_failed());

    assert!(!approve(&rmrk, USERS[0], USERS[3], token_id.into()).main_failed());

    let res = accept_resource(&rmrk, USERS[3], token_id, resource_id);
    assert!(res.contains(&(
        USERS[3],
        RMRKEvent::ResourceAccepted {
            token_id: token_id.into(),
            resource_id
        }
        .encode()
    )));

    // check pending resources
    check_pending_resources(&rmrk, token_id, BTreeSet::new());

    // check active resources
    let mut active_resources: BTreeSet<ResourceId> = BTreeSet::new();
    active_resources.insert(resource_id);
    check_active_resources(&rmrk, token_id, active_resources);
}

#[test]
fn accept_resource_failures() {
    let sys = System::new();
    before_multiresource_test(&sys);
    let rmrk = sys.get_program(1);
    let resource_id: ResourceId = 1;
    let resource = Resource::Basic(Default::default());

    add_resource_entry(&rmrk, USERS[0], resource_id, resource);

    let token_id: u64 = 10;

    assert!(!add_resource(&rmrk, USERS[0], token_id, resource_id, 0).main_failed());

    // must fail since not owner/approved tries to accept resource
    assert!(accept_resource(&rmrk, USERS[2], token_id, resource_id).main_failed());

    // must fail since resource with indicated id does not exist
    assert!(accept_resource(&rmrk, USERS[2], token_id, 2).main_failed());
}

#[test]
fn accept_multiple_resources() {
    let sys = System::new();
    before_multiresource_test(&sys);
    let rmrk = sys.get_program(1);
    let resource_id_1: ResourceId = 1;
    let resource_id_2: ResourceId = 2;
    let resource = Resource::Basic(Default::default());

    let token_id: u64 = 10;

    add_resource_entry(&rmrk, USERS[0], resource_id_1, resource.clone());

    add_resource_entry(&rmrk, USERS[0], resource_id_2, resource);

    assert!(!add_resource(&rmrk, USERS[0], token_id, resource_id_1, 0).main_failed());
    assert!(!add_resource(&rmrk, USERS[0], token_id, resource_id_2, 0).main_failed());

    assert!(!accept_resource(&rmrk, USERS[0], token_id, resource_id_1).main_failed());
    assert!(!accept_resource(&rmrk, USERS[0], token_id, resource_id_2).main_failed());

    // check pending resources
    check_pending_resources(&rmrk, token_id, BTreeSet::new());

    // check active resources
    let mut active_resources: BTreeSet<ResourceId> = BTreeSet::new();
    active_resources.insert(resource_id_1);
    active_resources.insert(resource_id_2);
    check_active_resources(&rmrk, token_id, active_resources);
}

#[test]
fn reorder_prioroties() {
    let sys = System::new();
    before_multiresource_test(&sys);
    let rmrk = sys.get_program(1);
    let resource = Resource::Basic(Default::default());
    let token_id: u64 = 10;

    for resource_id in 1..6 {
        add_resource_entry(&rmrk, USERS[0], resource_id, resource.clone());
        assert!(!add_resource(&rmrk, USERS[0], token_id, resource_id, 0).main_failed());
        assert!(!accept_resource(&rmrk, USERS[0], token_id, resource_id).main_failed());
    }
    let priorities = vec![1, 0, 4, 3, 2];
    let res = set_priority(&rmrk, USERS[0], token_id, priorities.clone());
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::PrioritySet {
            token_id: token_id.into(),
            priorities
        }
        .encode()
    )));
}

#[test]
fn reorder_prioroties_failures() {
    let sys = System::new();
    before_multiresource_test(&sys);
    let rmrk = sys.get_program(1);
    let resource = Resource::Basic(Default::default());
    let token_id: u64 = 10;

    for resource_id in 1..4 {
        add_resource_entry(&rmrk, USERS[0], resource_id, resource.clone());
        assert!(!add_resource(&rmrk, USERS[0], token_id, resource_id, 0).main_failed());
        assert!(!accept_resource(&rmrk, USERS[0], token_id, resource_id).main_failed());
    }
    let mut priorities = vec![1, 0, 2];

    // must fail since not owner/approved tries to reorder priorities
    assert!(set_priority(&rmrk, USERS[1], token_id, priorities.clone()).main_failed());

    // must fail since the new order has does not have the same length
    priorities.push(3);
    assert!(set_priority(&rmrk, USERS[0], token_id, priorities.clone()).main_failed());
}

#[test]
fn reorder_prioroties_from_approved_address() {
    let sys = System::new();
    before_multiresource_test(&sys);
    let rmrk = sys.get_program(1);
    let resource = Resource::Basic(Default::default());
    let token_id: u64 = 10;

    for resource_id in 1..4 {
        add_resource_entry(&rmrk, USERS[0], resource_id, resource.clone());
        assert!(!add_resource(&rmrk, USERS[0], token_id, resource_id, 0).main_failed());
        assert!(!accept_resource(&rmrk, USERS[0], token_id, resource_id).main_failed());
    }
    assert!(!approve(&rmrk, USERS[0], USERS[3], token_id.into()).main_failed());
    let priorities = vec![1, 0, 3];
    let res = set_priority(&rmrk, USERS[3], token_id, priorities.clone());
    assert!(res.contains(&(
        USERS[3],
        RMRKEvent::PrioritySet {
            token_id: token_id.into(),
            priorities
        }
        .encode()
    )));
}

#[test]
fn reject_resource_simple() {
    let sys = System::new();
    before_multiresource_test(&sys);
    let rmrk = sys.get_program(1);
    let token_id: u64 = 10;
    let resource_id: ResourceId = 1;
    let resource = Resource::Basic(Default::default());
    add_resource_entry(&rmrk, USERS[0], resource_id, resource);

    assert!(!add_resource(&rmrk, USERS[0], token_id, resource_id, 0).main_failed());

    let res = reject_resource(&rmrk, USERS[0], token_id, resource_id);
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::ResourceRejected {
            token_id: token_id.into(),
            resource_id,
        }
        .encode()
    )));

    // check pending resources
    check_pending_resources(&rmrk, token_id, BTreeSet::new());

    // check active resources
    check_active_resources(&rmrk, token_id, BTreeSet::new());
}

#[test]
fn reject_resource_failures() {
    let sys = System::new();
    before_multiresource_test(&sys);
    let rmrk = sys.get_program(1);
    let token_id: u64 = 10;
    let resource_id: ResourceId = 1;
    let resource = Resource::Basic(Default::default());
    add_resource_entry(&rmrk, USERS[0], resource_id, resource);

    // must fail since token does not have any pending resources
    assert!(reject_resource(&rmrk, USERS[0], token_id, resource_id).main_failed());

    // add resource index
    assert!(!add_resource(&rmrk, USERS[0], token_id, resource_id, 0).main_failed());

    // must fail since resource does not exist
    assert!(reject_resource(&rmrk, USERS[0], token_id, 10).main_failed());

    // must fail since not owner/approved tries to reject resource
    assert!(reject_resource(&rmrk, USERS[3], token_id, resource_id).main_failed());
}

#[test]
fn reject_resource_from_approved_address() {
    let sys = System::new();
    before_multiresource_test(&sys);
    let rmrk = sys.get_program(1);
    let token_id: u64 = 10;
    let resource_id: ResourceId = 1;
    let resource = Resource::Basic(Default::default());
    add_resource_entry(&rmrk, USERS[0], resource_id, resource);

    assert!(!add_resource(&rmrk, USERS[0], token_id, resource_id, 0).main_failed());

    assert!(!approve(&rmrk, USERS[0], USERS[3], token_id.into()).main_failed());

    assert!(!reject_resource(&rmrk, USERS[3], token_id, resource_id).main_failed());

    // check pending resources
    check_pending_resources(&rmrk, token_id, BTreeSet::new());

    // check active resources
    check_active_resources(&rmrk, token_id, BTreeSet::new());
}
