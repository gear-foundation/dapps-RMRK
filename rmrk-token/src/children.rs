use crate::*;
use gstd::msg;

impl RMRKToken {
    /// That message is designed to be send from another RMRK contracts
    /// when minting an NFT(child_token_id) to another NFT(parent_token_id).
    /// It adds a child to the NFT with tokenId `parent_token_id`
    /// The status of added child is `Pending`.
    ///
    /// # Requirements:
    /// * Token with TokenId `parent_token_id` must exist.
    /// * There cannot be two identical children.
    ///
    /// # Arguments:
    /// * `parent_token_id`: is the tokenId of the parent NFT.
    /// * `child_token_id`: is the tokenId of the child instance.
    ///
    /// On success replies [`RMRKEvent::PendingChild`].
    pub fn add_child(&mut self, parent_token_id: TokenId, child_token_id: TokenId) {
        self.assert_token_does_not_exist(parent_token_id);

        let child_token = (msg::source(), child_token_id);

        // check if the child already exists in pending array
        if let Some(children) = self.pending_children.get(&parent_token_id) {
            // if child already exists
            if children.contains(&child_token) {
                panic!("RMRKCore: child already exists in pending array");
            }
        }

        // add child to pending children array
        self.internal_add_child(parent_token_id, child_token, ChildStatus::Pending);

        msg::reply(
            RMRKReply::PendingChild {
                child_token_address: msg::source(),
                child_token_id,
                parent_token_id,
            },
            0,
        )
        .expect("Error in reply [RMRKEvent::PendingChild]");
    }

    /// Accepts an RMRK child being in the `Pending` status.
    /// Removes RMRK child from `pending_children` and adds to `accepted_children`.
    ///
    /// # Requirements:
    /// * The `msg::source()` must be an RMRK owner of NFT with tokenId `parent_token_id` or an approved account.
    /// * The indicated NFT with tokenId `child_token_id` must exist in the pending array of `parent_token_id`.
    ///
    /// # Arguments:
    /// * `parent_token_id`: is the tokenId of the parent NFT
    /// * `child_token_id`: is the tokenId of the child instance
    ///
    /// On success replies [`RMRKEvent::AcceptedChild`].
    pub fn accept_child(
        &mut self,
        tx_manager: &mut TxManager,
        msg: &RMRKAction,
        parent_token_id: TokenId,
        child_contract_id: ActorId,
        child_token_id: TokenId,
    ) -> Result<RMRKReply, RMRKError> {
        let rmrk_owner = self
            .rmrk_owners
            .get(&parent_token_id)
            .expect("RMRK: Token does not exist");

        // if the NFT owner is another NFT
        let owner_token_id = if let Some(owner_token_id) = rmrk_owner.token_id {
            owner_token_id
        } else {
            let root_owner = rmrk_owner.owner_id;
            self.assert_approved_or_owner(parent_token_id, &root_owner);
            let child_token = (child_contract_id, child_token_id);

            // remove child from pending array
            self.internal_remove_child(parent_token_id, child_token, ChildStatus::Pending);

            // add child to accepted children array
            self.internal_add_child(parent_token_id, child_token, ChildStatus::Accepted);

            return Ok(RMRKReply::AcceptedChild {
                child_contract_id,
                child_token_id,
                parent_token_id,
            });
        };

        let tx = tx_manager.get_tx(msg);
        let state = tx.state.clone();
        let current_msg_id = msg::id();

        match state {
            TxState::Initial => {
                let msg_id = get_root_owner_msg(&rmrk_owner.owner_id, owner_token_id);
                tx.state = TxState::MsgGetRootOwnerSent;
                tx_manager.msg_sent_to_msg.insert(msg_id, current_msg_id);
                exec::wait_for(5);
            }
            TxState::ReplyRootOwnerReceived => {
                let reply = tx.data.take().expect("Failed to get a reply");
                let root_owner = decode_root_owner(reply);
                self.assert_approved_or_owner(parent_token_id, &root_owner);
                let child_token = (child_contract_id, child_token_id);

                // remove child from pending array
                self.internal_remove_child(parent_token_id, child_token, ChildStatus::Pending);

                // add child to accepted children array
                self.internal_add_child(parent_token_id, child_token, ChildStatus::Accepted);

                return Ok(RMRKReply::AcceptedChild {
                    child_contract_id,
                    child_token_id,
                    parent_token_id,
                });
            }
            _ => {
                unreachable!()
            }
        }
    }

