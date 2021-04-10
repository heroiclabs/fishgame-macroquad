use std::collections::HashMap;

use macroquad::{
    prelude::*,
    experimental::{
        animation::AnimatedSprite
    }
};

use crate::plugin::ModId;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct ItemType(u64);
impl From<ItemType> for u64 {
    fn from(item_type: ItemType) -> u64 {
        item_type.0
    }
}
impl From<u64> for ItemType {
    fn from(item_type: u64) -> ItemType {
        ItemType(item_type)
    }
}

struct ItemImplementation {
    display_name: String,
    pub texture: Texture2D,
    pub sprite: AnimatedSprite,
    pub fx_sprite: AnimatedSprite,
    implementing_mod: ModId,
}
pub struct ItemImplementationRegistry(HashMap<ItemType, ItemImplementation>);
impl ItemImplementationRegistry {
    pub fn get_implementation(&self, item_type: ItemType) -> Option<&ItemImplementation> {
        self.0.get(&item_type)
    }

    pub fn item_types(&self) -> Vec<ItemType> {
        self.0.keys().copied().collect()
    }
}

pub struct ItemIdSource(u64);
impl Default for ItemIdSource {
    fn default() -> Self {
        Self(1)
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ItemInstanceId(u64);

impl ItemIdSource {
    pub fn next_id(&mut self) -> ItemInstanceId {
        let new_id = self.0;
        self.0 += 1;
        ItemInstanceId(new_id)
    }
}
