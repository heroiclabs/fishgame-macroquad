use std::{
    sync::Mutex,
    io::Cursor,
    collections::HashMap,
};

use plugin_api::{ItemType, ImageDescription, AnimationDescription, AnimatedSpriteDescription, PluginDescription, PluginId, ItemDescription, Rect, ItemInstanceId, import_game_api, SoundDescription};


lazy_static::lazy_static! {
    static ref ITEMS:Mutex<HashMap<ItemInstanceId, ItemState>> = Default::default();
}

const GUN: ItemType = ItemType::new(9868317461196439167);
const SWORD: ItemType = ItemType::new(11238048715746880612);

pub const GUN_THROWBACK: f32 = 700.0;

import_game_api!();

pub enum ItemState {
    Gun(GunState),
    Sword(SwordState),
}

#[wasm_plugin_guest::export_function]
fn plugin_description() -> PluginDescription {
    let sword_image = image::load(Cursor::new(include_bytes!("../../assets/Whale/Sword(65x93).png")), image::ImageFormat::Png).unwrap().to_rgba8();
    let sword_width = sword_image.width() as u16;
    let sword_height = sword_image.height() as u16;
    let sword_bytes = sword_image.into_vec();

    let gun_image = image::load(Cursor::new(include_bytes!("../../assets/Whale/Gun(92x32).png")), image::ImageFormat::Png).unwrap().to_rgba8();
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
                    animations: vec![
                        AnimationDescription {
                            name: "shoot".to_string(),
                            row: 2,
                            frames: 3,
                            fps: 15,
                        },
                    ],
                    playing: true,
                }),
            }
        ],
    }
}

#[wasm_plugin_guest::export_function]
fn new_instance(item_type: ItemType, item_id: ItemInstanceId) {
    let state = match item_type {
        GUN => ItemState::Gun(GunState::default()),
        SWORD => ItemState::Sword(SwordState::default()),
        _ => panic!()
    };

    ITEMS.lock().unwrap().insert(item_id, state);
}

#[wasm_plugin_guest::export_function]
fn destroy_instance(item_id: ItemInstanceId) {
    ITEMS.lock().unwrap().remove(&item_id);
}

#[wasm_plugin_guest::export_function]
fn uses_remaining(item_id: ItemInstanceId) -> Option<(u32, u32)> {
    if let Some(ItemState::Gun(state)) = ITEMS.lock().unwrap().get(&item_id) {
        Some((state.ammo, 3))
    } else {
        None
    }
}

#[wasm_plugin_guest::export_function]
fn update_shoot(item_id: ItemInstanceId, current_time: f64) -> bool {
    if let Some(item) = ITEMS.lock().unwrap().get_mut(&item_id) {
        match item {
            ItemState::Gun(state) => {
                if let Some(time) = state.recovery_time {
                    if time <= current_time {
                        set_sprite_animation(0);
                        set_sprite_fx(false);
                        state.recovery_time.take();
                        if state.ammo == 0 {
                            disarm();
                        }
                        true
                    } else {
                        false
                    }
                } else {
                    state.ammo -= 1;
                    play_sound_once("shoot".to_string());
                    spawn_bullet();
                    nakama_shoot();
                    set_sprite_fx(true);
                    let mut speed = get_speed();
                    speed[0] -= GUN_THROWBACK * facing_dir();
                    set_speed(speed);
                    set_sprite_animation(1);
                    state.recovery_time = Some(current_time + 0.08 * 3.0);
                    false
                }
            },
            ItemState::Sword(state) => {
                if let Some(time) = state.recovery_time {
                    if time <= current_time {
                        set_sprite_animation(0);
                        state.recovery_time.take();
                        true
                    } else {
                        nakama_shoot();
                        play_sound_once("sword".to_string());
                        let pos = position();
                        let sword_hit_box = if facing_dir() > 0.0 {
                            [pos[0] + 35., pos[1] - 5., 40., 60.]
                        } else {
                            [pos[0] - 50., pos[1] - 5., 40., 60.]
                        };
                        hit_rect(sword_hit_box);
                        false
                    }
                } else {
                    set_sprite_animation(1);
                    state.recovery_time = Some(current_time + 0.08 * 3.0);
                    false
                }
            },
        }
    } else {
        true
    }
}

#[wasm_plugin_guest::export_function]
fn update_remote_shoot(item_id: ItemInstanceId, current_time: f64) -> bool {
    if let Some(item) = ITEMS.lock().unwrap().get_mut(&item_id) {
        match item {
            ItemState::Gun(_) => {
                spawn_bullet();
                play_sound_once("shoot".to_string());
                true
            },
            ItemState::Sword(state) => {
                if let Some(time) = state.recovery_time {
                    if time <= current_time {
                        set_sprite_animation(0);
                        state.recovery_time.take();
                        true
                    } else {
                        false
                    }
                } else {
                    play_sound_once("sword".to_string());
                    set_sprite_animation(1);
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
