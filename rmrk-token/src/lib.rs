#![no_std]

use codec::Encode;
use gstd::{debug, msg, prelude::*, prog, ActorId};
use primitive_types::{U256};
use resource_io::{InitResource, ResourceAction, ResourceEvent};
use rmrk_io::*;
pub mod burn;
pub mod checks;
pub mod children;
pub mod messages;
pub mod transfer;
use messages::*;
pub mod mint;
pub mod multiresource;
use multiresource::*;

gstd::metadata! {
    title: "RMRK",
    init:
        input: InitRMRK,
    handle:
        input: RMRKAction,
        output: RMRKEvent,
    state:
        input: RMRKState,
        output: RMRKStateReply,
}

#[derive(Debug, Default)]
pub struct RMRKOwner {
    pub token_id: Option<TokenId>,
    pub owner_id: ActorId,
}

#[derive(Debug, Default)]
pub struct RMRKToken {
    pub name: String,
    pub symbol: String,
    pub admin: ActorId,
    pub token_approvals: BTreeMap<TokenId, BTreeSet<ActorId>>,
    pub rmrk_owners: BTreeMap<TokenId, RMRKOwner>,
    pub pending_children: BTreeMap<TokenId, BTreeSet<Vec<u8>>>,
    pub accepted_children: BTreeMap<TokenId, BTreeSet<Vec<u8>>>,
    pub children_status: BTreeMap<Vec<u8>, ChildStatus>,
    pub balances: BTreeMap<ActorId, U256>,
    pub multiresource: MultiResource,
    pub resource_hash: [u8; 32],
    pub resource_id: ActorId,
}

static mut RMRK: Option<RMRKToken> = None;
pub const ZERO_ID: ActorId = ActorId::new([0u8; 32]);

impl RMRKToken {
    // reply about root_owner
    async fn root_owner(&self, token_id: TokenId) {
        let root_owner = self.find_root_owner(token_id).await;
        msg::reply(RMRKEvent::RootOwner(root_owner), 0)
            .expect("Error in reply [RMRKEvent::RootOwner]");
    }

    // internal search for root owner
    async fn find_root_owner(&self, token_id: TokenId) -> ActorId {
        let rmrk_owner = self
            .rmrk_owners
            .get(&token_id)
            .expect("RMRK: Token does not exist");
        if rmrk_owner.token_id.is_some() {
            get_root_owner(&rmrk_owner.owner_id, rmrk_owner.token_id.unwrap()).await
        } else {
            rmrk_owner.owner_id
        }
    }

    fn get_pending_children(&self, token_id: TokenId) -> BTreeMap<ActorId, BTreeSet<TokenId>> {
        let mut pending_children: BTreeMap<ActorId, BTreeSet<TokenId>> = BTreeMap::new();
        if let Some(children) = self.pending_children.get(&token_id) {
            for child_vec in children.iter() {
                let child_contract_id = ActorId::new(child_vec[0..32].try_into().unwrap());
                let child_token_id = U256::from(&child_vec[32..64]);
                pending_children
                    .entry(child_contract_id)
                    .and_modify(|c| {
                        c.insert(child_token_id);
                    })
                    .or_insert_with(|| BTreeSet::from([child_token_id]));
            }
        }
        pending_children
    }

    fn get_accepted_children(&self, token_id: TokenId) -> BTreeMap<ActorId, BTreeSet<TokenId>> {
        let mut accepted_children: BTreeMap<ActorId, BTreeSet<TokenId>> = BTreeMap::new();
        if let Some(children) = self.accepted_children.get(&token_id) {
            for child_vec in children.iter() {
                let child_contract_id = ActorId::new(child_vec[0..32].try_into().unwrap());
                let child_token_id = U256::from(&child_vec[32..64]);
                accepted_children
                    .entry(child_contract_id)
                    .and_modify(|c| {
                        c.insert(child_token_id);
                    })
                    .or_insert_with(|| BTreeSet::from([child_token_id]));
            }
        }
        accepted_children
    }
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitRMRK = msg::load().expect("Unable to decode InitRMRK");

    let mut rmrk = RMRKToken {
        name: config.name,
        symbol: config.symbol,
        admin: msg::source(),
        ..RMRKToken::default()
    };
    if let Some(resource_hash) = config.resource_hash {
        let resource_id = prog::create_program_with_gas(
            resource_hash.into(),
            &0i32.to_le_bytes(),
            InitResource {
                resource_name: config.resource_name,
            }
            .encode(),
            1_000_000,
            0,
        )
        .expect("Error in creating program");
        rmrk.resource_id = resource_id;
        debug!("PROGRAM RESOURCE ID {:?}", resource_id);
        msg::reply(RMRKEvent::ResourceInited { resource_id }, 0).unwrap();
    }
    RMRK = Some(rmrk);
}

