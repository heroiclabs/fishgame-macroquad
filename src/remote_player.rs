use macroquad::{
    experimental::scene::{self, RefMut},
    prelude::*,
};

use crate::player::Fish;

pub struct RemotePlayer {
    pub fish: Fish,
    pub network_id: String,
}

impl RemotePlayer {
    pub fn new(network_id: String) -> RemotePlayer {
        RemotePlayer {
            fish: Fish::new(vec2(100., 105.)),
            network_id,
        }
    }
}
impl scene::Node for RemotePlayer {
    fn draw(mut node: RefMut<Self>) {
        draw_text_ex(
            &node.network_id[0..5],
            node.fish.pos().x - 1.,
            node.fish.pos().y - 1.,
            TextParams {
                font_size: 20,
                font_scale: 0.25,
                ..Default::default()
            },
        );

        node.fish.draw();
    }
}
