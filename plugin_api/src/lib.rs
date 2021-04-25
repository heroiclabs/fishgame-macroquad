use serde::{Serialize, Deserialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemType {
    id: u64
}

impl ItemType {
    pub const fn new(id: u64) -> Self {
        Self { id }
    }
}

impl From<ItemType> for u64 {
    fn from(item_type: ItemType) -> u64 {
        item_type.id
    }
}

impl From<u64> for ItemType {
    fn from(item_type: u64) -> ItemType{
        ItemType { id: item_type }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemInstanceId {
    pub id: u64
}

impl ItemInstanceId {
    pub const fn new(id: u64) -> Self {
        Self { id }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PluginId {
    id: u64
}

impl PluginId {
    pub const fn new(id: u64) -> Self {
        Self { id }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PluginDescription {
    pub plugin_id: PluginId,
    pub display_name: String,
    pub items: Vec<ItemDescription>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ItemDescription {
    pub item_type: ItemType,
    pub display_name: String,
    pub image: ImageDescription,
    pub mount_pos_right: [f32; 2],
    pub mount_pos_left: [f32; 2],
    pub pickup_src: Rect,
    pub pickup_dst: [f32; 2],
    pub sprite: AnimatedSpriteDescription,
    pub fx_sprite: AnimatedSpriteDescription,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ImageDescription {
    pub bytes: Vec<u8>,
    pub width: u16,
    pub height: u16,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AnimatedSpriteDescription {
    pub tile_width: u32,
    pub tile_height: u32,
    pub animations: Vec<AnimationDescription>,
    pub playing: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnimationDescription {
    pub name: String,
    pub row: u32,
    pub frames: u32,
    pub fps: u32,
}

#[macro_export]
macro_rules! import_game_api {
    () => {
        wasm_plugin_guest::import_functions! {
            fn spawn_bullet();
            fn set_sprite_fx(s: bool);
            fn get_speed() -> f32;
            fn set_speed(speed: f32);
            fn facing_dir() -> f32;
        }
    };
}