    /// Rejects an RMRK child being in the `Pending` status.
    /// It sends message to the child NFT contract to burn NFT token from it.
    ///
    /// # Requirements:
    /// * The `msg::source()` must be an RMRK owner or an approved account.
    /// * The indicated NFT with tokenId `child_token_id` must exist in the pending array of `parent_token_id`.
    ///
    /// Arguments:
    /// * `parent_token_id`: is the tokenId of the parent NFT.
    /// * `child_contract_id`: is the address of the child RMRK contract.
    /// * `child_token_id`: is the tokenId of the child instance.
    ///
    /// On success replies [`RMRKEvent::RejectedChild`].
    pub fn reject_child(
        &mut self,
        tx_manager: &mut TxManager,
        msg: &RMRKAction,
        parent_token_id: TokenId,
        child_contract_id: ActorId,
        child_token_id: TokenId,
    ) -> Result<RMRKReply, RMRKError> {
        let rmrk_owner = self
            .rmrk_owners
            .get(&parent_token_id)
            .expect("RMRK: Token does not exist");

        let tx = tx_manager.get_tx(msg);
        let state = tx.state.clone();
        let current_msg_id = msg::id();

        match state {
            TxState::Initial if rmrk_owner.token_id.is_some() => {
                let owner_token_id = rmrk_owner.token_id.expect("Can't be None");
                // get root owner
                let msg_id = get_root_owner_msg(&rmrk_owner.owner_id, owner_token_id);
                tx.state = TxState::MsgGetRootOwnerSent;
                tx_manager.msg_sent_to_msg.insert(msg_id, current_msg_id);
                exec::wait_for(5);
            }
            TxState::Initial if rmrk_owner.token_id.is_none() => {
                let root_owner = rmrk_owner.owner_id;
                self.assert_approved_or_owner(parent_token_id, &root_owner);

                let msg_id = burn_from_parent_msg(&child_contract_id, child_token_id, &root_owner);
                tx.state = TxState::MsgBurnFromParentSent;
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
                self.assert_approved_or_owner(parent_token_id, &root_owner);
                let msg_id = burn_from_parent_msg(&child_contract_id, child_token_id, &root_owner);
                tx.state = TxState::MsgBurnFromParentSent;
                tx_manager.msg_sent_to_msg.insert(msg_id, current_msg_id);
                exec::wait_for(5);
            }
            TxState::ReplyOnBurnFromParentReceived => {
                // remove child from pending array
                let child_token = (child_contract_id, child_token_id);
                self.internal_remove_child(parent_token_id, child_token, ChildStatus::Pending);
                return Ok(RMRKReply::RejectedChild {
                    child_contract_id,
                    child_token_id,
                    parent_token_id,
                });
            }
            _ => {
                unreachable!()
            }
        }
    }

