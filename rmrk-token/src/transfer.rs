use crate::*;
use gstd::{msg, ActorId};

impl RMRKToken {
    /// Transfers NFT to another account.
    /// If the previous owner is another RMRK contract, it sends the message [`RMRKAction::BurnChild`] to the parent conract.
    ///
    /// # Requirements:
    /// * The `token_id` must exist.
    /// * The `msg::source()` must be approved or owner of the token.
    /// * The `to` address should be a non-zero address.
    ///
    /// # Arguments:
    /// * `to`: is the receiving address.
    /// * `token_id`: is the tokenId of the transfered token.
    ///
    /// On success replies [`RMRKEvent::ChildBurnt`].
    pub fn transfer(
        &mut self,
        state: TxState,
        tx_data: Option<Vec<u8>>,
        args: (ActorId, TokenId),
    ) -> Result<(TxState, MessageId), RMRKError> {
        let (to, token_id) = args;
        let rmrk_owner = self
            .rmrk_owners
            .get(&token_id)
            .expect("RMRK: Token does not exist");

        match state {
            TxState::Initial => {
                self.assert_zero_address(&to);

                if let Some(parent_token_id) = rmrk_owner.token_id {
                    let msg_id = get_root_owner_msg(&rmrk_owner.owner_id, parent_token_id);
                    return Ok((TxState::MsgGetRootOwnerSent, msg_id));
                } else {
                    let root_owner = rmrk_owner.owner_id;
                    self.assert_approved_or_owner(token_id, &root_owner);
                    self.decrease_balance(&root_owner);
                    self.internal_mint(&to, token_id, &to, None);
                    return Ok((TxState::Completed, MessageId::zero()));
                };
            }
            TxState::ReplyRootOwnerReceived => {
                let root_owner = decode_root_owner(tx_data.expect("Can't be None"));
                self.assert_approved_or_owner(token_id, &root_owner);
                let parent_token_id = rmrk_owner.token_id.expect("Can't be None");
                let msg_id = burn_child_msg(&rmrk_owner.owner_id, parent_token_id, token_id);
                return Ok((TxState::MsgBurnChildSent, msg_id));
            }
            TxState::ReplyOnBurnChildReceived => {
                let root_owner = decode_root_owner(tx_data.expect("Can't be None"));
                self.decrease_balance(&root_owner);
                self.internal_mint(&to, token_id, &to, None);
                self.decrease_balance(&root_owner);
                self.internal_mint(&to, token_id, &to, None);
                return Ok((TxState::Completed, MessageId::zero()));
            }
            _ => {
                unreachable!()
            }
        };
    }

