use std::{
    path::Path,
    collections::HashMap,
};

use macroquad::{
    texture::Image,
    experimental::animation::{AnimatedSprite, Animation},
};
use nanoserde::DeJson;

use wasm_plugin_host::WasmPlugin;

use plugin_api::{ImageDescription, AnimationDescription, AnimatedSpriteDescription, PluginDescription, PluginId, GameApiContext};
use crate::nodes::{ItemImplementationRegistry};


pub fn image_from_desc(desc: ImageDescription) -> Image {
    Image {
        bytes: desc.bytes,
        width: desc.width,
        height: desc.height,
    }
}
pub fn animation_from_desc(desc: AnimationDescription) -> Animation {
    Animation {
        name: desc.name,
        row: desc.row,
        frames: desc.frames,
        fps: desc.fps,
    }
}

pub fn animated_sprite_from_desc(desc: AnimatedSpriteDescription) -> AnimatedSprite {
    let animations: Vec<Animation> = desc.animations.into_iter().map(|a| animation_from_desc(a)).collect();
    AnimatedSprite::new(
        desc.tile_width,
        desc.tile_height,
        &animations,
        desc.playing,
    )
}

pub struct PluginRegistry(HashMap<PluginId, (WasmPlugin, GameApiContext)>);

impl PluginRegistry {
    pub fn load(path: impl AsRef<Path>, item_registry: &mut ItemImplementationRegistry) -> Self {
        let mut plugins = HashMap::new();
        for entry in path.as_ref().read_dir().expect("Unable to read plugins directory") {
            if let Ok(entry) = entry {
                if entry.path().to_str().unwrap().contains(".wasm") {
                    let mut plugin = wasm_plugin_host::WasmPluginBuilder::from_file(entry.path()).expect(&format!("Failed to load plugin {:?}", entry.path())).finish().unwrap();
                    let description: PluginDescription = plugin.call_function("plugin_description").expect(&format!("Failed to call 'plugin_description' on plugin {:?}", entry.path()));

                    for item in description.items {
                        item_registry.add(item, description.plugin_id);
                    }

                    plugins.insert(description.plugin_id, (plugin, GameApiContext {}));
                }
            }
        }

        PluginRegistry(plugins)
    }

    pub(crate) fn get_plugin(&mut self, plugin_id: PluginId) -> Option<&mut WasmPlugin> {
        self.0.get_mut(&plugin_id).map(|(p, _)| p)
    }
}
