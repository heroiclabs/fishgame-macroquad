use std::collections::HashMap;

use macroquad::{
    experimental::{animation::AnimatedSprite, collections::storage, scene::Handle},
    prelude::*,
};

use crate::{
    nodes::{Player, RemotePlayer},
    plugin::{animated_sprite_from_desc, image_from_desc, Plugin, PluginRegistry},
};
use plugin_api::{ItemDescription, ItemInstanceId, ItemType, PluginApi, PluginId};

pub(crate) struct ItemImplementation {
    display_name: String,
    item_type: ItemType,
    pub texture: Texture2D,
    pub mount_pos_right: Vec2,
    pub mount_pos_left: Vec2,
    pub pickup_src: Rect,
    pub pickup_dst: Vec2,
    pub sprite: AnimatedSprite,
    pub fx_sprite: Option<AnimatedSprite>,
    implementing_plugin: PluginId,
}

impl ItemImplementation {
    pub(crate) fn from_description(description: ItemDescription, plugin: PluginId) -> Self {
        let texture = Texture2D::from_image(&image_from_desc(description.image));
        let pickup_src = description.pickup_src;
        Self {
            display_name: description.display_name,
            item_type: description.item_type,
            texture,
            mount_pos_right: vec2(
                description.mount_pos_right[0],
                description.mount_pos_right[1],
            ),
            mount_pos_left: vec2(description.mount_pos_left[0], description.mount_pos_left[1]),
            pickup_src: Rect::new(pickup_src.x, pickup_src.y, pickup_src.w, pickup_src.h),
            pickup_dst: vec2(description.pickup_dst[0], description.pickup_dst[1]),
            sprite: animated_sprite_from_desc(description.sprite),
            fx_sprite: description.fx_sprite.map(|d| animated_sprite_from_desc(d)),
            implementing_plugin: plugin,
        }
    }
}

impl ItemImplementation {
    fn with_plugin<R>(&self, mut f: impl FnMut(&mut Plugin) -> R) -> R {
        let mut plugin_registry = storage::get_mut::<PluginRegistry>();
        let plugin = plugin_registry
            .get_plugin(self.implementing_plugin)
            .unwrap();
        f(plugin)
    }

    pub(crate) fn construct(&self, item_id: ItemInstanceId) {
        self.with_plugin(|p| {
            p.new_instance(self.item_type, item_id);
        })
    }

    pub(crate) fn destroy(&self, item_id: ItemInstanceId) {
        self.with_plugin(|p| {
            p.destroy_instance(item_id);
        })
    }

    pub(crate) fn uses_remaining(&self, item_id: ItemInstanceId) -> Option<(u32, u32)> {
        self.with_plugin(|p| p.uses_remaining(item_id))
    }

    pub(crate) fn update_shoot(&self, item_id: ItemInstanceId, player: Handle<Player>) -> bool {
        self.with_plugin(|p| p.with_current_player(player, |p| p.update_shoot(item_id, get_time())))
    }

    pub(crate) fn update_remote_shoot(
        &self,
        item_id: ItemInstanceId,
        player: Handle<RemotePlayer>,
    ) -> bool {
        self.with_plugin(|p| {
            p.with_current_remote_player(player, |p| p.update_remote_shoot(item_id, get_time()))
        })
    }
}

#[derive(Default)]
pub struct ItemImplementationRegistry(HashMap<ItemType, ItemImplementation>);

impl ItemImplementationRegistry {
    pub(crate) fn add(&mut self, description: ItemDescription, implementing_plugin: PluginId) {
        self.0.insert(
            description.item_type,
            ItemImplementation::from_description(description, implementing_plugin),
        );
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

impl ItemIdSource {
    pub fn next_id(&mut self) -> ItemInstanceId {
        let new_id = self.0;
        self.0 += 1;
        ItemInstanceId::new(new_id)
    }
}
