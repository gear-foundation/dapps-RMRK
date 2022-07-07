#![no_std]

pub mod primitives {
    use gstd::{prelude::*, ActorId};
    use primitive_types::U256;

    pub type CollectionId = ActorId;
    pub type ResourceId = u8;
    pub type TokenId = U256;
    pub type BaseId = ActorId;
    pub type PartId = u32;
    pub type SlotId = u32;
    pub type ZIndex = u32;
    pub type CollectionAndToken = (CollectionId, TokenId);
    pub type Parts = Vec<SlotId>;
}