#[gstd::async_main]
async unsafe fn main() {
    let action: RMRKAction = msg::load().expect("Could not load msg");
    let rmrk = unsafe { RMRK.get_or_insert(Default::default()) };
    match action {
        RMRKAction::MintToNft {
            parent_id,
            parent_token_id,
            token_id,
        } => {
            rmrk.mint_to_nft(&parent_id, parent_token_id, token_id)
                .await
        }
        RMRKAction::MintToRootOwner {
            root_owner,
            token_id,
        } => rmrk.mint_to_root_owner(&root_owner, token_id),
        RMRKAction::Transfer { to, token_id } => rmrk.transfer(&to, token_id).await,
        RMRKAction::TransferToNft {
            to,
            destination_id,
            token_id,
        } => rmrk.transfer_to_nft(&to, destination_id, token_id).await,
        RMRKAction::Approve { to, token_id } => rmrk.approve(&to, token_id).await,
        RMRKAction::AddChild {
            parent_token_id,
            child_token_id,
        } => rmrk.add_child(parent_token_id, child_token_id).await,
        RMRKAction::AcceptChild {
            parent_token_id,
            child_contract_id,
            child_token_id,
        } => {
            rmrk.accept_child(parent_token_id, &child_contract_id, child_token_id)
                .await
        }
        RMRKAction::AddAcceptedChild {
            parent_token_id,
            child_token_id,
        } => {
            rmrk.add_accepted_child(parent_token_id, child_token_id)
                .await
        }
        RMRKAction::TransferChild {
            from,
            to,
            child_token_id,
        } => rmrk.transfer_child(from, to, child_token_id).await,
        RMRKAction::RejectChild {
            parent_token_id,
            child_contract_id,
            child_token_id,
        } => {
            rmrk.reject_child(parent_token_id, &child_contract_id, child_token_id)
                .await
        }
        RMRKAction::RemoveChild {
            parent_token_id,
            child_contract_id,
            child_token_id,
        } => {
            rmrk.remove_child(parent_token_id, &child_contract_id, child_token_id)
                .await
        }
        RMRKAction::BurnChild {
            parent_token_id,
            child_token_id,
        } => rmrk.burn_child(parent_token_id, child_token_id),
        RMRKAction::BurnFromParent {
            child_token_ids,
            root_owner,
        } => rmrk.burn_from_parent(child_token_ids, &root_owner).await,
        RMRKAction::Burn(token_id) => rmrk.burn(token_id).await,
        RMRKAction::RootOwner(token_id) => rmrk.root_owner(token_id).await,
        RMRKAction::AddResourceEntry {
            id,
            src,
            thumb,
            metadata_uri,
        } => rmrk.add_resource_entry(id, src, thumb, metadata_uri).await,
        RMRKAction::AddResource {
            token_id,
            resource_id,
            overwrite_id,
        } => rmrk.add_resource(token_id, resource_id, overwrite_id).await,
        RMRKAction::AcceptResource {
            token_id,
            resource_id,
        } => rmrk.accept_resource(token_id, resource_id).await,
        RMRKAction::RejectResource {
            token_id,
            resource_id,
        } => rmrk.reject_resource(token_id, resource_id).await,
        RMRKAction::SetPriority {
            token_id,
            priorities,
        } => rmrk.set_priority(token_id, priorities).await,
    }
}

#[no_mangle]
pub unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
    let query: RMRKState = msg::load().expect("failed to decode RMRKState");
    let rmrk = RMRK.get_or_insert(Default::default());

    let encoded = match query {
        RMRKState::Owner(token_id) => {
            if let Some(rmrk_owner) = rmrk.rmrk_owners.get(&token_id) {
                RMRKStateReply::Owner {
                    token_id: rmrk_owner.token_id,
                    owner_id: rmrk_owner.owner_id,
                }
            } else {
                RMRKStateReply::Owner {
                    token_id: None,
                    owner_id: ZERO_ID,
                }
            }
        }
        RMRKState::Balance(account) => {
            if let Some(balance) = rmrk.balances.get(&account) {
                RMRKStateReply::Balance(*balance)
            } else {
                RMRKStateReply::Balance(0.into())
            }
        }
        RMRKState::PendingChildren(token_id) => {
            RMRKStateReply::PendingChildren(rmrk.get_pending_children(token_id))
        }
        RMRKState::AcceptedChildren(token_id) => {
            RMRKStateReply::AcceptedChildren(rmrk.get_accepted_children(token_id))
        }
        RMRKState::PendingResources(token_id) => {
            let pending_resources = rmrk
                .multiresource
                .pending_resources
                .get(&token_id)
                .unwrap_or(&BTreeSet::new())
                .clone();
            RMRKStateReply::PendingResources(pending_resources)
        }
        RMRKState::ActiveResources(token_id) => {
            let active_resources = rmrk
                .multiresource
                .active_resources
                .get(&token_id)
                .unwrap_or(&BTreeSet::new())
                .clone();
            RMRKStateReply::ActiveResources(active_resources)
        }
    }
    .encode();
    gstd::util::to_leak_ptr(encoded)
}
