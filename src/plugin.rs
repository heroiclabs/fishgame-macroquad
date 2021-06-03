use macroquad::{
    audio::{load_sound_from_bytes, play_sound_once, Sound},
    experimental::{
        animation::{AnimatedSprite, Animation},
        scene::{self, Handle, RefMut},
    },
    prelude::*,
    texture::Image,
};
use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
};

use crate::nodes::{Fish, ItemImplementationRegistry, Player, RemotePlayer};
use plugin_api::{
    AnimatedSpriteDescription, AnimationDescription, GameApi, ImageDescription, ItemInstanceId,
    ItemType, PluginApi, PluginDescription, PluginId,
};

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
    let animations: Vec<Animation> = desc
        .animations
        .into_iter()
        .map(|a| animation_from_desc(a))
        .collect();
    AnimatedSprite::new(desc.tile_width, desc.tile_height, &animations, desc.playing)
}

#[derive(Default, Clone)]
pub struct HostGameApi {
    current_player: Arc<Mutex<Option<Handle<Player>>>>,
    current_remote_player: Arc<Mutex<Option<Handle<RemotePlayer>>>>,
    sounds: Arc<Mutex<HashMap<String, Sound>>>,
}
unsafe impl Send for HostGameApi {}
unsafe impl Sync for HostGameApi {}

enum LocalOrRemotePlayer {
    Local(RefMut<Player>),
    Remote(RefMut<RemotePlayer>),
}
struct FishMut {
    node: LocalOrRemotePlayer,
}
impl std::ops::Deref for FishMut {
    type Target = Fish;

    fn deref(&self) -> &Self::Target {
        match &self.node {
            LocalOrRemotePlayer::Local(player) => &player.fish,
            LocalOrRemotePlayer::Remote(player) => &player.fish,
        }
    }
}
impl std::ops::DerefMut for FishMut {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.node {
            LocalOrRemotePlayer::Local(player) => &mut player.fish,
            LocalOrRemotePlayer::Remote(player) => &mut player.fish,
        }
    }
}

impl HostGameApi {
    fn current_player(&self) -> Option<Handle<Player>> {
        self.current_player.lock().unwrap().map(|p| p.clone())
    }

    fn current_remote_player(&self) -> Option<Handle<RemotePlayer>> {
        self.current_remote_player
            .lock()
            .unwrap()
            .map(|p| p.clone())
    }

    fn current_fish(&self) -> FishMut {
        if let Some(handle) = self.current_player() {
            let node = scene::get_node(handle);
            FishMut {
                node: LocalOrRemotePlayer::Local(node),
            }
        } else {
            let node = scene::get_node(self.current_remote_player().unwrap());
            FishMut {
                node: LocalOrRemotePlayer::Remote(node),
            }
        }
    }
}

impl GameApi for HostGameApi {
    fn spawn_bullet(&self) {
        let mut bullets = scene::find_node_by_type::<crate::nodes::Bullets>().unwrap();
        let fish = self.current_fish();
        bullets.spawn_bullet(fish.pos, fish.facing);
    }

    fn hit_rect(&self, rect: [f32; 4]) -> u32 {
        if let Some(handle) = self.current_player() {
            let hit_box = Rect::new(rect[0], rect[1], rect[2], rect[3]);
            let node = &mut *scene::get_node(handle);
            let others = scene::find_nodes_by_type::<crate::nodes::RemotePlayer>();
            let mut hit_count = 0;
            for player in others {
                if Rect::new(player.pos().x, player.pos().y, 20., 64.).overlaps(&hit_box) {
                    hit_count += 1;
                    let mut net = scene::get_node(node.nakama_realtime);
                    net.kill(&player.id, !node.fish.facing);
                }
            }
            hit_count
        } else {
            0
        }
    }

    fn set_sprite_fx(&self, s: bool) {
        let mut fish = self.current_fish();
        if let Some(weapon) = &mut fish.weapon {
            weapon.fx = s;
        }
    }

    fn get_speed(&self) -> [f32; 2] {
        let fish = self.current_fish();
        [fish.speed.x, fish.speed.y]
    }

    fn set_speed(&self, speed: [f32; 2]) {
        let mut fish = self.current_fish();
        fish.speed.x = speed[0];
        fish.speed.y = speed[1];
    }

    fn set_sprite_animation(&self, animation: u32) {
        let mut fish = self.current_fish();
        if let Some(weapon) = &mut fish.weapon {
            weapon.sprite.set_animation(animation as usize);
        }
    }

    fn set_fx_sprite_animation(&self, animation: u32) {
        let mut fish = self.current_fish();
        if let Some(weapon) = &mut fish.weapon {
            if let Some(sprite) = &mut weapon.fx_sprite {
                sprite.set_animation(animation as usize);
            }
        }
    }

    fn set_sprite_frame(&self, frame: u32) {
        let mut fish = self.current_fish();
        if let Some(weapon) = &mut fish.weapon {
            weapon.sprite.set_frame(frame);
        }
    }