    /// Transfers NFT to another NFT.
    ///
    /// # Requirements:
    /// * The `token_id` must exist.
    /// * The `msg::source()` must be approved or root owner of the token.
    /// * The `to` address should be a non-zero address
    ///
    /// # Arguments:
    /// * `to`: is the address of new parent RMRK contract.
    /// * `destination_id: is the tokenId of the parent RMRK token.
    /// * `token_id`: is the tokenId of the transfered token.
    ///
    /// On success replies [`RMRKEvent::TransferToNft`].
    pub fn transfer_to_nft(
        &mut self,
        tx_manager: &mut TxManager,
        msg: &RMRKAction,
        to: &ActorId,
        destination_id: TokenId,
        token_id: TokenId,
    ) -> Result<RMRKReply, RMRKError> {
        self.assert_zero_address(to);

        let tx = tx_manager.get_tx(msg);
        let state = tx.state.clone();
        let current_msg_id = msg::id();

        let rmrk_owner = self
            .rmrk_owners
            .get(&token_id)
            .expect("RMRK: Token does not exist");

        match state {
            TxState::Initial if rmrk_owner.token_id.is_some() => {
                let parent_token_id = rmrk_owner.token_id.expect("Can't be None");
                // get root owner
                let msg_id = get_root_owner_msg(&rmrk_owner.owner_id, parent_token_id);
                tx.state = TxState::MsgGetRootOwnerSent;
                tx_manager.msg_sent_to_msg.insert(msg_id, current_msg_id);
                exec::wait_for(5);
            }
            TxState::Initial if rmrk_owner.token_id.is_none() => {
                let root_owner = rmrk_owner.owner_id;
                self.assert_approved_or_owner(token_id, &root_owner);
                // get new root owner
                let msg_id = get_root_owner_msg(to, destination_id);
                tx.state = TxState::MsgGetNewRootOwnerSent;
                tx_manager.msg_sent_to_msg.insert(msg_id, current_msg_id);
                exec::wait_for(5);
            }
            TxState::ReplyRootOwnerReceived => {
                let reply = tx.data.take().expect("Failed to get a reply");
                let decoded_reply =
                    RMRKReply::decode(&mut &reply[..]).expect("Failed to decode a reply");
                let root_owner = if let RMRKReply::RootOwner(root_owner) = decoded_reply {
                    root_owner
                } else {
                    panic!("Wrong received reply");
                };
                self.assert_approved_or_owner(token_id, &root_owner);
                // get new root owner
                let msg_id = get_root_owner_msg(to, destination_id);
                tx.state = TxState::MsgGetNewRootOwnerSent;
                tx.data = Some(root_owner.encode());
                tx_manager.msg_sent_to_msg.insert(msg_id, current_msg_id);
                exec::wait_for(5);
            }
            TxState::ReplyNewRootOwnerReceived => {
                let reply = tx.data.take().expect("Failed to get a reply");
                let decoded_reply =
                    RMRKReply::decode(&mut &reply[..]).expect("Failed to decode a reply");
                let new_root_owner = if let RMRKReply::RootOwner(root_owner) = decoded_reply {
                    root_owner
                } else {
                    panic!("Wrong received reply");
                };
                let data = tx.data.clone().expect("msg");
                let root_owner = ActorId::decode(&mut &data[..]).expect("Failed to get root owner");
                tx.data = Some((root_owner, new_root_owner).encode());
                // if root owner transfers child RMRK token between RMRK tokens inside the same RMRK contract
                if rmrk_owner.owner_id == *to {
                    let msg_id = transfer_child_msg(
                        to,
                        rmrk_owner.token_id.expect("Cant be None"),
                        destination_id,
                        token_id,
                    );
                    tx.state = TxState::MsgTransferChildSent;
                    tx_manager.msg_sent_to_msg.insert(msg_id, current_msg_id);
                    exec::wait_for(5);
                }

                if let Some(parent_token_id) = rmrk_owner.token_id {
                    let msg_id = burn_child_msg(&rmrk_owner.owner_id, parent_token_id, token_id);
                    tx.state = TxState::MsgBurnChildSent;
                    tx_manager.msg_sent_to_msg.insert(msg_id, current_msg_id);
                    exec::wait_for(5);
                }
                let data = tx.data.clone().expect("msg");
                let (root_owner, new_root_owner) = <(ActorId, ActorId)>::decode(&mut &data[..])
                    .expect("Failed to get root owners");

                if root_owner == new_root_owner {
                    let msg_id = add_accepted_child_msg(to, destination_id, token_id);
                    tx.state = TxState::MsgAddAcceptedChildSent;
                    tx_manager.msg_sent_to_msg.insert(msg_id, current_msg_id);
                    exec::wait_for(5);
                }
                let msg_id = add_child_msg(to, destination_id, token_id);
                tx.state = TxState::MsgAddChildSent;
                tx_manager.msg_sent_to_msg.insert(msg_id, current_msg_id);
                exec::wait_for(5);
            }
            TxState::ReplyOnBurnChildReceived => {
                let data = tx.data.clone().expect("msg");
                let (root_owner, new_root_owner) = <(ActorId, ActorId)>::decode(&mut &data[..])
                    .expect("Failed to get root owners");

                if root_owner == new_root_owner {
                    let msg_id = add_accepted_child_msg(to, destination_id, token_id);
                    tx.state = TxState::MsgAddAcceptedChildSent;
                    tx_manager.msg_sent_to_msg.insert(msg_id, current_msg_id);
                    exec::wait_for(5);
                }
                let msg_id = add_child_msg(to, destination_id, token_id);
                tx.state = TxState::MsgAddChildSent;
                tx_manager.msg_sent_to_msg.insert(msg_id, current_msg_id);
                exec::wait_for(5);
            }
            TxState::ReplyOnAddAcceptedChildReceived
            | TxState::ReplyAddChildReceived
            | TxState::ReplyOnTransferChildReceived => {
                let data = tx.data.clone().expect("msg");
                let (root_owner, new_root_owner) = <(ActorId, ActorId)>::decode(&mut &data[..])
                    .expect("Failed to get root owners");
                let mut new_rmrk_owner: RMRKOwner = Default::default();
                new_rmrk_owner.owner_id = *to;
                new_rmrk_owner.token_id = Some(destination_id);
                self.rmrk_owners.insert(token_id, new_rmrk_owner);
                self.increase_balance(&new_root_owner);
                self.decrease_balance(&root_owner);
                return Ok(RMRKReply::TransferToNft {
                    to: *to,
                    token_id,
                    destination_id,
                });
            }
            _ => {
                unreachable!()
            }
        }
    }

