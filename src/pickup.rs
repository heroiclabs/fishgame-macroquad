use macroquad::{
    experimental::{
        collections::storage,
        coroutines::{start_coroutine, wait_seconds},
        scene::{self, RefMut},
    },
    prelude::*,
};

use crate::Resources;

pub struct Pickup {
    pub pos: Vec2,
    visual_scale: f32,
}

impl Pickup {
    pub fn new(pos: Vec2) -> Pickup {
        Pickup {
            pos,
            visual_scale: 1.0,
        }
    }
}

impl scene::Node for Pickup {
    fn ready(node: RefMut<Self>) {
        let handle = node.handle();

        start_coroutine(async move {
            let n = 25;
            for i in 0..n {
                // if player pick up the item real quick - the node may be already removed here
                if let Some(mut this) = scene::get_node(handle) {
                    this.visual_scale =
                        1.0 + (i as f32 / n as f32 * std::f32::consts::PI).sin() * 3.0;
                }

                next_frame().await;
            }
        });

        start_coroutine(async move {
            wait_seconds(5.).await;

            let n = 10;
            for _ in 0..n {
                if let Some(mut this) = scene::get_node(handle) {
                    this.visual_scale -= 1.0 / n as f32;
                }
                next_frame().await;
            }

            if let Some(this) = scene::get_node(handle) {
                this.delete();
            }
        });
    }

    fn draw(node: RefMut<Self>) {
        let resources = storage::get_mut::<Resources>().unwrap();

        resources.tiled_map.spr_ex(
            "tileset",
            Rect::new(1.0 * 8.0, 6.0 * 8.0, 8.0, 8.0),
            Rect::new(
                node.pos.x - (8.0 * node.visual_scale - 8.) / 2.,
                node.pos.y - (8.0 * node.visual_scale - 8.) / 2.,
                8.0 * node.visual_scale,
                8.0 * node.visual_scale,
            ),
        );
        resources
            .tiled_map
            .spr("tileset", 122, Rect::new(node.pos.x, node.pos.y, 8.0, 8.0));
    }
}
