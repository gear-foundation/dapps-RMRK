#![no_std]

use codec::{Decode, Encode};
use gstd::{prelude::*, ActorId};
use primitive_types::{H256, U256};
use scale_info::TypeInfo;
pub type TokenId = U256;
pub type ResourceId = u8;

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct InitRMRK {
    pub name: String,
    pub symbol: String,
    pub resource_name: Option<String>,
    pub resource_hash: [u8; 32],
}

#[derive(Debug, Clone, Encode)]
pub struct Child {
    pub token_id: TokenId,
    pub status: ChildStatus,
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, Copy, PartialEq)]
pub enum ChildStatus {
    Pending,
    Accepted,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum RMRKAction {
    MintToNft {
        to: ActorId,
        token_id: TokenId,
        destination_id: TokenId,
    },
    MintToRootOwner {
        to: ActorId,
        token_id: TokenId,
    },
    Burn {
        token_id: TokenId,
    },
    BurnFromParent {
        child_token_ids: Vec<TokenId>,
        root_owner: ActorId,
    },
    BurnChild {
        parent_token_id: TokenId,
        child_token_id: TokenId,
    },
    Transfer {
        to: ActorId,
        token_id: TokenId,
    },
    TransferToNft {
        to: ActorId,
        token_id: TokenId,
        destination_id: TokenId,
    },
    TransferChild {
        from: TokenId,
        to: TokenId,
        child_token_id: TokenId,
    },
    Approve {
        to: ActorId,
        token_id: TokenId,
    },
    AddChild {
        parent_token_id: TokenId,
        child_token_id: TokenId,
    },
    AddAcceptedChild {
        parent_token_id: TokenId,
        child_token_id: TokenId,
    },
    RejectChild {
        parent_token_id: TokenId,
        child_contract_id: ActorId,
        child_token_id: TokenId,
    },
    RemoveChild {
        parent_token_id: TokenId,
        child_contract_id: ActorId,
        child_token_id: TokenId,
    },
    AcceptChild {
        parent_token_id: TokenId,
        child_contract_id: ActorId,
        child_token_id: TokenId,
    },
    RootOwner {
        token_id: TokenId,
    },
    Owner {
        token_id: TokenId,
    },
    PendingChildren {
        token_id: TokenId,
    },
    AcceptedChildren {
        token_id: TokenId,
    },
    AddResourceEntry {
        id: u8,
        src: String,
        thumb: String,
        metadata_uri: String,
    },
    AddResource {
        token_id: TokenId,
        resource_id: u8,
        overwrite_id: u8,
    },
    AcceptResource {
        token_id: TokenId,
        resource_id: u8,
    },
    RejectResource {
        token_id: TokenId,
        resource_id: u8,
    },
    SetPriority {
        token_id: TokenId,
        priorities: Vec<u8>,
    },
    GetPendingResources {
        token_id: TokenId,
    },
    GetActiveResources {
        token_id: TokenId,
    },
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum RMRKEvent {
    MintToNft {
        to: ActorId,
        token_id: TokenId,
        destination_id: TokenId,
    },
    MintToRootOwner {
        to: ActorId,
        token_id: TokenId,
    },
    Approval {
        owner: ActorId,
        approved_account: ActorId,
        token_id: TokenId,
    },
    PendingChild {
        child_token_address: ActorId,
        child_token_id: TokenId,
        parent_token_id: TokenId,
        root_owner: ActorId,
    },
    AcceptedChild {
        child_token_address: ActorId,
        child_token_id: TokenId,
        parent_token_id: TokenId,
    },
    RejectedChild {
        child_token_address: ActorId,
        child_token_id: TokenId,
        parent_token_id: TokenId,
    },
    RemovedChild {
        child_token_address: ActorId,
        child_token_id: TokenId,
        parent_token_id: TokenId,
    },
    ChildAdded {
        parent_token_id: TokenId,
        child_token_id: TokenId,
        child_status: ChildStatus,
    },
    ChildBurnt {
        parent_token_id: TokenId,
        child_token_id: TokenId,
    },
    ChildTransferred {
        from: TokenId,
        to: TokenId,
        child_contract_id: ActorId,
        child_token_id: TokenId,
    },
    TokensBurnt {
        token_ids: Vec<TokenId>,
    },
    NFTParent {
        parent: ActorId,
    },
    RootOwner {
        root_owner: ActorId,
    },

    Transfer {
        to: ActorId,
        token_id: TokenId,
    },
    TransferToNft {
        to: ActorId,
        token_id: TokenId,
        destination_id: TokenId,
    },
    Owner {
        token_id: Option<TokenId>,
        owner_id: ActorId,
    },
    PendingChildren {
        children: BTreeMap<ActorId, Vec<TokenId>>,
    },
    AcceptedChildren {
        children: BTreeMap<ActorId, Vec<TokenId>>,
    },
    ResourceEntryAdded {
        id: u8,
    },
    ResourceAdded {
        token_id: TokenId,
        resource_id: ResourceId,
        overwrite_id: ResourceId,
    },
    ResourceAccepted {
        token_id: TokenId,
        resource_id: ResourceId,
    },
    ResourceRejected {
        token_id: TokenId,
        resource_id: ResourceId,
    },
    PrioritySet {
        token_id: TokenId,
        priorities: Vec<u8>,
    },
    ResourceInited {
        resource_id: ActorId,
    },
    PendingResources {
        pending_resources: BTreeSet<ResourceId>,
    },
    ActiveResources {
        active_resources: BTreeSet<ResourceId>,
    },
}
