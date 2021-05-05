use std::{collections::HashMap, io::Cursor, sync::Mutex};

use plugin_api::{
    import_game_api, AnimatedSpriteDescription, AnimationDescription, GameApi, ImageDescription,
    ItemDescription, ItemInstanceId, ItemType, PluginDescription, PluginId, Rect, SoundDescription,
};

lazy_static::lazy_static! {
    static ref ITEMS:Mutex<HashMap<ItemInstanceId, ItemState>> = Default::default();
}

const GUN: ItemType = ItemType::new(9868317461196439167);
const SWORD: ItemType = ItemType::new(11238048715746880612);

pub const GUN_THROWBACK: f32 = 700.0;

pub enum ItemState {
    Gun(GunState),
    Sword(SwordState),
}

#[cfg(not(feature = "inline"))]
use wasm_game_api::*;

#[cfg(not(feature = "inline"))]
mod wasm_game_api {
    use super::*;
    import_game_api!();

    pub struct GuestGameApi;
    impl GameApi for GuestGameApi {
        fn spawn_bullet(&self) {
            spawn_bullet();
        }

        fn hit_rect(&self, rect: [f32; 4]) -> u32 {
            hit_rect(rect)
        }

        fn set_sprite_fx(&self, s: bool) {
            set_sprite_fx(s);
        }

        fn get_speed(&self) -> [f32; 2] {
            get_speed()
        }

        fn set_speed(&self, speed: [f32; 2]) {
            set_speed(speed);
        }

        fn facing_dir(&self) -> f32 {
            facing_dir()
        }

        fn position(&self) -> [f32; 2] {
            position()
        }

        fn set_sprite_animation(&self, animation: u32) {
            set_sprite_animation(animation);
        }

        fn set_fx_sprite_animation(&self, animation: u32) {
            set_fx_sprite_animation(animation);
        }

        fn set_sprite_frame(&self, frame: u32) {
            set_sprite_frame(frame);
        }

        fn set_fx_sprite_frame(&self, frame: u32) {
            set_fx_sprite_frame(frame);
        }

        fn disarm(&self) {
            disarm();
        }

        fn play_sound_once(&self, sound: String) {
            play_sound_once(sound);
        }

        fn nakama_shoot(&self) {
            nakama_shoot();
        }

        fn debug_print(&self, message: String) {
            debug_print(message);
        }
    }
}
#[cfg_attr(not(feature = "inline"), wasm_plugin_guest::export_function)]
pub fn plugin_description() -> PluginDescription {
    let sword_image = image::load(
        Cursor::new(include_bytes!("../../assets/Whale/Sword(65x93).png")),
        image::ImageFormat::Png,
    )
    .unwrap()
    .to_rgba8();
    let sword_width = sword_image.width() as u16;
    let sword_height = sword_image.height() as u16;
    let sword_bytes = sword_image.into_vec();

    let gun_image = image::load(
        Cursor::new(include_bytes!("../../assets/Whale/Gun(92x32).png")),
        image::ImageFormat::Png,
    )
    .unwrap()
    .to_rgba8();
    let gun_width = gun_image.width() as u16;
    let gun_height = gun_image.height() as u16;
    let gun_bytes = gun_image.into_vec();

    PluginDescription {
        plugin_id: PluginId::new(11229058760733382699),
        display_name: "basic weapons".to_string(),
        sounds: vec![
            SoundDescription {
                name: "sword".to_string(),
                bytes: include_bytes!("../../assets/sounds/sword.wav").to_vec(),
            },
            SoundDescription {
                name: "shoot".to_string(),
                bytes: include_bytes!("../../assets/sounds/shoot.ogg").to_vec(),
            },
        ],
        items: vec![
            ItemDescription {
                item_type: SWORD,
                display_name: "Sword".to_string(),
                image: ImageDescription {
                    bytes: sword_bytes,
                    width: sword_width as u16,
                    height: sword_height as u16,
                },
                mount_pos_right: [10.0, -35.0],
                mount_pos_left: [-50.0, -35.0],
                pickup_src: Rect {
                    x: 200.0,
                    y: 98.0,
                    w: 55.0,
                    h: 83.0,
                },
                pickup_dst: [32.0, 32.0],
                sprite: AnimatedSpriteDescription {
                    tile_width: 65,
                    tile_height: 93,
                    animations: vec![
                        AnimationDescription {
                            name: "idle".to_string(),
                            row: 0,
                            frames: 1,
                            fps: 1,
                        },
                        AnimationDescription {
                            name: "shoot".to_string(),
                            row: 1,
                            frames: 4,
                            fps: 15,
                        },
                    ],
                    playing: true,
                },
                fx_sprite: None,
            },
            ItemDescription {
                item_type: GUN,
                display_name: "Gun".to_string(),
                image: ImageDescription {
                    bytes: gun_bytes,
                    width: gun_width as u16,
                    height: gun_height as u16,
                },
                mount_pos_right: [0.0, 16.0],
                mount_pos_left: [-60.0, 16.0],
                pickup_src: Rect {
                    x: 0.0,
                    y: 0.0,
                    w: 64.0,
                    h: 32.0,
                },
                pickup_dst: [32.0, 16.0],
                sprite: AnimatedSpriteDescription {
                    tile_width: 92,
                    tile_height: 32,
                    animations: vec![
                        AnimationDescription {
                            name: "idle".to_string(),
                            row: 0,
                            frames: 1,
                            fps: 1,
                        },
                        AnimationDescription {
                            name: "shoot".to_string(),
                            row: 1,
                            frames: 3,
                            fps: 15,
                        },
                    ],
                    playing: true,
                },
                fx_sprite: Some(AnimatedSpriteDescription {
                    tile_width: 76,
                    tile_height: 66,
                    animations: vec![AnimationDescription {
                        name: "shoot".to_string(),
                        row: 2,
                        frames: 3,
                        fps: 15,
                    }],
                    playing: true,
                }),
            },
        ],
    }
}

