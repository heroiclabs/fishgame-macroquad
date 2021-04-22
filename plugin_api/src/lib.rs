use nanoserde::{DeJson, SerJson};

#[derive(Copy, Clone, PartialEq, Eq, Hash, DeJson, SerJson)]
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

#[derive(Copy, Clone, PartialEq, Eq, Hash, DeJson, SerJson)]
pub struct PluginId(pub u64);

#[derive(Clone, DeJson, SerJson)]
pub struct PluginDescription {
    pub plugin_id: PluginId,
    pub display_name: String,
    pub items: Vec<ItemDescription>,
}

#[derive(Clone, DeJson, SerJson)]
pub struct ItemDescription {
    pub item_type: ItemType,
    pub display_name: String,
    pub image: ImageDescription,
    pub sprite: AnimatedSpriteDescription,
    pub fx_sprite: AnimatedSpriteDescription,
}

// TODO: All these *Description structs could be replaced by adding the
// DeJson and SerJson derives inside macroquad, maybe behind a feature gate.
#[derive(Clone, DeJson, SerJson)]
pub struct ImageDescription {
    pub bytes: Vec<u8>,
    pub width: u16,
    pub height: u16,
}

#[derive(Clone, DeJson, SerJson)]
pub struct AnimatedSpriteDescription {
    pub tile_width: u32,
    pub tile_height: u32,
    pub animations: Vec<AnimationDescription>,
    pub playing: bool,
}

#[derive(Clone, Debug, DeJson, SerJson)]
pub struct AnimationDescription {
    pub name: String,
    pub row: u32,
    pub frames: u32,
    pub fps: u32,
}

pub struct GameApiContext {
}