    /// Approves an account to transfer NFT.
    ///
    /// # Requirements:
    /// * The `token_id` must exist.
    /// * The `msg::source()` must be approved or root owner of the token.
    /// * The `to` address must be a non-zero address
    ///
    /// # Arguments:
    /// * `to`: is the address of approved account.
    /// * `token_id`: is the tokenId of the token.
    ///
    /// On success replies [`RMRKEvent::Approval`].
    pub fn approve(
        &mut self,
        tx_manager: &mut TxManager,
        msg: &RMRKAction,
        to: &ActorId,
        token_id: TokenId,
    ) -> Result<RMRKReply, RMRKError> {
        self.assert_zero_address(to);

        let rmrk_owner = self
            .rmrk_owners
            .get(&token_id)
            .expect("RMRK: Token does not exist");

        // if the NFT owner is another NFT
        let parent_token_id = if let Some(parent_token_id) = rmrk_owner.token_id {
            parent_token_id
        } else {
            let root_owner = rmrk_owner.owner_id;
            self.assert_owner(&root_owner);
            self.token_approvals
                .entry(token_id)
                .and_modify(|approvals| {
                    approvals.insert(*to);
                })
                .or_insert_with(|| HashSet::from([*to]));
            return Ok(RMRKReply::Approval {
                root_owner,
                approved_account: *to,
                token_id,
            });
        };

        let tx = tx_manager.get_tx(msg);
        let state = tx.state.clone();
        let current_msg_id = msg::id();

        match state {
            TxState::Initial => {
                let msg_id = get_root_owner_msg(&rmrk_owner.owner_id, parent_token_id);
                tx.state = TxState::MsgGetRootOwnerSent;
                tx_manager.msg_sent_to_msg.insert(msg_id, current_msg_id);
                exec::wait_for(5);
            }
            TxState::ReplyRootOwnerReceived => {
                let reply = tx.data.take().expect("Failed to get a reply");
                let decoded_reply =
                    RMRKReply::decode(&mut &reply[..]).expect("Failed to decode a reply");
                let root_owner = if let RMRKReply::RootOwner(root_owner) = decoded_reply {
                    root_owner
                } else {
                    panic!("Wrong received reply");
                };
                self.assert_owner(&root_owner);
                self.token_approvals
                    .entry(token_id)
                    .and_modify(|approvals| {
                        approvals.insert(*to);
                    })
                    .or_insert_with(|| HashSet::from([*to]));
                return Ok(RMRKReply::Approval {
                    root_owner,
                    approved_account: *to,
                    token_id,
                });
            }
            _ => {
                unreachable!()
            }
        }
    }
}

pub fn transfer_reply(tx: &mut Tx, processing_msg_id: MessageId) {
    let state = tx.state.clone();
    match state {
        TxState::MsgGetRootOwnerSent => {
            tx.state = TxState::ReplyRootOwnerReceived;
            tx.data = Some(msg::load_bytes().expect("Failed to load the payload"));
            exec::wake(processing_msg_id).expect("Failed to wake the message");
        }
        TxState::MsgBurnChildSent => {
            tx.state = TxState::ReplyOnBurnChildReceived;
            exec::wake(processing_msg_id).expect("Failed to wake the message");
        }
        _ => {
            unreachable!()
        }
    }
}

pub fn transfer_to_nft_reply(tx: &mut Tx, processing_msg_id: MessageId) {
    let state = tx.state.clone();
    match state {
        TxState::MsgGetRootOwnerSent => {
            tx.state = TxState::ReplyRootOwnerReceived;
            tx.data = Some(msg::load_bytes().expect("Failed to load the payload"));
            exec::wake(processing_msg_id).expect("Failed to wake the message");
        }
        TxState::MsgGetNewRootOwnerSent => {
            tx.state = TxState::ReplyNewRootOwnerReceived;
            tx.data = Some(msg::load_bytes().expect("Failed to load the payload"));
            exec::wake(processing_msg_id).expect("Failed to wake the message");
        }
        TxState::MsgTransferChildSent => {
            tx.state = TxState::ReplyOnTransferChildReceived;
            exec::wake(processing_msg_id).expect("Failed to wake the message");
        }
        TxState::MsgAddAcceptedChildSent => {
            tx.state = TxState::ReplyOnAddAcceptedChildReceived;
            exec::wake(processing_msg_id).expect("Failed to wake the message");
        }
        TxState::MsgAddChildSent => {
            tx.state = TxState::ReplyAddChildReceived;
            exec::wake(processing_msg_id).expect("Failed to wake the message");
        }
        _ => {
            unreachable!()
        }
    }
}

pub fn approve_reply(tx: &mut Tx, processing_msg_id: MessageId) {
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