    fn set_fx_sprite_frame(&self, frame: u32) {
        let mut fish = self.current_fish();
        if let Some(weapon) = &mut fish.weapon {
            if let Some(sprite) = &mut weapon.fx_sprite {
                sprite.set_frame(frame);
            }
        }
    }

    fn facing_dir(&self) -> f32 {
        let fish = self.current_fish();
        fish.facing_dir()
    }

    fn position(&self) -> [f32; 2] {
        let fish = self.current_fish();
        let pos = fish.pos();
        [pos.x, pos.y]
    }

    fn nakama_shoot(&self) {
        if let Some(handle) = self.current_player() {
            let node = &mut *scene::get_node(handle);
            let mut nakama = scene::get_node(node.nakama_realtime);
            nakama.shoot();
        }
    }

    fn disarm(&self) {
        let mut fish = self.current_fish();
        fish.disarm();
    }

    fn play_sound_once(&self, name: String) {
        if let Some(sound) = self.sounds.lock().unwrap().get(&name) {
            play_sound_once(*sound);
        }
    }

    fn debug_print(&self, message: String) {
        println!("{}", message);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native_host::*;
#[cfg(target_arch = "wasm32")]
pub use wasm_host::*;

#[cfg(not(target_arch = "wasm32"))]
mod native_host {
    use super::*;

    use wasm_plugin_host::WasmPlugin;

    pub struct PluginRegistry(HashMap<PluginId, Plugin>);

    pub struct Plugin {
        pub wasm_plugin: WasmPlugin,
        pub game_api: HostGameApi,
    }

    impl PluginRegistry {
        pub async fn load(
            path: impl AsRef<Path>,
            item_registry: &mut ItemImplementationRegistry,
        ) -> Self {
            let mut plugins = HashMap::new();
            for entry in path
                .as_ref()
                .read_dir()
                .expect("Unable to read plugins directory")
            {
                if let Ok(entry) = entry {
                    if entry.path().to_str().unwrap().contains(".wasm") {
                        let game_api = HostGameApi::default();
                        let mut builder =
                            wasm_plugin_host::WasmPluginBuilder::from_file(entry.path())
                                .expect(&format!("Failed to load plugin {:?}", entry.path()));
                        // TODO: This should probably be a macro or something to reduce boilerplate
                        builder = builder.import_function_with_context(
                            "spawn_bullet",
                            game_api.clone(),
                            |ctx: &HostGameApi| {
                                ctx.spawn_bullet();
                            },
                        );
                        builder = builder.import_function_with_context(
                            "hit_rect",
                            game_api.clone(),
                            |ctx: &HostGameApi, rect: [f32; 4]| ctx.hit_rect(rect),
                        );
                        builder = builder.import_function_with_context(
                            "set_sprite_fx",
                            game_api.clone(),
                            |ctx: &HostGameApi, s: bool| {
                                ctx.set_sprite_fx(s);
                            },
                        );
                        builder = builder.import_function_with_context(
                            "get_speed",
                            game_api.clone(),
                            |ctx: &HostGameApi| ctx.get_speed(),
                        );
                        builder = builder.import_function_with_context(
                            "set_speed",
                            game_api.clone(),
                            |ctx: &HostGameApi, s: [f32; 2]| {
                                ctx.set_speed(s);
                            },
                        );
                        builder = builder.import_function_with_context(
                            "set_sprite_animation",
                            game_api.clone(),
                            |ctx: &HostGameApi, animation: u32| {
                                ctx.set_sprite_animation(animation);
                            },
                        );
                        builder = builder.import_function_with_context(
                            "set_fx_sprite_animation",
                            game_api.clone(),
                            |ctx: &HostGameApi, animation: u32| {
                                ctx.set_fx_sprite_animation(animation);
                            },
                        );
                        builder = builder.import_function_with_context(
                            "set_sprite_frame",
                            game_api.clone(),
                            |ctx: &HostGameApi, frame: u32| {
                                ctx.set_sprite_frame(frame);
                            },
                        );
                        builder = builder.import_function_with_context(
                            "set_fx_sprite_frame",
                            game_api.clone(),
                            |ctx: &HostGameApi, frame: u32| {
                                ctx.set_fx_sprite_frame(frame);
                            },
                        );
                        builder = builder.import_function_with_context(
                            "facing_dir",
                            game_api.clone(),
                            |ctx: &HostGameApi| ctx.facing_dir(),
                        );
                        builder = builder.import_function_with_context(
                            "position",
                            game_api.clone(),
                            |ctx: &HostGameApi| ctx.position(),
                        );
                        builder = builder.import_function_with_context(
                            "disarm",
                            game_api.clone(),
                            |ctx: &HostGameApi| ctx.disarm(),
                        );
                        builder = builder.import_function_with_context(
                            "play_sound_once",
                            game_api.clone(),
                            |ctx: &HostGameApi, sound: String| {
                                ctx.play_sound_once(sound);
                            },
                        );
                        builder = builder.import_function_with_context(
                            "nakama_shoot",
                            game_api.clone(),
                            |ctx: &HostGameApi| ctx.nakama_shoot(),
                        );
                        builder = builder.import_function_with_context(
                            "debug_print",
                            game_api.clone(),
                            |ctx: &HostGameApi, message: String| ctx.debug_print(message),
                        );
                        let mut plugin = builder.finish().unwrap();
                        let description: PluginDescription =
                            plugin.call_function("plugin_description").expect(&format!(
                                "Failed to call 'plugin_description' on plugin {:?}",
                                entry.path()
                            ));

                        for item in description.items {
                            item_registry.add(item, description.plugin_id);
                        }

                        {
                            let mut sounds = game_api.sounds.lock().unwrap();
                            for sound in description.sounds {
                                sounds.insert(
                                    sound.name,
                                    load_sound_from_bytes(&sound.bytes).await.unwrap(),
                                );
                            }
                        }

                        plugins.insert(
                            description.plugin_id,
                            Plugin {
                                wasm_plugin: plugin,
                                game_api,
                            },
                        );
                    }
                }
            }

            PluginRegistry(plugins)
        }

        pub(crate) fn get_plugin(&mut self, plugin_id: PluginId) -> Option<&mut Plugin> {
            self.0.get_mut(&plugin_id)
        }
    }

    impl Plugin {
        pub fn with_current_player<R>(
            &mut self,
            player: Handle<Player>,
            mut f: impl FnMut(&mut Plugin) -> R,
        ) -> R {
            self.game_api.current_player.lock().unwrap().replace(player);
            let result = f(self);
            self.game_api.current_player.lock().unwrap().take();
            result
        }

        pub fn with_current_remote_player<R>(
            &mut self,
            player: Handle<RemotePlayer>,
            mut f: impl FnMut(&mut Plugin) -> R,
        ) -> R {
            self.game_api
                .current_remote_player
                .lock()
                .unwrap()
                .replace(player);
            let result = f(self);
            self.game_api.current_remote_player.lock().unwrap().take();
            result
        }
    }

    impl PluginApi for Plugin {
        fn new_instance(&mut self, item_type: ItemType, item_id: ItemInstanceId) {
            self.wasm_plugin
                .call_function_with_argument("new_instance", &(item_type, item_id))
                .unwrap()
        }

        fn destroy_instance(&mut self, item_id: ItemInstanceId) {
            self.wasm_plugin
                .call_function_with_argument("destroy_instance", &item_id)
                .unwrap()
        }

        fn uses_remaining(&mut self, item_id: ItemInstanceId) -> Option<(u32, u32)> {
            self.wasm_plugin
                .call_function_with_argument("uses_remaining", &item_id)
                .unwrap()
        }

        fn update_shoot(&mut self, item_id: ItemInstanceId, time: f64) -> bool {
            self.wasm_plugin
                .call_function_with_argument("update_shoot", &(item_id, time))
                .unwrap()
        }

        fn update_remote_shoot(&mut self, item_id: ItemInstanceId, time: f64) -> bool {
            self.wasm_plugin
                .call_function_with_argument("update_remote_shoot", &(item_id, time))
                .unwrap()
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm_host {
    use super::*;
    use core_plugin::{
        destroy_instance, new_instance, plugin_description, update_remote_shoot, update_shoot,
        uses_remaining,
    };

    pub struct Plugin {
        pub game_api: HostGameApi,
    }

    impl Plugin {
        pub fn with_current_player<R>(
            &mut self,
            player: Handle<Player>,
            mut f: impl FnMut(&mut Plugin) -> R,
        ) -> R {
            self.game_api.current_player.lock().unwrap().replace(player);
            let result = f(self);
            self.game_api.current_player.lock().unwrap().take();
            result
        }

        pub fn with_current_remote_player<R>(
            &mut self,
            player: Handle<RemotePlayer>,
            mut f: impl FnMut(&mut Plugin) -> R,
        ) -> R {
            self.game_api
                .current_remote_player
                .lock()
                .unwrap()
                .replace(player);
            let result = f(self);
            self.game_api.current_remote_player.lock().unwrap().take();
            result
        }
    }

    impl PluginApi for Plugin {
        fn new_instance(&mut self, item_type: ItemType, item_id: ItemInstanceId) {
            new_instance(item_type, item_id);
        }

        fn destroy_instance(&mut self, item_id: ItemInstanceId) {
            destroy_instance(item_id);
        }

        fn uses_remaining(&mut self, item_id: ItemInstanceId) -> Option<(u32, u32)> {
            uses_remaining(item_id)
        }

        fn update_shoot(&mut self, item_id: ItemInstanceId, time: f64) -> bool {
            update_shoot(item_id, time, &self.game_api)
        }

        fn update_remote_shoot(&mut self, item_id: ItemInstanceId, time: f64) -> bool {
            update_remote_shoot(item_id, time, &self.game_api)
        }
    }

    pub struct PluginRegistry;
    impl PluginRegistry {
        pub async fn load(
            _path: impl AsRef<Path>,
            _item_registry: &mut ItemImplementationRegistry,
        ) -> Self {
            PluginRegistry
        }
        pub(crate) fn get_plugin(&mut self, plugin_id: PluginId) -> Option<&mut Plugin> {
            None
        }
    }
}
