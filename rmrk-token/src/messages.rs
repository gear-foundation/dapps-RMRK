use crate::*;
use catalog_io::CatalogReply;
use catalog_io::*;
use gstd::{exec, msg, ActorId};
use resource_io::Resource;
use types::primitives::{CollectionId, PartId, ResourceId, TokenId};
pub const REPLY_PROVISION: u64 = 1_000_000_000;

pub async fn get_root_owner(to: &ActorId, token_id: TokenId) -> ActorId {
    let response: RMRKReply = msg::send_for_reply_as(*to, RMRKAction::RootOwner(token_id), 0)
        .expect("Error in sending message [RMRKAction::RootOwner]")
        .await
        .expect("Error in message [RMRKAction::RootOwner]");

    if let RMRKReply::RootOwner(root_owner) = response {
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
    msg::send_for_reply(
        *parent_contract_id,
        RMRKAction::AddChild {
            parent_token_id,
            child_token_id,
        },
        0,
    )
    .expect("Error in sending message [RMRKAction::AddChild]")
    .await
    .expect("");
}

pub fn add_child_msg(
    parent_contract_id: &ActorId,
    parent_token_id: TokenId,
    child_token_id: TokenId,
) -> MessageId {
    let msg_id = msg::send(
        *parent_contract_id,
        RMRKAction::AddChild {
            parent_token_id,
            child_token_id,
        },
        0,
    )
    .expect("Error in sending message [RMRKAction::AddChild]");
    exec::create_provision(msg_id, REPLY_PROVISION).expect("Failed to create a reply provision");
    msg_id
}

pub fn burn_child_msg(
    parent_contract_id: &ActorId,
    parent_token_id: TokenId,
    child_token_id: TokenId,
) -> MessageId {
    let msg_id = msg::send(
        *parent_contract_id,
        RMRKAction::BurnChild {
            parent_token_id,
            child_token_id,
        },
        0,
    )
    .expect("Error in sending message [RMRKAction::BurnChild]");
    exec::create_provision(msg_id, REPLY_PROVISION).expect("Failed to create a reply provision");
    msg_id
}
pub fn get_root_owner_msg(contract_id: &ActorId, token_id: TokenId) -> MessageId {
    let msg_id = msg::send(*contract_id, RMRKAction::RootOwner(token_id), 0)
        .expect("Error in sending message [RMRKAction::RootOwner]");
    exec::create_provision(msg_id, REPLY_PROVISION).expect("Failed to create a reply provision");
    msg_id
}

pub async fn burn_from_parent(
    child_contract_id: &ActorId,
    child_token_id: TokenId,
    root_owner: &ActorId,
) {
    msg::send_for_reply_as::<_, RMRKReply>(
        *child_contract_id,
        RMRKAction::BurnFromParent {
            child_token_id,
            root_owner: *root_owner,
        },
        0,
    )
    .expect("Error in sending message [RMRKAction::BurnFromParent]")
    .await
    .expect("Error in message [RMRKAction::BurnFromParent]");
}

pub fn burn_from_parent_msg(
    child_contract_id: &ActorId,
    child_token_id: TokenId,
    root_owner: &ActorId,
) -> MessageId {
    let msg_id = msg::send(
        *child_contract_id,
        RMRKAction::BurnFromParent {
            child_token_id,
            root_owner: *root_owner,
        },
        0,
    )
    .expect("Error in sending message [RMRKAction::BurnFromParent]");
    exec::create_provision(msg_id, REPLY_PROVISION).expect("Failed to create a reply provision");
    msg_id
}

pub async fn burn_child(
    parent_contract_id: &ActorId,
    parent_token_id: TokenId,
    child_token_id: TokenId,
) {
    msg::send_for_reply_as::<_, RMRKReply>(
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
    msg::send_for_reply_as::<_, RMRKReply>(
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

pub fn transfer_child_msg(
    parent_contract_id: &ActorId,
    from: TokenId,
    to: TokenId,
    child_token_id: TokenId,
) -> MessageId {
    let msg_id = msg::send(
        *parent_contract_id,
        RMRKAction::TransferChild {
            from,
            to,
            child_token_id,
        },
        0,
    )
    .expect("Error in sending async message `[RMRKAction::TransferChild]`");
    exec::create_provision(msg_id, REPLY_PROVISION).expect("Failed to create a reply provision");
    msg_id
}

pub fn add_accepted_child_msg(
    parent_contract_id: &ActorId,
    parent_token_id: TokenId,
    child_token_id: TokenId,
) -> MessageId {
    let msg_id = msg::send(
        *parent_contract_id,
        RMRKAction::AddAcceptedChild {
            parent_token_id,
            child_token_id,
        },
        0,
    )
    .expect("Error in sending async message `[RMRKAction::AddAcceptedChild]`");
    exec::create_provision(msg_id, REPLY_PROVISION).expect("Failed to create a reply provision");
    msg_id
}

pub async fn add_resource_entry(to: &ActorId, resource_id: ResourceId, resource: Resource) {
    msg::send_for_reply_as::<_, ResourceEvent>(
        *to,
        ResourceAction::AddResourceEntry {
            resource_id,
            resource,
        },
        0,
    )
    .expect(
        "Error in sending async message `[ResourceAction::AddResourceEntry]` to resource contract",
    )
    .await
    .expect("Error in async message `[ResourceAction::AddResourceEntry]`");
}

pub fn add_resource_entry_msg(
    to: &ActorId,
    resource_id: ResourceId,
    resource: Resource,
) -> MessageId {
    let msg_id = msg::send(
        *to,
        ResourceAction::AddResourceEntry {
            resource_id,
            resource,
        },
        0,
    )
    .expect(
        "Error in sending async message `[ResourceAction::AddResourceEntry]` to resource contract",
    );
    exec::create_provision(msg_id, REPLY_PROVISION).expect("Failed to create a reply provision");
    msg_id
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

pub fn get_resource_msg(resource_address: &ActorId, id: u8) -> MessageId {
    let msg_id = msg::send(*resource_address, ResourceAction::GetResource { id }, 0).expect(
        "Error in sending async message `[ResourceAction::GetResource]` to resource contract",
    );
    exec::create_provision(msg_id, REPLY_PROVISION).expect("Failed to create a reply provision");
    msg_id
}

pub async fn get_resource(resource_address: &ActorId, id: ResourceId) -> Resource {
    let response: ResourceEvent = msg::send_for_reply_as(
        *resource_address,
        ResourceAction::GetResource { id },
        0,
    )
    .expect("Error in sending async message `[ResourceAction::GetResource]` to resource contract")
    .await
    .expect("Error in async message `[ResourceAction::GetResource]`");
    if let ResourceEvent::Resource(resource) = response {
        resource
    } else {
        panic!("Wrong received message from resource contract");
    }
}

pub async fn check_is_in_equippable_list(base_id: BaseId, part_id: PartId, token_id: TokenId) {
    // msg::send_for_reply_as::<_, CatalogReply>(
    //     base_id,
    //     BaseAction::CheckEquippable {
    //         part_id,
    //         collection_id: exec::program_id(),
    //         token_id,
    //     },
    //     0,
    // )
    // .expect("Error in sending async message `[BaseAction::CheckEquippable]` to base contract")
    // .await
    // .expect("Error in async message `[BaseAction::CheckEquippable]`");
}

pub fn check_equippable_msg(
    catalog_id: &ActorId,
    part_id: PartId,
    collection_id: &CollectionId,
) -> MessageId {
    let msg_id = msg::send(
        *catalog_id,
        CatalogAction::CheckEquippable {
            part_id,
            collection_id: *collection_id,
        },
        0,
    )
    .expect("Error in sending a message");
    exec::create_provision(msg_id, REPLY_PROVISION).expect("Failed to create a reply provision");
    msg_id
}

pub fn can_token_be_equipped_msg(
    child_id: &ActorId,
    parent_id: &ActorId,
    token_id: TokenId,
    asset_id: u64,
    slot_part_id: PartId,
) -> MessageId {
    let msg_id = msg::send(
        *child_id,
        RMRKAction::CanTokenBeEquippedWithAssetIntoSlot {
            parent_id: *parent_id,
            token_id,
            asset_id,
            slot_part_id,
        },
        0,
    )
    .expect("Error in sending a message");
    exec::create_provision(msg_id, REPLY_PROVISION).expect("Failed to create a reply provision");
    msg_id
}

pub async fn add_part_to_resource(
    resource_contract_id: ActorId,
    resource_id: ResourceId,
    part_id: PartId,
) {
    msg::send_for_reply_as::<_, ResourceEvent>(
        resource_contract_id,
        ResourceAction::AddPartToResource {
            resource_id,
            part_id,
        },
        0,
    )
    .expect(
        "Error in sending async message `[ResourceAction::AddPartToResource]` to resource contract",
    )
    .await
    .expect("Error in async message `[ResourceAction::AddPartToResource]`");
}
