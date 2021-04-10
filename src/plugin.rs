use std::collections::HashMap;

use macroquad::{
    experimental::{
        animation::AnimatedSprite
    }
};
use wasm_plugin_host::WasmPlugin;

use crate::item::ItemType;


pub struct ModId(u64);

pub struct ModDescription {
    mod_id: ModId,
    display_name: String,
    items: Vec<ItemDescription>,
}

pub struct ItemDescription {
    item_type: ItemType,
    display_name: String,
    texture_data: Vec<u8>,
    sprite: AnimatedSprite,
    fx_sprite: AnimatedSprite,
}

struct GameApiContext {
}

pub struct ModRegistry(HashMap<ModId, (WasmPlugin, GameApiContext)>);
