use crate::*;
use gstd::{debug, msg, ActorId};

impl RMRKToken {
    /// Mints token that will belong to another token in another RMRK contract.
    ///
    /// # Requirements:
    /// * The `parent_id` must be a deployed RMRK contract.
    /// * The token with id `parent_token_id` must exist in `parent_id` contract.
    /// * The `token_id` must not exist.
    ///
    /// # Arguments:
    /// * `parent_id`: is the address of RMRK parent contract.
    /// * `parent_token_id`: is the parent RMRK token.
    /// * `token_id`: is the tokenId of new RMRK token.
    ///
    /// On success replies [`RMRKEvent::MintToNft`].
    pub fn mint_to_nft(
        &mut self,
        state: TxState,
        tx_data: Option<Vec<u8>>,
        args: (ActorId, TokenId, TokenId),
    ) -> Result<(TxState, MessageId), RMRKError> {
        let (parent_id, parent_token_id, token_id) = args;
        match state {
            TxState::Initial => {
                self.assert_token_exists(token_id);
                let msg_id = add_child_msg(&parent_id, parent_token_id, token_id);
                Ok((TxState::MsgAddChildSent, msg_id))
            }
            TxState::ReplyAddChildReceived => {
                let msg_id = get_root_owner_msg(&parent_id, parent_token_id);
                Ok((TxState::MsgGetRootOwnerSent, msg_id))
            }
            TxState::ReplyRootOwnerReceived => {
                let reply = tx_data.expect("Failed to get a reply");
                let decoded_reply =
                    RMRKReply::decode(&mut &reply[..]).expect("Failed to decode a reply");
                let root_owner = if let RMRKReply::RootOwner(root_owner) = decoded_reply {
                    root_owner
                } else {
                    panic!("Wrong received reply");
                };

                self.internal_mint(&root_owner, token_id, &parent_id, Some(parent_token_id));
                Ok((TxState::Completed, MessageId::zero()))
            }
            _ => {
                unreachable!()
            }
        }
    }

    /// Mints token to the user.
    ///
    /// # Requirements:
    /// * The `token_id` must not exist.
    /// * The `to` address should be a non-zero address.
    ///
    /// # Arguments:
    /// * `root_owner`: is the address who will own the token.
    /// * `token_id`: is the tokenId of new RMRK token.
    ///
    /// On success replies [`RMRKEvent::MintToRootOwner`].
    pub fn mint_to_root_owner(&mut self, root_owner: &ActorId, token_id: TokenId) {
        self.assert_zero_address(root_owner);
        // check that token does not exist
        self.assert_token_exists(token_id);

        self.internal_mint(root_owner, token_id, root_owner, None);

        msg::reply(
            RMRKReply::MintToRootOwner {
                root_owner: *root_owner,
                token_id,
            },
            0,
        )
        .expect("Error in reply [RMRKEvent::MintToRootOwner]");
    }

    pub fn internal_mint(
        &mut self,
        root_owner: &ActorId,
        token_id: TokenId,
        parent_id: &ActorId,
        parent_token_id: Option<TokenId>,
    ) {
        self.increase_balance(root_owner);
        self.rmrk_owners.insert(
            token_id,
            RMRKOwner {
                token_id: parent_token_id,
                owner_id: *parent_id,
            },
        );
    }

    pub fn increase_balance(&mut self, account: &ActorId) {
        self.balances
            .entry(*account)
            .and_modify(|balance| *balance += 1.into())
            .or_insert_with(|| 1.into());
    }

    pub fn decrease_balance(&mut self, account: &ActorId) {
        self.balances
            .entry(*account)
            .and_modify(|balance| *balance -= 1.into());
    }
}

pub fn mint_to_nft_reply(tx: &mut Tx, processing_msg_id: MessageId) {
    let state = tx.state.clone();
    match state {
        TxState::MsgAddChildSent => {
            tx.state = TxState::ReplyAddChildReceived;
            exec::wake(processing_msg_id).expect("Failed to wake the message");
        }
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
