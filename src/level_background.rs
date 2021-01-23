use macroquad::{
    experimental::{
        collections::storage,
        scene::{self, RefMut},
    },
    prelude::*,
};

use crate::Resources;

pub struct LevelBackground {}

impl LevelBackground {
    pub fn new() -> LevelBackground {
        LevelBackground {}
    }
}

impl scene::Node for LevelBackground {
    fn draw(_node: RefMut<Self>) {
        let resources = storage::get_mut::<Resources>().unwrap();

        let w =
            resources.tiled_map.raw_tiled_map.tilewidth * resources.tiled_map.raw_tiled_map.width;
        let h =
            resources.tiled_map.raw_tiled_map.tileheight * resources.tiled_map.raw_tiled_map.height;
        resources
            .tiled_map
            .draw_tiles("main layer", Rect::new(0.0, 0.0, w as _, h as _), None);
    }
}
