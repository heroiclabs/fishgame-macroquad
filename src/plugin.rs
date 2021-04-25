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
    pub game_api: GameApi,
}

#[derive(Default, Clone)]
pub struct GameApi {
    current_fish: Arc<Mutex<Option<*mut Fish>>>,
}
unsafe impl Send for GameApi {}
unsafe impl Sync for GameApi {}

pub struct CurrentFishGuard(GameApi);
impl Drop for CurrentFishGuard {
    fn drop(&mut self) {
        self.0.clear_current_fish();
    }
}

impl GameApi {
    pub fn set_current_fish(&self, player: &mut Fish) -> CurrentFishGuard {
        self.current_fish.lock().unwrap().replace(player as *mut Fish);
        CurrentFishGuard(self.clone())
    }
    pub fn clear_current_fish(&self) {
        self.current_fish.lock().unwrap().take();
    }

    fn with_current_fish(&self, f: impl Fn(&mut Fish)) {
        let fish_guard = self.current_fish.lock().unwrap();
        let fish_ptr: *mut Fish = fish_guard.unwrap();
        let fish_ref = unsafe { fish_ptr.as_mut().unwrap() };
        f(fish_ref);
    }

    fn spawn_bullet(&self) {
        self.with_current_fish(|fish| {
            let mut bullets = scene::find_node_by_type::<crate::nodes::Bullets>().unwrap();
            bullets.spawn_bullet(fish.pos, fish.facing);
        });
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
                    let game_api = GameApi::default();
                    let mut builder = wasm_plugin_host::WasmPluginBuilder::from_file(entry.path()).expect(&format!("Failed to load plugin {:?}", entry.path()));
                    builder = builder.import_function_with_context("spawn_bullet", game_api.clone(), |ctx: &GameApi| { ctx.spawn_bullet(); });
                    builder = builder.import_function_with_context("set_sprite_fx", game_api.clone(), |ctx: &GameApi, s: bool| { ctx.set_sprite_fx(s); });
                    builder = builder.import_function_with_context("get_speed", game_api.clone(), |ctx: &GameApi| { ctx.get_speed() });
                    builder = builder.import_function_with_context("set_speed", game_api.clone(), |ctx: &GameApi, s: f32| { ctx.set_speed(s); });
                    builder = builder.import_function_with_context("facing_dir", game_api.clone(), |ctx: &GameApi| { ctx.facing_dir() });
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
