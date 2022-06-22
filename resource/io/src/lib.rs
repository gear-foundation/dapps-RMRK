#![no_std]

use codec::{Decode, Encode};
use gstd::prelude::*;
use primitive_types::{H256, U256};
use scale_info::TypeInfo;
pub type TokenId = U256;
pub type ResourceId = H256;

#[derive(Debug, Default, Clone, Encode, Decode, TypeInfo)]
pub struct Resource {
    pub id: u8,
    pub src: String,
    pub thumb: String,
    pub metadata_uri: String,
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
        id: u8,
        src: String,
        thumb: String,
        metadata_uri: String,
    },

    /// Used to check from the RMRK contract whether the resource with indicated id exists or not.
    ///
    /// # Arguments:
    /// * `id`: is a resource identifier.
    ///
    /// On success replies [`ResourceEvent::Resource`].
    GetResource { id: u8 },
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum ResourceEvent {
    ResourceEntryAdded { id: u8 },
    Resource(Resource),
}
