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
    AddResourceEntry {
        id: u8,
        src: String,
        thumb: String,
        metadata_uri: String,
    },
    GetResource {
        id: u8,
    },
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum ResourceEvent {
    ResourceEntryAdded { id: u8 },
    Resource(Resource),
}
