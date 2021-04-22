use std::collections::HashMap;

use macroquad::{
    prelude::*,
    experimental::animation::AnimatedSprite,
};

use plugin_api::{ItemType, PluginId, ItemDescription};
use crate::plugin::{image_from_desc, animated_sprite_from_desc};


pub(crate) struct ItemImplementation {
    display_name: String,
    pub texture: Texture2D,
    pub sprite: AnimatedSprite,
    pub fx_sprite: AnimatedSprite,
    implementing_plugin: PluginId,
}

impl ItemImplementation {
    pub(crate) fn from_description(description: ItemDescription, plugin: PluginId) -> Self {
        let texture = Texture2D::from_image(&image_from_desc(description.image));
        Self {
            display_name: description.display_name,
            texture,
            sprite: animated_sprite_from_desc(description.sprite),
            fx_sprite: animated_sprite_from_desc(description.fx_sprite),
            implementing_plugin: plugin
        }
    }
}

#[derive(Default)]
pub struct ItemImplementationRegistry(HashMap<ItemType, ItemImplementation>);

impl ItemImplementationRegistry {
    pub(crate) fn add(&mut self, description: ItemDescription, implementing_plugin: PluginId) {
        self.0.insert(description.item_type, ItemImplementation::from_description(description, implementing_plugin));
    }

    pub(crate) fn get_implementation(&self, item_type: ItemType) -> Option<&ItemImplementation> {
        self.0.get(&item_type)
    }

    pub(crate) fn item_types(&self) -> Vec<ItemType> {
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
