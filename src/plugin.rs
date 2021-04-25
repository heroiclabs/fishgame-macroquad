use std::{
    sync::{Arc, Mutex},
    path::Path,
    collections::HashMap,
};

use macroquad::{
    texture::Image,
    experimental::{
        scene::{self, Handle, RefMut},
        animation::{AnimatedSprite, Animation},
    },
};

use wasm_plugin_host::WasmPlugin;

use plugin_api::{ImageDescription, AnimationDescription, AnimatedSpriteDescription, PluginDescription, PluginId};
use crate::nodes::{ItemImplementationRegistry, Player, Fish};


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

pub struct PluginRegistry(HashMap<PluginId, Plugin>);

pub struct Plugin {
    pub wasm_plugin: WasmPlugin,
    pub game_api: Arc<Mutex<GameApi>>
}

#[derive(Default)]
pub struct GameApi {
    current_fish: Option<*mut Fish>,
}
unsafe impl Send for GameApi {}
unsafe impl Sync for GameApi {}

impl GameApi {
    pub fn set_current_fish(&mut self, player: &mut Fish) {
        self.current_fish.replace(player as *mut Fish);
    }
    pub fn clear_current_fish(&mut self) {
        self.current_fish.take();
    }

    fn current_fish(&mut self) -> &mut Fish {
        unsafe { self.current_fish.unwrap().as_mut() }.unwrap()
    }

    fn spawn_bullet(&mut self) {
        let fish = self.current_fish();
        let mut bullets = scene::find_node_by_type::<crate::nodes::Bullets>().unwrap();
        bullets.spawn_bullet(fish.pos, fish.facing);
    }
    fn set_sprite_fx(&self, s: bool) {
    }
    fn get_speed(&self) -> f32 {
        0.0
    }
    fn set_speed(&self, speed: f32) {
    }
    fn facing_dir(&self) -> f32 {
        0.0
    }
}

impl PluginRegistry {
    pub fn load(path: impl AsRef<Path>, item_registry: &mut ItemImplementationRegistry) -> Self {
        let mut plugins = HashMap::new();
        for entry in path.as_ref().read_dir().expect("Unable to read plugins directory") {
            if let Ok(entry) = entry {
                if entry.path().to_str().unwrap().contains(".wasm") {
                    let game_api = Arc::new(Mutex::new(GameApi::default()));
                    let mut builder = wasm_plugin_host::WasmPluginBuilder::from_file(entry.path()).expect(&format!("Failed to load plugin {:?}", entry.path()));
                    builder = builder.import_function_with_context("spawn_bullet", game_api.clone(), |ctx: &Arc<Mutex<GameApi>>| { ctx.lock().unwrap().spawn_bullet(); });
                    builder = builder.import_function_with_context("set_sprite_fx", game_api.clone(), |ctx: &Arc<Mutex<GameApi>>, s: bool| { ctx.lock().unwrap().set_sprite_fx(s); });
                    builder = builder.import_function_with_context("get_speed", game_api.clone(), |ctx: &Arc<Mutex<GameApi>>| { ctx.lock().unwrap().get_speed() });
                    builder = builder.import_function_with_context("set_speed", game_api.clone(), |ctx: &Arc<Mutex<GameApi>>, s: f32| { ctx.lock().unwrap().set_speed(s); });
                    builder = builder.import_function_with_context("facing_dir", game_api.clone(), |ctx: &Arc<Mutex<GameApi>>| { ctx.lock().unwrap().facing_dir() });
                    let mut plugin = builder
                        .finish()
                        .unwrap();
                    let description: PluginDescription = plugin.call_function("plugin_description").expect(&format!("Failed to call 'plugin_description' on plugin {:?}", entry.path()));

                    for item in description.items {
                        item_registry.add(item, description.plugin_id);
                    }

                    plugins.insert(description.plugin_id, Plugin {
                        wasm_plugin: plugin,
                        game_api
                    });
                }
            }
        }

        PluginRegistry(plugins)
    }

    pub(crate) fn get_plugin(&mut self, plugin_id: PluginId) -> Option<&mut Plugin> {
        self.0.get_mut(&plugin_id)
    }
}
