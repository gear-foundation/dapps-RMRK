use crate::*;
use gstd::msg;
use resource_io::{ComposedResource, Resource, SlotResource};
use types::primitives::ResourceId;

impl RMRKToken {
    pub async fn equip(
        &mut self,
        token_id: TokenId,
        resource_id: ResourceId,
        equippable: CollectionAndToken,
        equippable_resource_id: ResourceId,
    ) {
        assert!(
            !self.equipped_tokens.contains(&token_id),
            "Token is already equipped"
        );
        let root_owner = self.find_root_owner(token_id).await;
        self.assert_owner(&root_owner);

        // check that resource exists for token
        self.assert_resource_exists(token_id, resource_id);

        // sends message to resource contract to check that the token has slot resource and get slot id from it
        let resource = get_resource(&self.resource_id, resource_id).await;
        let slot_id = if let Resource::Slot(SlotResource { slot, .. }) = resource {
            slot
        } else {
            panic!("The resource must be slot");
        };

        // sends  message to parent contract
        // parent contract checks whether child is in accepted status
        // parent contract checks that it has the indicated resource that must be composed
        let (parent_contract_id, parent_token_id) = equippable;
        check_equippable(
            parent_contract_id,
            parent_token_id,
            token_id,
            equippable_resource_id,
            slot_id,
        )
        .await;

        self.equipped_tokens.insert(token_id);

        msg::reply(
            RMRKEvent::TokenEquipped {
                token_id,
                resource_id,
                slot_id,
                equippable,
            },
            0,
        )
        .expect("Error in reply [RMRKEvent::TokenEquipped]");
    }

    pub async fn check_equippable(
        &mut self,
        token_id: TokenId,
        child_token_id: TokenId,
        resource_id: ResourceId,
        slot_id: SlotId,
    ) {
        let child_token = (msg::source(), child_token_id);
        // check that token has the indicated child in the accepted child array
        if let Some(accepted_children) = self.accepted_children.get(&token_id) {
            assert!(
                accepted_children.contains(&child_token),
                "That token is not in the accepted list"
            );
        } else {
            panic!("Token has no accepted children");
        }

        // check that token has the indicated resource
        self.assert_resource_exists(token_id, resource_id);

        // check that resource is composed
        let resource = get_resource(&self.resource_id, resource_id).await;
        if let Resource::Composed(ComposedResource { base, .. }) = resource {
            // check that the token in equippable list
            check_is_in_equippable_list(base, slot_id, token_id).await;
        } else {
            panic!("The resource must be composed");
        }

        msg::reply(RMRKEvent::EquippableIsOk, 0)
            .expect("Error in reply [RMRKEvent::EquippableIsOk]");
    }
}
