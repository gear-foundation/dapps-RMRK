#![no_std]

use codec::{Decode, Encode};
use gstd::prelude::*;
use scale_info::TypeInfo;
use types::primitives::{BaseId, PartId, ResourceId, SlotId};

#[derive(Debug, Default, Clone, Encode, Decode, TypeInfo)]
pub struct BasicResource {
    /// URI like ipfs hash
    pub src: String,

    /// If the resource has the thumb property, this will be a URI to a thumbnail of the given
    /// resource.
    pub thumb: Option<String>,

    /// Reference to IPFS location of metadata
    pub metadata_uri: String,
}

#[derive(Debug, Default, Clone, Encode, Decode, TypeInfo)]
pub struct ComposedResource {
    /// URI like ipfs hash
    pub src: String,

    /// If the resource has the thumb property, this will be a URI to a thumbnail of the given
    /// resource.
    pub thumb: String,

    /// Reference to IPFS location of metadata
    pub metadata_uri: String,

    // The address of base contract
    pub base: BaseId,

    //  If a resource is composed, it will have an array of parts that compose it
    pub parts: Vec<PartId>,
}

#[derive(Debug, Default, Clone, Encode, Decode, TypeInfo)]
pub struct SlotResource {
    /// URI like ipfs hash
    pub src: String,

    /// If the resource has the thumb property, this will be a URI to a thumbnail of the given
    /// resource.
    pub thumb: String,

    /// Reference to IPFS location of metadata
    pub metadata_uri: String,

    // The address of base contract
    pub base: BaseId,

    /// If the resource has the slot property, it was designed to fit into a specific Base's slot.
    pub slot: SlotId,
}
#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub enum Resource {
    Basic(BasicResource),
    Slot(SlotResource),
    Composed(ComposedResource),
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct InitResource {
    pub resource_name: String,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum ResourceAction {
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
    AddResourceEntry {
        resource_id: ResourceId,
        resource: Resource,
    },

    /// Used to check from the RMRK contract whether the resource with indicated id exists or not.
    ///
    /// # Arguments:
    /// * `id`: is a resource identifier.
    ///
    /// On success replies [`ResourceEvent::Resource`].
    GetResource { id: ResourceId },
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum ResourceEvent {
    ResourceEntryAdded {
        resource_id: ResourceId,
        resource: Resource,
    },
    Resource(Resource),
}