    /// Removes an RMRK child being in the `Accepted` status.
    /// It sends message to the child NFT contract to burn NFT token from it.
    ///
    /// # Requirements:
    /// * The `msg::source()` must be an RMRK owner or an approved account.
    ///
    /// # Arguments:
    /// * `parent_token_id`: is the tokenId of the parent NFT.
    /// * `child_contract_id`: is the address of the child RMRK contract.
    /// * `child_token_id`: is the tokenId of the child instance.
    ///
    /// On success replies [`RMRKEvent::RemovedChild`].
    pub fn remove_child(
        &mut self,
        tx_manager: &mut TxManager,
        msg: &RMRKAction,
        parent_token_id: TokenId,
        child_contract_id: ActorId,
        child_token_id: TokenId,
    ) -> Result<RMRKReply, RMRKError> {
        let rmrk_owner = self
            .rmrk_owners
            .get(&parent_token_id)
            .expect("RMRK: Token does not exist");

        let tx = tx_manager.get_tx(msg);
        let state = tx.state.clone();
        let current_msg_id = msg::id();

        match state {
            TxState::Initial if rmrk_owner.token_id.is_some() => {
                let owner_token_id = rmrk_owner.token_id.expect("Can't be None");
                // get root owner
                let msg_id = get_root_owner_msg(&rmrk_owner.owner_id, owner_token_id);
                tx.state = TxState::MsgGetRootOwnerSent;
                tx_manager.msg_sent_to_msg.insert(msg_id, current_msg_id);
                exec::wait_for(5);
            }
            TxState::Initial if rmrk_owner.token_id.is_none() => {
                let root_owner = rmrk_owner.owner_id;
                self.assert_approved_or_owner(parent_token_id, &root_owner);

                let msg_id = burn_from_parent_msg(&child_contract_id, child_token_id, &root_owner);
                tx.state = TxState::MsgBurnFromParentSent;
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
                self.assert_approved_or_owner(parent_token_id, &root_owner);
                let msg_id = burn_from_parent_msg(&child_contract_id, child_token_id, &root_owner);
                tx.state = TxState::MsgBurnFromParentSent;
                tx_manager.msg_sent_to_msg.insert(msg_id, current_msg_id);
                exec::wait_for(5);
            }
            TxState::ReplyOnBurnFromParentReceived => {
                // remove child from pending array
                let child_token = (child_contract_id, child_token_id);
                self.internal_remove_child(parent_token_id, child_token, ChildStatus::Accepted);
                return Ok(RMRKReply::RemovedChild {
                    child_contract_id,
                    child_token_id,
                    parent_token_id,
                });
            }
            _ => {
                unreachable!()
            }
        }
    }

    /// That message is designed to be sent from another RMRK contracts
    /// when root owner transfers his child to another parent token within one contract.
    /// If root owner transfers child token from NFT to another his NFT
    /// it adds a child to the NFT  with a status that child had before.
    /// If root owner transfers child token from NFT to another NFT that he does not own
    /// it adds a child to the NFT  with a status `Pending`.
    ///
    /// # Requirements:
    /// * The `msg::source()` must be a child RMRK contract.
    /// * The `to` must be an existing RMRK token
    ///
    /// # Arguments:
    /// * `from`: RMRK token from which the child token will be transferred.
    /// * `to`: RMRK token to which the child token will be transferred.
    /// * `child_token_id`: is the tokenId of the child in the RMRK child contract.
    ///
    /// On success replies [`RMRKEvent::ChildTransferred`].
    pub fn transfer_child(
        &mut self,
        tx_manager: &mut TxManager,
        msg: &RMRKAction,
        from: TokenId,
        to: TokenId,
        child_token_id: TokenId,
    ) {
        self.assert_token_does_not_exist(to);

        let child_token = (msg::source(), child_token_id);

        // check the status of the child
        let child_status = self
            .children_status
            .get(&child_token)
            .expect("RMRK: The child does not exist");

        let from_rmrk_owner = self
            .rmrk_owners
            .get(&from)
            .expect("RMRK: Token does not exist");

        let to_rmrk_owner = self
            .rmrk_owners
            .get(&to)
            .expect("RMRK: Token does not exist");

        let tx = tx_manager.get_tx(msg);
        let state = tx.state.clone();
        let current_msg_id = msg::id();

        let (from_root_owner, to_root_owner) = match state {
            TxState::Initial => match (from_rmrk_owner.token_id, to_rmrk_owner.token_id) {
                (None, None) => (from_rmrk_owner.owner_id, to_rmrk_owner.owner_id),
                (Some(parent_token_id), None) | (Some(parent_token_id), Some(_)) => {
                    let msg_id = get_root_owner_msg(&from_rmrk_owner.owner_id, parent_token_id);
                    tx.state = TxState::MsgGetRootOwnerSent;
                    tx_manager.msg_sent_to_msg.insert(msg_id, current_msg_id);
                    exec::wait_for(5);
                }
                (None, Some(parent_token_id)) => {
                    let msg_id = get_root_owner_msg(&to_rmrk_owner.owner_id, parent_token_id);
                    tx.state = TxState::MsgGetNewRootOwnerSent;
                    tx.data = Some(from_rmrk_owner.owner_id.encode());
                    tx_manager.msg_sent_to_msg.insert(msg_id, current_msg_id);
                    exec::wait_for(5);
                }
            },
            TxState::ReplyRootOwnerReceived => {
                match (from_rmrk_owner.token_id, to_rmrk_owner.token_id) {
                    (Some(_), Some(parent_token_id)) => {
                        let msg_id = get_root_owner_msg(&to_rmrk_owner.owner_id, parent_token_id);
                        tx.state = TxState::MsgGetNewRootOwnerSent;
                        tx_manager.msg_sent_to_msg.insert(msg_id, current_msg_id);
                        exec::wait_for(5);
                    }
                    (Some(_), None) => {
                        let data = tx.data.clone().expect("msg");

                        let from_root_owner = root_owner_from_data(&data[..]);
                        (from_root_owner, to_rmrk_owner.owner_id)
                    }
                    _ => {
                        unreachable!()
                    }
                }
            }
            TxState::ReplyNewRootOwnerReceived => {
                let reply = tx.data.take().expect("Failed to get a reply");
                let to_root_owner = decode_root_owner(reply);

                match (from_rmrk_owner.token_id, to_rmrk_owner.token_id) {
                    (Some(_), Some(_)) => {
                        let data = tx.data.clone().expect("msg");

                        let from_root_owner =
                            ActorId::decode(&mut &data[..]).expect("Failed to decode ActorId");
                        (from_root_owner, to_root_owner)
                    }
                    (None, Some(_)) => (from_rmrk_owner.owner_id, to_root_owner),
                    _ => {
                        unreachable!()
                    }
                }
            }
            _ => {
                unreachable!()
            }
        };

        self.assert_exec_origin(&from_root_owner);
        match child_status {
            ChildStatus::Pending => {
                self.internal_remove_child(from, child_token, ChildStatus::Pending);
                self.internal_add_child(to, child_token, ChildStatus::Pending);
            }
            ChildStatus::Accepted => {
                self.internal_remove_child(from, child_token, ChildStatus::Accepted);
                if from_root_owner == to_root_owner {
                    self.internal_add_child(to, child_token, ChildStatus::Accepted);
                } else {
                    self.internal_add_child(to, child_token, ChildStatus::Pending);
                }
            }
        }
        msg::reply(
            RMRKReply::ChildTransferred {
                from,
                to,
                child_contract_id: msg::source(),
                child_token_id,
            },
            0,
        )
        .expect("Error in reply `[RMRKEvent::ChildTransferred]`");
    }

