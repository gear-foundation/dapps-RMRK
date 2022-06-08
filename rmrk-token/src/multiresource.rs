use crate::*;

pub const MAX_RESOURCE_LEN: u8 = 128;

#[derive(Debug, Default)]
pub struct MultiResource {
    pub token_resources: BTreeMap<TokenId, BTreeSet<ResourceId>>,
    pub pending_resources: BTreeMap<TokenId, BTreeSet<ResourceId>>,
    pub active_resources: BTreeMap<TokenId, BTreeSet<ResourceId>>,
    pub resource_overwrites: BTreeMap<TokenId, BTreeMap<ResourceId, ResourceId>>,
    pub active_resources_priorities: BTreeMap<TokenId, Vec<u8>>,
}
impl RMRKToken {
    /// Adds resource entry on resource storage contract
    /// It sends a message to resource storage contract with information about new resource
    ///
    /// Arguments:
    /// * `id`: is a resource identifier
    /// * `src`: a string pointing to the media associated with the resource
    /// * `thumb`: a string pointing to thumbnail media associated with the resource
    /// * `metadata_uri`:  a string pointing to a metadata file associated with the resource
    pub async fn add_resource_entry(
        &mut self,
        id: u8,
        src: String,
        thumb: String,
        metadata_uri: String,
    ) {
        // sends message to resource storage contract
        add_resource_entry(&self.resource_id, id, src, thumb, metadata_uri).await;
        msg::reply(RMRKEvent::ResourceEntryAdded { id }, 0).unwrap();
    }

    /// Adds resource to an existing token
    /// Proposed resource is placed in the "Pending" array
    /// A pending resource can be also proposed to overwrite an existing resource
    ///
    /// Requirements
    /// Token with indicated `token_id` must exist
    /// The proposed resource must not already exist for the token
    /// The resource that is proposed to be overwritten must exist for the token
    ///
    /// Arguments:
    /// * `token_id`: an id of the token
    /// * `resource_id`: a proposed resource
    /// * `overwrite_id`: a resource to be overwritten
    pub async fn add_resource(&mut self, token_id: TokenId, resource_id: u8, overwrite_id: u8) {
        self.assert_token_does_not_exist(token_id);
        assert_resource_exists(&self.resource_id, resource_id).await;

        if let Some(token_resources) = self.multiresource.token_resources.get(&token_id) {
            assert!(
                !token_resources.contains(&resource_id),
                "Resource already exists on token"
            );
        }
        if let Some(pending_resources) = self.multiresource.pending_resources.get(&token_id) {
            assert!(
                pending_resources.len() < MAX_RESOURCE_LEN as usize,
                "Max pending resources reached"
            );
        }

        if overwrite_id != 0 {
            if let Some(token_resources) = self.multiresource.active_resources.get(&token_id) {
                assert!(
                    token_resources.contains(&overwrite_id),
                    "Proposed overwritten resource must exist on token"
                );
            } else {
                panic!("No resources to overwrite")
            }
            self.multiresource
                .resource_overwrites
                .entry(token_id)
                .and_modify(|r| {
                    r.insert(resource_id, overwrite_id);
                })
                .or_insert_with(|| {
                    let mut r = BTreeMap::new();
                    r.insert(resource_id, overwrite_id);
                    r
                });
        }

        self.multiresource
            .token_resources
            .entry(token_id)
            .and_modify(|r| {
                r.insert(resource_id);
            })
            .or_insert_with(|| {
                let mut r = BTreeSet::new();
                r.insert(resource_id);
                r
            });
        self.multiresource
            .pending_resources
            .entry(token_id)
            .and_modify(|r| {
                r.insert(resource_id);
            })
            .or_insert_with(|| {
                let mut r = BTreeSet::new();
                r.insert(resource_id);
                r
            });

        msg::reply(
            RMRKEvent::ResourceAdded {
                token_id,
                resource_id,
                overwrite_id,
            },
            0,
        )
        .unwrap();
    }

