use macroquad::{
    experimental::{collections::storage, scene},
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
    fn draw(&mut self) {
        let resources = storage::get_mut::<Resources>().unwrap();

        resources
            .tiled_map
            .draw_tiles("main layer", Rect::new(0.0, 0.0, 320.0, 152.0), None)
    }
}