    /// That function is designed to be called from another RMRK contracts
    /// when root owner transfers his child NFT to another his NFT in another contract.
    /// It adds a child to the RMRK token with tokenId `parent_token_id` with status `Accepted`.
    ///
    /// # Requirements:
    /// * The `msg::source()` must be a child RMRK contract.
    /// * The `parent_token_id` must be an existing RMRK token that must have `child_token_id` in its `accepted_children`.
    ///
    /// # Arguments:
    /// * `parent_token_id`: RMRK token to which the child token will be transferred.
    /// * `child_token_id`: is the tokenId of the child of the RMRK child contract.
    ///
    /// On success replies [`RMRKEvent::AcceptedChild`].
    pub fn add_accepted_child(&mut self, parent_token_id: TokenId, child_token_id: TokenId) {
        // let root_owner = self.find_root_owner(parent_token_id).await;
        // self.assert_exec_origin(&root_owner);

        // let child_token = (msg::source(), child_token_id);

        // self.internal_add_child(parent_token_id, child_token, ChildStatus::Accepted);

        // msg::reply(
        //     RMRKReply::AcceptedChild {
        //         child_contract_id: msg::source(),
        //         child_token_id,
        //         parent_token_id,
        //     },
        //     0,
        // )
        // .expect("Error in reply `[RMRKEvent::AcceptedChild]`");
    }

    /// Burns a child of NFT.
    /// That function must be called from the child RMRK contract during `transfer`, `transfer_to_nft` and `burn` functions.
    ///
    /// # Requirements:
    /// * The `msg::source()` must be a child RMRK contract.
    /// * The indicated child must exist the children list of `parent_token_id`.
    ///
    /// # Arguments:
    /// * `parent_token_id`: is the tokenId of the parent NFT.
    /// * `child_token_id`: is the tokenId of the child instance.
    ///
    /// On success replies [`RMRKEvent::ChildBurnt`].
    pub fn burn_child(&mut self, parent_token_id: TokenId, child_token_id: TokenId) {
        let child_token = (msg::source(), child_token_id);
        let child_status = self
            .children_status
            .remove(&child_token)
            .expect("Child does not exist");

        self.internal_remove_child(parent_token_id, child_token, child_status);

        msg::reply(
            RMRKReply::ChildBurnt {
                parent_token_id,
                child_token_id,
            },
            0,
        )
        .expect("Error in reply [`RMRKEvent::ChildBurnt`]");
    }

