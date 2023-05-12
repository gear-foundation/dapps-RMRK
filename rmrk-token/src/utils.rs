use crate::*;

impl From<MultiResource> for MultiResourceState {
    fn from(multiresource: MultiResource) -> MultiResourceState {
        MultiResourceState {
            pending_resources: multiresource
                .pending_resources
                .iter()
                .map(|(key, value)| (*key, value.iter().copied().collect()))
                .collect(),
            active_resources: multiresource
                .active_resources
                .iter()
                .map(|(key, value)| (*key, value.iter().copied().collect()))
                .collect(),
            resource_overwrites: multiresource
                .resource_overwrites
                .iter()
                .map(|(key, value)| {
                    (
                        *key,
                        value
                            .clone()
                            .iter()
                            .map(|(key, value)| (*key, *value))
                            .collect(),
                    )
                })
                .collect(),
            active_resources_priorities: multiresource
                .active_resources_priorities
                .iter()
                .map(|(key, value)| (*key, value.clone()))
                .collect(),
        }
    }
}

impl From<&RMRKToken> for RMRKState {
    fn from(rmrk: &RMRKToken) -> RMRKState {
        RMRKState {
            name: rmrk.name.clone(),
            symbol: rmrk.symbol.clone(),
            admin: rmrk.admin,
            token_approvals: rmrk
                .token_approvals
                .iter()
                .map(|(key, value)| (*key, value.iter().copied().collect()))
                .collect(),
            rmrk_owners: rmrk
                .rmrk_owners
                .iter()
                .map(|(key, value)| (*key, value.clone()))
                .collect(),
            pending_children: rmrk
                .pending_children
                .iter()
                .map(|(key, value)| (*key, value.iter().copied().collect()))
                .collect(),
            accepted_children: rmrk
                .accepted_children
                .iter()
                .map(|(key, value)| (*key, value.iter().copied().collect()))
                .collect(),
            children_status: rmrk
                .children_status
                .iter()
                .map(|(key, value)| (*key, *value))
                .collect(),
            balances: rmrk
                .balances
                .clone()
                .iter()
                .map(|(key, value)| (*key, *value))
                .collect(),
            multiresource: rmrk.multiresource.clone().into(),
            resource_id: rmrk.resource_id,
            assets: AssetsState::default(),
        }
    }
}

impl From<&Assets> for AssetsState {
    fn from(assets: &Assets) -> AssetsState {
        AssetsState {
            assets: assets
                .assets
                .iter()
                .map(|(key, value)| (*key, value.clone()))
                .collect(),
            catalog_addresses: assets
                .catalog_addresses
                .iter()
                .map(|(key, value)| (*key, *value))
                .collect(),
            equippable_group_ids: assets
                .equippable_group_ids
                .iter()
                .map(|(key, value)| (*key, *value))
                .collect(),
            part_ids: assets
                .part_ids
                .iter()
                .map(|(key, value)| (*key, value.clone()))
                .collect(),
            pending_assets: assets
                .pending_assets
                .iter()
                .map(|(key, value)| (*key, value.clone()))
                .collect(),
            active_assets: assets
                .active_assets
                .iter()
                .map(|(key, value)| (*key, value.clone()))
                .collect(),
            active_assets_priorities: assets
                .active_assets_priorities
                .iter()
                .map(|(key, value)| (*key, value.clone()))
                .collect(),
            asset_replacement: assets
                .asset_replacement
                .iter()
                .map(|(key, value)| {
                    (
                        *key,
                        value.iter().map(|(key, value)| (*key, *value)).collect(),
                    )
                })
                .collect(),
            valid_parent_slots: assets
                .valid_parent_slots
                .iter()
                .map(|(key, value)| {
                    (
                        *key,
                        value.iter().map(|(key, value)| (*key, *value)).collect(),
                    )
                })
                .collect(),
            equipments: assets
                .equipments
                .iter()
                .map(|(key, value)| {
                    (
                        *key,
                        value
                            .iter()
                            .map(|(key, value)| (*key, value.clone()))
                            .collect(),
                    )
                })
                .collect(),
        }
    }
}