#[cfg_attr(not(feature = "inline"), wasm_plugin_guest::export_function)]
pub fn new_instance(item_type: ItemType, item_id: ItemInstanceId) {
    let state = match item_type {
        GUN => ItemState::Gun(GunState::default()),
        SWORD => ItemState::Sword(SwordState::default()),
        _ => panic!(),
    };

    ITEMS.lock().unwrap().insert(item_id, state);
}

#[cfg_attr(not(feature = "inline"), wasm_plugin_guest::export_function)]
pub fn destroy_instance(item_id: ItemInstanceId) {
    ITEMS.lock().unwrap().remove(&item_id);
}

#[cfg_attr(not(feature = "inline"), wasm_plugin_guest::export_function)]
pub fn uses_remaining(item_id: ItemInstanceId) -> Option<(u32, u32)> {
    if let Some(ItemState::Gun(state)) = ITEMS.lock().unwrap().get(&item_id) {
        Some((state.ammo, 3))
    } else {
        None
    }
}

#[cfg(not(feature = "inline"))]
#[wasm_plugin_guest::export_function]
pub fn update_shoot(item_id: ItemInstanceId, current_time: f64) -> bool {
    inner_update_shoot(item_id, current_time, &GuestGameApi)
}

#[cfg(feature = "inline")]
pub fn update_shoot(item_id: ItemInstanceId, current_time: f64, game_api: &dyn GameApi) -> bool {
    inner_update_shoot(item_id, current_time, game_api)
}

fn inner_update_shoot(item_id: ItemInstanceId, current_time: f64, game_api: &dyn GameApi) -> bool {
    if let Some(item) = ITEMS.lock().unwrap().get_mut(&item_id) {
        match item {
            ItemState::Gun(state) => {
                if let Some(time) = state.recovery_time {
                    if time <= current_time {
                        game_api.set_sprite_animation(0);
                        game_api.set_sprite_fx(false);
                        state.recovery_time.take();
                        if state.ammo == 0 {
                            game_api.disarm();
                        }
                        true
                    } else {
                        false
                    }
                } else {
                    state.ammo -= 1;
                    game_api.play_sound_once("shoot".to_string());
                    game_api.spawn_bullet();
                    game_api.nakama_shoot();
                    game_api.set_sprite_fx(true);
                    let mut speed = game_api.get_speed();
                    speed[0] -= GUN_THROWBACK * game_api.facing_dir();
                    game_api.set_speed(speed);
                    game_api.set_sprite_animation(1);
                    state.recovery_time = Some(current_time + 0.08 * 3.0);
                    false
                }
            }
            ItemState::Sword(state) => {
                if let Some(time) = state.recovery_time {
                    if time <= current_time {
                        game_api.set_sprite_animation(0);
                        state.recovery_time.take();
                        true
                    } else {
                        game_api.nakama_shoot();
                        game_api.play_sound_once("sword".to_string());
                        let pos = game_api.position();
                        let sword_hit_box = if game_api.facing_dir() > 0.0 {
                            [pos[0] + 35., pos[1] - 5., 40., 60.]
                        } else {
                            [pos[0] - 50., pos[1] - 5., 40., 60.]
                        };
                        game_api.hit_rect(sword_hit_box);
                        false
                    }
                } else {
                    game_api.set_sprite_animation(1);
                    state.recovery_time = Some(current_time + 0.08 * 3.0);
                    false
                }
            }
        }
    } else {
        true
    }
}

#[cfg(not(feature = "inline"))]
#[wasm_plugin_guest::export_function]
pub fn update_remote_shoot(item_id: ItemInstanceId, current_time: f64) -> bool {
    inner_update_remote_shoot(item_id, current_time, &GuestGameApi)
}

#[cfg(feature = "inline")]
pub fn update_remote_shoot(
    item_id: ItemInstanceId,
    current_time: f64,
    game_api: &dyn GameApi,
) -> bool {
    inner_update_remote_shoot(item_id, current_time, game_api)
}

fn inner_update_remote_shoot(
    item_id: ItemInstanceId,
    current_time: f64,
    game_api: &dyn GameApi,
) -> bool {
    if let Some(item) = ITEMS.lock().unwrap().get_mut(&item_id) {
        match item {
            ItemState::Gun(_) => {
                game_api.spawn_bullet();
                game_api.play_sound_once("shoot".to_string());
                true
            }
            ItemState::Sword(state) => {
                if let Some(time) = state.recovery_time {
                    if time <= current_time {
                        game_api.set_sprite_animation(0);
                        state.recovery_time.take();
                        true
                    } else {
                        false
                    }
                } else {
                    game_api.play_sound_once("sword".to_string());
                    game_api.set_sprite_animation(1);
                    state.recovery_time = Some(current_time + 0.08 * 3.0);
                    false
                }
            }
        }
    } else {
        true
    }
}

pub struct GunState {
    recovery_time: Option<f64>,
    ammo: u32,
}

impl Default for GunState {
    fn default() -> Self {
        Self {
            recovery_time: None,
            ammo: 3,
        }
    }
}

#[derive(Default)]
pub struct SwordState {
    recovery_time: Option<f64>,
}
