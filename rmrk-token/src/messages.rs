use crate::*;
use gstd::{msg, ActorId};

pub async fn get_root_owner(to: &ActorId, token_id: TokenId) -> ActorId {
    let response: RMRKEvent = msg::send_for_reply_as(*to, RMRKAction::RootOwner(token_id), 0)
        .expect("Error in sending message [RMRKAction::RootOwner]")
        .await
        .expect("Error in message [RMRKAction::RootOwner]");

    if let RMRKEvent::RootOwner(root_owner) = response {
        root_owner
    } else {
        panic!("wrong received message");
    }
}

pub async fn add_child(
    parent_contract_id: &ActorId,
    parent_token_id: TokenId,
    child_token_id: TokenId,
) {
    msg::send_for_reply_as::<_, RMRKEvent>(
        *parent_contract_id,
        RMRKAction::AddChild {
            parent_token_id,
            child_token_id,
        },
        0,
    )
    .expect("Error in sending message [RMRKAction::AddChild]")
    .await
    .expect("Error in message [RMRKAction::AddChild]");
}

pub async fn burn_from_parent(
    child_contract_id: &ActorId,
    child_token_ids: BTreeSet<TokenId>,
    root_owner: &ActorId,
) {
    msg::send_for_reply_as::<_, RMRKEvent>(
        *child_contract_id,
        RMRKAction::BurnFromParent {
            child_token_ids,
            root_owner: *root_owner,
        },
        0,
    )
    .expect("Error in sending message [RMRKAction::BurnFromParent]")
    .await
    .expect("Error in message [RMRKAction::BurnFromParent]");
}

pub async fn burn_child(
    parent_contract_id: &ActorId,
    parent_token_id: TokenId,
    child_token_id: TokenId,
) {
    msg::send_for_reply_as::<_, RMRKEvent>(
        *parent_contract_id,
        RMRKAction::BurnChild {
            parent_token_id,
            child_token_id,
        },
        0,
    )
    .expect("Error in sending message [RMRKAction::BurnChild]")
    .await
    .expect("Error in message [RMRKAction::BurnChild]");
}

pub async fn transfer_child(
    parent_contract_id: &ActorId,
    from: TokenId,
    to: TokenId,
    child_token_id: TokenId,
) {
    msg::send_for_reply_as::<_, RMRKEvent>(
        *parent_contract_id,
        RMRKAction::TransferChild {
            from,
            to,
            child_token_id,
        },
        0,
    )
    .expect("Error in sending async message `[RMRKAction::TransferChild]`")
    .await
    .expect("Error in async message `[RMRKAction::TransferChild]`");
}

pub async fn add_accepted_child(
    parent_contract_id: &ActorId,
    parent_token_id: TokenId,
    child_token_id: TokenId,
) {
    msg::send_for_reply_as::<_, RMRKEvent>(
        *parent_contract_id,
        RMRKAction::AddAcceptedChild {
            parent_token_id,
            child_token_id,
        },
        0,
    )
    .expect("Error in sending async message `[RMRKAction::AddAcceptedChild]`")
    .await
    .expect("Error in  async message `[RMRKAction::AddAcceptedChild]");
}

pub async fn add_resource_entry(
    to: &ActorId,
    id: u8,
    src: String,
    thumb: String,
    metadata_uri: String,
) {
    msg::send_for_reply_as::<_, ResourceEvent>(
        *to,
        ResourceAction::AddResourceEntry {
            id,
            src,
            thumb,
            metadata_uri,
        },
        0,
    )
    .expect(
        "Error in sending async message `[ResourceAction::AddResourceEntry]` to resource contract",
    )
    .await
    .expect("Error in async message `[ResourceAction::AddResourceEntry]`");
}

pub async fn assert_resource_exists(resource_address: &ActorId, id: u8) {
    msg::send_for_reply_as::<_, ResourceEvent>(
        *resource_address,
        ResourceAction::GetResource { id },
        0,
    )
    .expect("Error in sending async message `[ResourceAction::GetResource]` to resource contract")
    .await
    .expect("Error in async message `[ResourceAction::GetResource]`");
}
