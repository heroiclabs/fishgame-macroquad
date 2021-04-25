use std::{
    io::Cursor,
    sync::Mutex, collections::HashMap,
};

use plugin_api::{ItemType, ImageDescription, AnimationDescription, AnimatedSpriteDescription, PluginDescription, PluginId, ItemDescription, Rect, ItemInstanceId, import_game_api};

use once_cell::sync::Lazy;

// The Mutex here is really not necessary since this is guaranteed to be a single
// threaded environment, I'm just avoiding writing unsafe blocks. A library for
// writing these plugins could probably include a more efficient state store
// that takes advantage of the single threadedness.
//
// An alternative design would be for the new_instance function to allocate state on
// the heap and return a pointer which would then get passed back in when other
// functions are called. I think I like having the non-pointer key and letting the
// plugin interpret that however it sees fit but you could argue for the other version.
static ITEMS: Lazy<Mutex<HashMap<ItemInstanceId, ItemState>>> = Lazy::new(Mutex::default);

const GUN: ItemType = ItemType::new(9868317461196439167);
const SWORD: ItemType = ItemType::new(11238048715746880612);

pub const GUN_THROWBACK: f32 = 700.0;

import_game_api!();

enum ItemState {
    Gun(u32),
    Sword,
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
                    ],
                    playing: true,
                },
                fx_sprite: AnimatedSpriteDescription {
                    tile_width: 76,
                    tile_height: 66,
                    animations: vec![
                    ],
                    playing: true,
                },
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
                    ],
                    playing: true,
                },
                fx_sprite: AnimatedSpriteDescription {
                    tile_width: 76,
                    tile_height: 66,
                    animations: vec![
                    ],
                    playing: true,
                },
            }
        ],
    }
}

#[wasm_plugin_guest::export_function]
fn new_instance(item_type: ItemType, item_id: ItemInstanceId) {
    let state = match item_type {
        GUN => ItemState::Gun(3),
        SWORD => ItemState::Sword,
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
    if let Some(ItemState::Gun(ammo)) = ITEMS.lock().unwrap().get(&item_id) {
        Some((*ammo, 3))
    } else {
        None
    }
}

#[wasm_plugin_guest::export_function]
fn update_shoot(item_id: ItemInstanceId) -> bool {
    if let Some(item) = ITEMS.lock().unwrap().get(&item_id) {
        match item {
            ItemState::Gun(_) => gun_handler(),
            ItemState::Sword => true,
        }
    } else {
        true
    }
}

fn gun_handler() -> bool {
    spawn_bullet();
    let mut speed = get_speed();
    speed += GUN_THROWBACK * facing_dir();
    true
}
