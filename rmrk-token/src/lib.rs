#![no_std]

use children::{accept_child_reply, reject_child_reply, remove_child_reply, transfer_child_reply};
use equippable::{equip_reply, Assets};
use gstd::{debug, exec, msg, prelude::*, prog::ProgramGenerator, ActorId, MessageId};
use mint::mint_to_nft_reply;
use resource_io::{InitResource, ResourceAction, ResourceEvent};
use rmrk_io::*;
use transfer::{transfer_reply, transfer_to_nft_reply};
use types::primitives::{BaseId, CollectionAndToken, PartId, TokenId};
mod burn;
mod checks;
mod children;
mod equippable;
mod messages;
mod transfer;
use messages::*;
mod mint;
mod multiresource;
use multiresource::*;
mod utils;
use hashbrown::{HashMap, HashSet};

pub mod tx_manager;
use tx_manager::TxManager;

#[derive(Debug, Default)]
struct RMRKToken {
    name: String,
    symbol: String,
    admin: ActorId,
    token_approvals: HashMap<TokenId, HashSet<ActorId>>,
    rmrk_owners: HashMap<TokenId, RMRKOwner>,
    pending_children: HashMap<TokenId, HashSet<CollectionAndToken>>,
    accepted_children: HashMap<TokenId, HashSet<CollectionAndToken>>,
    children_status: HashMap<CollectionAndToken, ChildStatus>,
    balances: HashMap<ActorId, TokenId>,
    multiresource: MultiResource,
    resource_id: ActorId,
}

static mut RMRK: Option<RMRKToken> = None;
static mut ASSETS: Option<Assets> = None;
static mut TX_MANAGER: Option<TxManager> = None;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum TxState {
    #[default]
    Initial,
    MsgGetRootOwnerSent,
    ReplyRootOwnerReceived,
    MsgGetNewRootOwnerSent,
    ReplyNewRootOwnerReceived,
    MsgAddChildSent,
    ReplyAddChildReceived,
    MsgBurnChildSent,
    MsgAddAcceptedChildSent,
    ReplyOnBurnChildReceived,
    MsgTransferChildSent,
    ReplyOnTransferChildReceived,
    ReplyOnAddAcceptedChildReceived,
    MsgBurnFromParentSent,
    ReplyOnBurnFromParentReceived,
    MsgAddResourceSent,
    ReplyOnAddResourceReceived,
    MsgGetResourceSent,
    ReplyOnGetResourceReceived,
    MsgCheckEquippableSent,
    ReplyCheckEquippableReceived,
    MsgCanTokenBeEquippedSent,
    ReplyCanTokenBeEquippedReceived,
    Completed,
    Error(RMRKError),
}

#[derive(Clone, Debug)]
pub enum MintToNft {
    Initial,
    MsgGetRootOwnerSent,
    ReplyRootOwnerReceived,
    MsgAddChildSent,
    ReplyAddChildReceived,
}

#[derive(Debug, Clone)]
pub struct Tx {
    msg: RMRKAction,
    state: TxState,
    data: Option<Vec<u8>>,
}

impl RMRKToken {
    // reply about root_owner
    fn root_owner(&self, tx_manager: &mut TxManager, msg: &RMRKAction, token_id: TokenId) {
        let rmrk_owner = self
            .rmrk_owners
            .get(&token_id)
            .expect("RMRK: Token does not exist");

        let tx = tx_manager.get_tx(msg);
        let state = tx.state.clone();
        let current_msg_id = msg::id();

        let root_owner = match state {
            TxState::Initial => {
                if let Some(parent_token_id) = rmrk_owner.token_id {
                    let msg_id = get_root_owner_msg(&rmrk_owner.owner_id, parent_token_id);
                    tx.state = TxState::MsgGetRootOwnerSent;
                    tx_manager.msg_sent_to_msg.insert(msg_id, current_msg_id);
                    exec::wait_for(5);
                } else {
                    rmrk_owner.owner_id
                }
            }
            TxState::ReplyRootOwnerReceived => {
                let reply = tx.data.take().expect("Failed to get a reply");
                decode_root_owner(reply)
            }
            _ => {
                unreachable!()
            }
        };

        msg::reply(RMRKReply::RootOwner(root_owner), 0)
            .expect("Error in reply [RMRKEvent::RootOwner]");
    }
}

