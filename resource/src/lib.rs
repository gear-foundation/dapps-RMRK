#![no_std]

use gstd::{msg, prelude::*, ActorId};
use resource_io::*;

#[derive(Debug, Default)]
pub struct ResourceStorage {
    pub name: String,
    // the admin is the rmrk contract that initializes the storage contract
    pub admin: ActorId,
    pub resources: BTreeMap<u8, Resource>,
    pub all_resources: Vec<Resource>,
}

static mut RESOURCE_STORAGE: Option<ResourceStorage> = None;
pub const ZERO_ID: ActorId = ActorId::new([0u8; 32]);

impl ResourceStorage {
    /// Adds resource entry on resource storage contract.
    ///
    /// # Requirements:
    /// * The `msg::source()` must be the contract admin (RMRK contract).
    /// * `id` can not be equal to zero.
    /// * Resource with indicated `id` must not exist.
    ///
    /// # Arguments:
    /// * `id`: is a resource identifier.
    /// * `src`: a string pointing to the media associated with the resource.
    /// * `thumb`: a string pointing to thumbnail media associated with the resource.
    /// * `metadata_uri`:  a string pointing to a metadata file associated with the resource.
    ///
    /// On success replies [`ResourceEvent::ResourceEntryAdded`].
    fn add_resource_entry(&mut self, resource_id: u8, resource: Resource) {
        assert!(resource_id != 0, " Write to zero");
        assert!(msg::source() == self.admin, "Not admin");
        assert!(
            self.resources
                .insert(resource_id, resource.clone())
                .is_none(),
            "resource already exists"
        );
        self.all_resources.push(resource.clone());
        msg::reply(
            ResourceEvent::ResourceEntryAdded {
                resource_id,
                resource,
            },
            0,
        )
        .expect("Error in reply `[ResourceEvent::ResourceEntryAdded]`");
    }
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitResource = msg::load().expect("Unable to decode InitResource");
    let resource = ResourceStorage {
        name: config.resource_name,
        admin: msg::source(),
        ..ResourceStorage::default()
    };
    RESOURCE_STORAGE = Some(resource);
}

#[gstd::async_main]
async unsafe fn main() {
    let action: ResourceAction = msg::load().expect("Could not load msg");
    let storage = unsafe { RESOURCE_STORAGE.get_or_insert(Default::default()) };
    match action {
        ResourceAction::AddResourceEntry {
            resource_id,
            resource,
        } => storage.add_resource_entry(resource_id, resource),
        ResourceAction::GetResource { id } => {
            let resource = storage.resources.get(&id).expect("Resource is not found");
            msg::reply(ResourceEvent::Resource(resource.clone()), 0)
                .expect("Error in reply `[ResourceEvent::Resource]`");
        }
    }
}