    /// Accepts resource from pending
    ///
    /// Requirements
    /// Only root owner or approved account can accept a resource
    /// `resource_id` must exist for the token in the pending array
    ///
    /// Arguments:
    /// * `token_id`: an id of the token
    /// * `resource_id`: a resource to be accepted
    pub async fn accept_resource(&mut self, token_id: TokenId, resource_id: u8) {
        let root_owner = self.find_root_owner(token_id).await;
        self.assert_approved_or_owner(token_id, &root_owner);

        if let Some(pending_resources) = self.multiresource.pending_resources.get_mut(&token_id) {
            assert!(
                pending_resources.remove(&resource_id),
                "Resource does not exist"
            );
        }
        if let Some(resources) = self.multiresource.resource_overwrites.get_mut(&token_id) {
            if let Some(overwrite_resource) = resources.remove(&resource_id) {
                self.multiresource
                    .active_resources
                    .entry(token_id)
                    .and_modify(|r| {
                        r.remove(&overwrite_resource);
                    });
            }
        }
        self.multiresource
            .active_resources
            .entry(token_id)
            .and_modify(|r| {
                r.insert(resource_id);
            })
            .or_insert_with(|| {
                let mut r = BTreeSet::new();
                r.insert(resource_id);
                r
            });
        self.multiresource
            .active_resources_priorities
            .remove(&token_id);
        msg::reply(
            RMRKEvent::ResourceAccepted {
                token_id,
                resource_id,
            },
            0,
        )
        .unwrap();
    }

    /// Rejects a resource, dropping it from the pending array.
    ///
    /// Requirements
    /// Only root owner or approved account can reject a resource
    /// `resource_id` must exist for the token in the pending array
    ///
    /// Arguments:
    /// * `token_id`: an id of the token
    /// * `resource_id`: a resource to be rejected
    pub async fn reject_resource(&mut self, token_id: TokenId, resource_id: u8) {
        let root_owner = self.find_root_owner(token_id).await;
        self.assert_approved_or_owner(token_id, &root_owner);
        if let Some(pending_resources) = self.multiresource.pending_resources.get_mut(&token_id) {
            assert!(
                pending_resources.remove(&resource_id),
                "Resource does not exist"
            );
        } else {
            panic!("Token does not have any pending resources");
        }

        self.multiresource
            .token_resources
            .entry(token_id)
            .and_modify(|r| {
                r.remove(&resource_id);
            });

        msg::reply(
            RMRKEvent::ResourceRejected {
                token_id,
                resource_id,
            },
            0,
        )
        .unwrap();
    }

    /// Sets the priority of the active resources array
    /// Priorities have a 1:1 relationship with their corresponding index in
    /// the active resources array. E.G, a priority array of [1, 3, 2] indicates
    ///  that the the active resource at index 1 of the active resource array
    ///  has a priority of 1, index 2 has a priority of 3, and index 3 has a priority
    ///  of 2. There is no validation on priority value input; out of order indexes
    ///  must be handled by the frontend.
    ///
    /// Requirements
    /// Only root owner or approved account can set priority
    /// The length of the priorities array must be equal to the present length of the active resources array
    ///
    /// Arguments:
    /// * `token_id`: an id of the token
    /// * `priorities`: An array of priorities to set
    pub async fn set_priority(&mut self, token_id: TokenId, priorities: Vec<u8>) {
        let root_owner = self.find_root_owner(token_id).await;
        self.assert_approved_or_owner(token_id, &root_owner);

        if let Some(active_resources) = self.multiresource.active_resources.get(&token_id) {
            assert!(
                active_resources.len() == priorities.len(),
                "Wrong priority list length"
            );
        }
        self.multiresource
            .active_resources_priorities
            .insert(token_id, priorities.clone());
        msg::reply(
            RMRKEvent::PrioritySet {
                token_id,
                priorities,
            },
            0,
        )
        .unwrap();
    }
}
