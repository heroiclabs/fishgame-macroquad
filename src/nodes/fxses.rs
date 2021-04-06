use macroquad::{
    experimental::{
        collections::storage,
        scene::{self, Handle, RefMut},
    },
    telemetry,
};

use crate::{nodes::Camera, Resources};

pub struct Fxses {
    pub camera: Handle<Camera>,
}

impl scene::Node for Fxses {
    fn draw(node: RefMut<Self>) {
        let camera = scene::get_node(node.camera)
            .unwrap()
            .macroquad_camera()
            .clone();
        let mut resources = storage::get_mut::<Resources>().unwrap();

        let _z = telemetry::ZoneGuard::new("draw particles");

        resources.hit_fxses.draw(camera);
        resources.explosion_fxses.draw(camera);
        resources.disarm_fxses.draw(camera);
    }
}