#[no_mangle]
extern "C" fn init() {
    let config: InitRMRK = msg::load().expect("Unable to decode InitRMRK");
    let tx_manager: TxManager = Default::default();
    let mut rmrk = RMRKToken {
        name: config.name,
        symbol: config.symbol,
        admin: msg::source(),
        ..RMRKToken::default()
    };
    let assets: Assets = Default::default();
    if let Some(resource_hash) = config.resource_hash {
        let (_message_id, resource_id) = ProgramGenerator::create_program(
            resource_hash.into(),
            InitResource {
                resource_name: config.resource_name,
            }
            .encode(),
            0,
        )
        .expect("Error in creating program");
        rmrk.resource_id = resource_id;
        msg::reply(RMRKReply::ResourceInited { resource_id }, 0).unwrap();
    }
    unsafe {
        RMRK = Some(rmrk);
        TX_MANAGER = Some(tx_manager);
        ASSETS = Some(assets);
    }
}

#[no_mangle]
extern "C" fn handle() {
    let action: RMRKAction = msg::load().expect("Could not load msg");
    let rmrk = unsafe { RMRK.as_mut().expect("The contract is not initialized") };

    let assets = unsafe { ASSETS.as_mut().expect("The contract is not initialized") };
    let tx_manager = unsafe { TX_MANAGER.as_mut().expect("Tx manager is not initialized") };
    match action.clone() {
        RMRKAction::MintToNft {
            parent_id,
            parent_token_id,
            token_id,
        } => {
            tx_manager_wrapper(
                rmrk,
                &action,
                tx_manager,
                RMRKToken::mint_to_nft,
                (parent_id, parent_token_id, token_id),
                RMRKReply::MintToNft {
                    parent_id,
                    parent_token_id,
                    token_id,
                },
            );
        }
        RMRKAction::MintToRootOwner {
            root_owner,
            token_id,
        } => rmrk.mint_to_root_owner(&root_owner, token_id),
        RMRKAction::Transfer { to, token_id } => {
            {
                tx_manager_wrapper(
                    rmrk,
                    &action,
                    tx_manager,
                    RMRKToken::transfer,
                    (to, token_id),
                    RMRKReply::Transfer { to, token_id },
                );
            };
        }
        RMRKAction::TransferToNft {
            to,
            destination_id,
            token_id,
        } => {
            rmrk.transfer_to_nft(tx_manager, &action, &to, destination_id, token_id);
        }
        RMRKAction::Approve { to, token_id } => {
            rmrk.approve(tx_manager, &action, &to, token_id);
        }
        RMRKAction::AddChild {
            parent_token_id,
            child_token_id,
        } => rmrk.add_child(parent_token_id, child_token_id),
        RMRKAction::AcceptChild {
            parent_token_id,
            child_contract_id,
            child_token_id,
        } => {
            let reply = rmrk.accept_child(
                tx_manager,
                &action,
                parent_token_id,
                child_contract_id,
                child_token_id,
            );
            msg::reply(reply, 0).expect("Failed to send a reply");
        }
        RMRKAction::AddAcceptedChild {
            parent_token_id,
            child_token_id,
        } => {
            rmrk.add_accepted_child(parent_token_id, child_token_id);
        }
        RMRKAction::TransferChild {
            from,
            to,
            child_token_id,
        } => {
            rmrk.transfer_child(tx_manager, &action, from, to, child_token_id);
        }
        RMRKAction::RejectChild {
            parent_token_id,
            child_contract_id,
            child_token_id,
        } => {
            rmrk.reject_child(
                tx_manager,
                &action,
                parent_token_id,
                child_contract_id,
                child_token_id,
            );
        }
        RMRKAction::RemoveChild {
            parent_token_id,
            child_contract_id,
            child_token_id,
        } => {
            rmrk.remove_child(
                tx_manager,
                &action,
                parent_token_id,
                child_contract_id,
                child_token_id,
            );
        }
        RMRKAction::BurnChild {
            parent_token_id,
            child_token_id,
        } => rmrk.burn_child(parent_token_id, child_token_id),
        RMRKAction::BurnFromParent {
            child_token_id,
            root_owner,
        } => rmrk.burn_from_parent(child_token_id, &root_owner),
        RMRKAction::Burn(token_id) => rmrk.burn(token_id),
        RMRKAction::RootOwner(token_id) => rmrk.root_owner(tx_manager, &action, token_id),
        RMRKAction::AddResourceEntry {
            resource_id,
            resource,
        } => {
            rmrk.add_resource_entry(tx_manager, &action, resource_id, resource);
        }
        RMRKAction::AddResource {
            token_id,
            resource_id,
            overwrite_id,
        } => {
            rmrk.add_resource(tx_manager, &action, token_id, resource_id, overwrite_id);
        }
        RMRKAction::AcceptResource {
            token_id,
            resource_id,
        } => {
            rmrk.accept_resource(tx_manager, &action, token_id, resource_id);
        }
        RMRKAction::RejectResource {
            token_id,
            resource_id,
        } => {
            rmrk.reject_resource(tx_manager, &action, token_id, resource_id);
        }
        RMRKAction::SetPriority {
            token_id,
            priorities,
        } => {
            rmrk.set_priority(tx_manager, &action, token_id, priorities);
        }
        RMRKAction::Equip {
            token_id,
            child_token_id,
            child_id,
            asset_id,
            slot_part_id,
            child_asset_id,
        } => {
            tx_manager_wrapper(
                assets,
                &action,
                tx_manager,
                Assets::equip,
                (
                    token_id,
                    child_token_id,
                    child_id,
                    asset_id,
                    slot_part_id,
                    child_asset_id,
                ),
                RMRKReply::ChildAssetEquipped {
                    token_id,
                    asset_id,
                    slot_part_id,
                    child_token_id,
                    child_id,
                    child_asset_id,
                },
            );
        }
        RMRKAction::CheckSlotResource {
            token_id,
            resource_id,
            base_id,
            slot_id,
        } => {
            rmrk.check_slot_resource(tx_manager, &action, token_id, resource_id, base_id, slot_id);
        }
        RMRKAction::AddEquippableAssetEntry {
            equippable_group_id,
            catalog_address,
            metadata_uri,
            part_ids,
        } => {
            let reply = assets.add_equippable_asset_entry(
                equippable_group_id,
                catalog_address,
                metadata_uri,
                part_ids,
            );
            msg::reply(reply, 0).expect("Failed to send a reply");
        }
        RMRKAction::AddAssetToToken {
            token_id,
            asset_id,
            replaces_asset_with_id,
        } => {
            let reply = assets.add_asset_to_token(token_id, asset_id, replaces_asset_with_id);
            msg::reply(reply, 0).expect("Failed to send a reply");
        }
        RMRKAction::AcceptAsset { token_id, asset_id } => {
            let reply = assets.accept_asset(token_id, asset_id);
            msg::reply(reply, 0).expect("Failed to send a reply");
        }
        RMRKAction::SetValidParentForEquippableGroup {
            equippable_group_id,
            slot_part_id,
            parent_id,
        } => {
            let reply = assets.set_valid_parent_for_equippable_group(
                equippable_group_id,
                slot_part_id,
                parent_id,
            );
            msg::reply(reply, 0).expect("Failed to send a reply");
        }
        RMRKAction::CanTokenBeEquippedWithAssetIntoSlot {
            parent_id,
            token_id,
            asset_id,
            slot_part_id,
        } => {
            let reply = assets.can_token_be_equipped_with_asset_into_slot(
                parent_id,
                token_id,
                asset_id,
                slot_part_id,
            );
            msg::reply(reply, 0).expect("Failed to send a reply");
        }
    }
}

