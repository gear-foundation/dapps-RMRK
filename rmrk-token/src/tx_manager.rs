use crate::*;

#[derive(Default)]
pub struct TxManager {
    pub txs: BTreeMap<MessageId, Tx>,
    // mapping from send message ID to processing message ID
    pub msg_sent_to_msg: BTreeMap<MessageId, MessageId>,
}

impl TxManager {
    pub fn get_tx(&mut self, msg: &RMRKAction) -> &mut Tx {
        self.txs.entry(msg::id()).or_insert_with(|| Tx {
            msg: msg.clone(),
            state: TxState::Initial,
            data: None,
        })
    }
}