    fn internal_add_child(
        &mut self,
        parent_token_id: TokenId,
        child_token: CollectionAndToken,
        child_status: ChildStatus,
    ) {
        match child_status {
            ChildStatus::Pending => {
                self.pending_children
                    .entry(parent_token_id)
                    .and_modify(|children| {
                        children.insert(child_token);
                    })
                    .or_insert_with(|| HashSet::from([child_token]));

                self.children_status
                    .insert(child_token, ChildStatus::Pending);
            }
            ChildStatus::Accepted => {
                self.accepted_children
                    .entry(parent_token_id)
                    .and_modify(|children| {
                        children.insert(child_token);
                    })
                    .or_insert_with(|| HashSet::from([child_token]));

                self.children_status
                    .insert(child_token, ChildStatus::Accepted);
            }
        }
    }

    fn internal_remove_child(
        &mut self,
        parent_token_id: TokenId,
        child_token: CollectionAndToken,
        child_status: ChildStatus,
    ) {
        self.children_status.remove(&child_token);
        match child_status {
            ChildStatus::Pending => {
                if let Some(children) = self.pending_children.get_mut(&parent_token_id) {
                    if !children.remove(&child_token) {
                        panic!("RMRK: child does not exist in pending array or has already been accepted");
                    }
                } else {
                    panic!("RMRK: there are no pending children at all");
                }
            }
            ChildStatus::Accepted => {
                if let Some(children) = self.accepted_children.get_mut(&parent_token_id) {
                    if children.contains(&child_token) {
                        children.remove(&child_token);
                    } else {
                        panic!("RMRK: child does not exist");
                    }
                }
            }
        }
    }
}

pub fn accept_child_reply(tx: &mut Tx, processing_msg_id: MessageId) {
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

pub fn reject_child_reply(tx: &mut Tx, processing_msg_id: MessageId) {
    let state = tx.state.clone();
    match state {
        TxState::MsgGetRootOwnerSent => {
            tx.state = TxState::ReplyRootOwnerReceived;
            tx.data = Some(msg::load_bytes().expect("Failed to load the payload"));
            exec::wake(processing_msg_id).expect("Failed to wake the message");
        }
        TxState::MsgBurnFromParentSent => {
            tx.state = TxState::ReplyOnBurnFromParentReceived;
            exec::wake(processing_msg_id).expect("Failed to wake the message");
        }
        _ => {
            unreachable!()
        }
    }
}

pub fn remove_child_reply(tx: &mut Tx, processing_msg_id: MessageId) {
    let state = tx.state.clone();
    match state {
        TxState::MsgGetRootOwnerSent => {
            tx.state = TxState::ReplyRootOwnerReceived;
            tx.data = Some(msg::load_bytes().expect("Failed to load the payload"));
            exec::wake(processing_msg_id).expect("Failed to wake the message");
        }
        TxState::MsgBurnFromParentSent => {
            tx.state = TxState::ReplyOnBurnFromParentReceived;
            exec::wake(processing_msg_id).expect("Failed to wake the message");
        }
        _ => {
            unreachable!()
        }
    }
}

pub fn transfer_child_reply(tx: &mut Tx, processing_msg_id: MessageId) {
    let state = tx.state.clone();
    match state {
        TxState::MsgGetRootOwnerSent => {
            tx.state = TxState::ReplyRootOwnerReceived;
            let reply = msg::load_bytes().expect("Failed to load the payload");
            let root_owner = decode_root_owner(reply);
            tx.data = Some(root_owner.encode());
            exec::wake(processing_msg_id).expect("Failed to wake the message");
        }
        TxState::MsgGetNewRootOwnerSent => {
            tx.state = TxState::ReplyRootOwnerReceived;
            tx.data = Some(msg::load_bytes().expect("Failed to load the payload"));
            tx.state = TxState::ReplyOnBurnFromParentReceived;
            exec::wake(processing_msg_id).expect("Failed to wake the message");
        }
        _ => {
            unreachable!()
        }
    }
}
