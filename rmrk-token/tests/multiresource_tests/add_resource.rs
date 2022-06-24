use crate::multiresource_tests::utils::*;
use codec::Encode;
use gstd::BTreeSet;
use gtest::System;
use resource_io::Resource;
use rmrk_io::*;

// adds resource entry to the resource storage contract through the rmrk token contract
#[test]
fn add_resource_entry_simple() {
    let sys = System::new();
    before_test(&sys);
    let rmrk = sys.get_program(1);
    let resource = Resource {
        id: 1,
        ..Default::default()
    };
    add_resource_entry(&rmrk, USERS[0], resource);
}

#[test]
fn add_resource_entry_failures() {
    let sys = System::new();
    before_test(&sys);
    let resource = Resource {
        id: 1,
        ..Default::default()
    };

    let rmrk = sys.get_program(1);

    // add resource
    add_resource_entry(&rmrk, USERS[0], resource.clone());

    // must fail since resource already exists
    assert!(rmrk
        .send(
            USERS[0],
            RMRKAction::AddResourceEntry {
                id: resource.id,
                src: resource.src.clone(),
                thumb: resource.thumb.clone(),
                metadata_uri: resource.metadata_uri.clone(),
            },
        )
        .main_failed());

    // must fail since resource id is zero
    assert!(rmrk
        .send(
            USERS[0],
            RMRKAction::AddResourceEntry {
                id: 0,
                src: resource.src,
                thumb: resource.thumb,
                metadata_uri: resource.metadata_uri,
            },
        )
        .main_failed());
}

// propose resource for the token
#[test]
fn add_resource_to_token() {
    let sys = System::new();
    before_test(&sys);
    let rmrk = sys.get_program(1);
    let resource = Resource {
        id: 1,
        ..Default::default()
    };
    // add resource entry to the storage contract
    add_resource_entry(&rmrk, USERS[0], resource);

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

    // // check pending resources
    // let res = get_pending_resources(&rmrk, token_id);
    // let mut pending_resources: BTreeSet<ResourceId> = BTreeSet::new();
    // pending_resources.insert(resource_id);
    // assert!(res.contains(&(
    //     10,
    //     RMRKEvent::PendingResources { pending_resources }.encode()
    // )));

    // // check active resources
    // let res = get_active_resources(&rmrk, token_id);
    // assert!(res.contains(&(
    //     10,
    //     RMRKEvent::ActiveResources {
    //         active_resources: BTreeSet::new()
    //     }
    //     .encode()
    // )));
}

#[test]
fn add_resource_to_token_failures() {
    let sys = System::new();
    before_test(&sys);
    let rmrk = sys.get_program(1);
    let mut resource = Resource {
        id: 1,
        ..Default::default()
    };
    let token_id: u64 = 10;
    let mut pending_resources: BTreeSet<ResourceId> = BTreeSet::new();

    // must fail since cannot add resource with not added resource id to a token
    assert!(add_resource(&rmrk, USERS[0], token_id, 1, 0).main_failed());

    // must fail since cannot since token does not exist
    assert!(add_resource(&rmrk, USERS[0], 11, 1, 0).main_failed());

    for resource_id in 1..129 {
        resource.id = resource_id;
        add_resource_entry(&rmrk, USERS[0], resource.clone());
        pending_resources.insert(resource_id);
        assert!(!add_resource(&rmrk, USERS[0], token_id, resource_id, 0).main_failed());
    }

    // must fail since too many resources have already been added
    resource.id = 129;
    add_resource_entry(&rmrk, USERS[0], resource);
    assert!(add_resource(&rmrk, USERS[0], token_id, 129, 0).main_failed());

    // // check pending resources
    // let res = get_pending_resources(&rmrk, token_id);
    // assert!(res.contains(&(
    //     10,
    //     RMRKEvent::PendingResources { pending_resources }.encode()
    // )));

    // must fail since that resource has already been added
    assert!(add_resource(&rmrk, USERS[0], token_id, 100, 0).main_failed());
}

#[test]
fn add_resource_to_different_tokens() {
    let sys = System::new();
    before_test(&sys);
    let rmrk = sys.get_program(1);
    let resource = Resource {
        id: 1,
        ..Default::default()
    };
    add_resource_entry(&rmrk, USERS[0], resource);

    let token_id_0: u64 = 10;
    let token_id_1: u64 = 11;
    let resource_id: u8 = 1;

    // add resource to token_id_0
    assert!(!add_resource(&rmrk, USERS[0], token_id_0, resource_id, 0).main_failed());
    // add the same resource to token_id_1
    assert!(!add_resource(&rmrk, USERS[0], token_id_1, resource_id, 0).main_failed());

    let mut pending_resources: BTreeSet<ResourceId> = BTreeSet::new();
    pending_resources.insert(resource_id);

    // // check pending resources of token_id_0
    // let res = get_pending_resources(&rmrk, token_id_0);
    // assert!(res.contains(&(
    //     10,
    //     RMRKEvent::PendingResources {
    //         pending_resources: pending_resources.clone()
    //     }
    //     .encode()
    // )));

    // // check pending resources of token_id_1
    // let res = get_pending_resources(&rmrk, token_id_1);
    // assert!(res.contains(&(
    //     10,
    //     RMRKEvent::PendingResources { pending_resources }.encode()
    // )));
}
