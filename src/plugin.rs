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
use crate::nodes::{ItemImplementationRegistry, Player};


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
    current_player: Arc<Mutex<Option<Handle<Player>>>>,
}
unsafe impl Send for GameApi {}
unsafe impl Sync for GameApi {}

impl GameApi {
    pub fn with_current_player<R>(&self, player: Handle<Player>, mut f: impl FnMut() -> R) -> R {
        self.current_player.lock().unwrap().replace(player);
        let result = f();
        self.current_player.lock().unwrap().take();
        result
    }

    fn current_player(&self) -> Handle<Player> {
        self.current_player.lock().unwrap().unwrap().clone()
    }

    fn spawn_bullet(&self) {
        let node = &mut *scene::get_node(self.current_player());
        let mut bullets = scene::find_node_by_type::<crate::nodes::Bullets>().unwrap();
        bullets.spawn_bullet(node.fish.pos, node.fish.facing);
    }

    fn set_sprite_fx(&self, s: bool) {
        let node = &mut *scene::get_node(self.current_player());
        if let Some(weapon) = &mut node.fish.weapon {
            weapon.fx = s;
        }
    }

    fn get_speed(&self) -> [f32; 2] {
        let node = &mut *scene::get_node(self.current_player());
        [node.fish.speed.x, node.fish.speed.y]
    }

    fn set_speed(&self, speed: [f32; 2]) {
        let node = &mut *scene::get_node(self.current_player());
        node.fish.speed.x = speed[0];
        node.fish.speed.y = speed[1];
    }

    fn set_sprite_animation(&self, animation: usize) {
        let node = &mut *scene::get_node(self.current_player());
        if let Some(weapon) = &mut node.fish.weapon {
            weapon.sprite.set_animation(animation);
        }
    }

    fn set_fx_sprite_animation(&self, animation: usize) {
        let node = &mut *scene::get_node(self.current_player());
        if let Some(weapon) = &mut node.fish.weapon {
            if let Some(sprite) = &mut weapon.fx_sprite {
                sprite.set_animation(animation);
            }
        }
    }

    fn set_sprite_frame(&self, frame: u32) {
        let node = &mut *scene::get_node(self.current_player());
        if let Some(weapon) = &mut node.fish.weapon {
            weapon.sprite.set_frame(frame);
        }
    }

    fn set_fx_sprite_frame(&self, frame: u32) {
        let node = &mut *scene::get_node(self.current_player());
        if let Some(weapon) = &mut node.fish.weapon {
            if let Some(sprite) = &mut weapon.fx_sprite {
                sprite.set_frame(frame);
            }
        }
    }

    fn facing_dir(&self) -> f32 {
        let node = &mut *scene::get_node(self.current_player());
        node.fish.facing_dir()
    }

    fn debug_print(&self, message: String) {
        println!("{}", message);
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
                    builder = builder.import_function_with_context("set_speed", game_api.clone(), |ctx: &GameApi, s: [f32; 2]| { ctx.set_speed(s); });
                    builder = builder.import_function_with_context("set_sprite_animation", game_api.clone(), |ctx: &GameApi, animation: usize| { ctx.set_sprite_animation(animation); });
                    builder = builder.import_function_with_context("set_fx_sprite_animation", game_api.clone(), |ctx: &GameApi, animation: usize| { ctx.set_fx_sprite_animation(animation); });
                    builder = builder.import_function_with_context("set_sprite_frame", game_api.clone(), |ctx: &GameApi, frame: u32| { ctx.set_sprite_frame(frame); });
                    builder = builder.import_function_with_context("set_fx_sprite_frame", game_api.clone(), |ctx: &GameApi, frame: u32| { ctx.set_fx_sprite_frame(frame); });
                    builder = builder.import_function_with_context("facing_dir", game_api.clone(), |ctx: &GameApi| { ctx.facing_dir() });
                    builder = builder.import_function_with_context("debug_print", game_api.clone(), |ctx: &GameApi, message: String| { ctx.debug_print(message) });
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
