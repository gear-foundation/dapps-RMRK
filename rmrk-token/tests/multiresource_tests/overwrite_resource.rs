use crate::multiresource_tests::utils::*;
use codec::Encode;
use gstd::{ActorId, BTreeMap, BTreeSet};
use gtest::{Program, System};
use primitive_types::H256;
use resource_io::{Resource, ResourceAction, ResourceEvent};
use rmrk_io::*;

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
    add_resource_entry(&rmrk, USERS[0], resource.clone());
    // add overwrite resource to storage contract
    add_resource_entry(&rmrk, USERS[0], new_resource.clone());

    let token_id: u64 = 10;
    let resource_id: u8 = 1;
    let new_resource_id: u8 = 2;

    // add and accept resource_id
    assert!(!add_resource(&rmrk, USERS[0], token_id, resource_id, 0).main_failed());
    assert!(!accept_resource(&rmrk, USERS[0], token_id, resource_id).main_failed());

    // add resource to overwrite
    assert!(!add_resource(&rmrk, USERS[0], token_id, new_resource_id, resource_id).main_failed());
    // check pending resources
    let res = get_pending_resources(&rmrk, token_id);
    let mut resources: BTreeSet<ResourceId> = BTreeSet::new();
    resources.insert(new_resource_id);
    assert!(res.contains(&(
        10,
        RMRKEvent::PendingResources {
            pending_resources: resources.clone()
        }
        .encode()
    )));

    // accept new resource instead of previous one
    assert!(!accept_resource(&rmrk, USERS[0], token_id, new_resource_id).main_failed());
    // check active resources
    let res = get_active_resources(&rmrk, token_id);
    assert!(res.contains(&(
        10,
        RMRKEvent::ActiveResources {
            active_resources: resources
        }
        .encode()
    )));
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
    add_resource_entry(&rmrk, USERS[0], resource_1.clone());
    add_resource_entry(&rmrk, USERS[0], resource_2.clone());
    add_resource_entry(&rmrk, USERS[0], resource_3.clone());

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