#[no_mangle]
extern "C" fn state() {
    let rmrk = unsafe { RMRK.as_ref().expect("RMRK is not initialized") };
    let assets = unsafe { ASSETS.as_ref().expect("ASSETS is not initialized") };
    let mut rmrk_state: RMRKState = rmrk.into();
    let assets_state: AssetsState = assets.into();
    rmrk_state.assets = assets_state;
    msg::reply(rmrk_state, 0).expect("Failed to share state");
}

#[no_mangle]
extern "C" fn metahash() {
    let metahash: [u8; 32] = include!("../.metahash");
    msg::reply(metahash, 0).expect("Failed to share metahash");
}

#[no_mangle]
extern "C" fn handle_reply() {
    let reply_to = msg::reply_to().expect("Failed to get the reply details");
    let tx_manager = unsafe { TX_MANAGER.as_mut().expect("Tx manager is not initialized") };
    let processing_msg_id = tx_manager
        .msg_sent_to_msg
        .remove(&reply_to)
        .expect("Receive reply on undefined message");
    let tx = tx_manager
        .txs
        .get_mut(&processing_msg_id)
        .expect("Message does not exist");
    let action = tx.msg.clone();
    match action {
        RMRKAction::MintToNft { .. } => mint_to_nft_reply(tx, processing_msg_id),
        RMRKAction::Transfer { .. } => transfer_reply(tx, processing_msg_id),
        RMRKAction::TransferToNft { .. } => transfer_to_nft_reply(tx, processing_msg_id),
        RMRKAction::AcceptChild { .. } => accept_child_reply(tx, processing_msg_id),
        RMRKAction::RejectChild { .. } => reject_child_reply(tx, processing_msg_id),
        RMRKAction::RemoveChild { .. } => remove_child_reply(tx, processing_msg_id),
        RMRKAction::TransferChild { .. } => transfer_child_reply(tx, processing_msg_id),
        RMRKAction::AddResourceEntry { .. } => add_resource_entry_reply(tx, processing_msg_id),
        RMRKAction::AddResource { .. } => add_resource_reply(tx, processing_msg_id),
        RMRKAction::AcceptResource { .. } => accept_resource_reply(tx, processing_msg_id),
        RMRKAction::RejectResource { .. } => reject_resource_reply(tx, processing_msg_id),
        RMRKAction::SetPriority { .. } => set_priority_reply(tx, processing_msg_id),
        RMRKAction::RootOwner(_) => get_root_owner_reply(tx, processing_msg_id),
        RMRKAction::Equip { .. } => equip_reply(tx, processing_msg_id),
        _ => {}
    }
}

pub fn decode_root_owner(reply: Vec<u8>) -> ActorId {
    let decoded_reply = RMRKReply::decode(&mut &reply[..]).expect("Failed to decode a reply");
    if let RMRKReply::RootOwner(root_owner) = decoded_reply {
        root_owner
    } else {
        panic!("Wrong received reply");
    }
}

pub fn root_owner_from_data(mut reply: &[u8]) -> ActorId {
    ActorId::decode(&mut reply).expect("Unable to decode ActorId")
}

pub fn get_root_owner_reply(tx: &mut Tx, processing_msg_id: MessageId) {
    let state = tx.state.clone();
    match state {
        TxState::MsgGetRootOwnerSent => {
            tx.state = TxState::ReplyRootOwnerReceived;
            tx.data = Some(msg::load_bytes().expect("Failed to load the payload"));
            exec::wake(processing_msg_id).expect("Failed to wake the message");
        }
        _ => {
            unreachable!()
        }
    }
}

fn tx_manager_wrapper<T, A, B>(
    rmrk_token: &mut A,
    action: &RMRKAction,
    tx_manager: &mut TxManager,
    mut f: T,
    args: B,
    success_reply: RMRKReply,
) where
    T: FnMut(&mut A, TxState, Option<Vec<u8>>, B) -> Result<(TxState, MessageId), RMRKError>,
    A: Default,
    B: Encode,
{
    let tx = tx_manager.get_tx(&action);
    let state = tx.state.clone();
    let tx_data = tx.data.clone();
    let result = f(rmrk_token, state, tx_data, args);
    tx.data = None;
    let reply = match result {
        Ok((tx_state, msg_id)) => {
            tx.state = tx_state.clone();
            if tx_state == TxState::Completed {
                Ok(success_reply)
            } else {
                tx_manager.msg_sent_to_msg.insert(msg_id, msg::id());
                exec::wait_for(5);
            }
        }
        Err(error) => Err(error),
    };
    msg::reply(reply, 0).expect("Failed to send a reply");
}
