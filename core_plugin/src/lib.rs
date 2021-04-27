use std::{
    future::Future,
    sync::Arc,
    cell::{RefCell, Cell},
    io::Cursor,
    collections::HashMap,
    pin::Pin,
    task::{Context, Poll},
};
use async_executor::{LocalExecutor, Task};
use futures_lite::future;

use plugin_api::{ItemType, ImageDescription, AnimationDescription, AnimatedSpriteDescription, PluginDescription, PluginId, ItemDescription, Rect, ItemInstanceId, import_game_api};


// The Mutex here is really not necessary since this is guaranteed to be a single
// threaded environment, I'm just avoiding writing unsafe blocks. A library for
// writing these plugins could probably include a more efficient state store
// that takes advantage of the single threadedness.
//
// An alternative design would be for the new_instance function to allocate state on
// the heap and return a pointer which would then get passed back in when other
// functions are called. I think I like having the non-pointer key and letting the
// plugin interpret that however it sees fit but you could argue for the other version.
thread_local! {
        pub static ITEMS:RefCell<HashMap<ItemInstanceId, ItemState>> = Default::default();
}

const GUN: ItemType = ItemType::new(9868317461196439167);
const SWORD: ItemType = ItemType::new(11238048715746880612);

pub const GUN_THROWBACK: f32 = 700.0;

import_game_api!();

pub enum ItemState {
    Gun(ItemCoroutine, GunState),
    Sword(ItemCoroutine),
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
        GUN => ItemState::Gun(ItemCoroutine::default(), GunState::default()),
        SWORD => ItemState::Sword(ItemCoroutine::default()),
        _ => panic!()
    };

    ITEMS.with(|items| items.borrow_mut().insert(item_id, state));
}

#[wasm_plugin_guest::export_function]
fn destroy_instance(item_id: ItemInstanceId) {
    ITEMS.with(|items| items.borrow_mut().remove(&item_id));
}

#[wasm_plugin_guest::export_function]
fn uses_remaining(item_id: ItemInstanceId) -> Option<(u32, u32)> {
    ITEMS.with(|items| {
        if let Some(ItemState::Gun(_, state)) = items.borrow_mut().get(&item_id) {
            Some((state.ammo, 3))
        } else {
            None
        }
    })
}

#[wasm_plugin_guest::export_function]
fn update_shoot(item_id: ItemInstanceId, delta_time: f32) -> bool {
    ITEMS.with(|items| {
        if let Some(item) = items.borrow_mut().get_mut(&item_id) {
            match item {
                ItemState::Gun(coroutine, state) => {
                    if !coroutine.started() {
                        state.ammo -= 1;
                        coroutine.start(gun_coroutine);
                    }
                    coroutine.step(delta_time)
                },
                ItemState::Sword(coroutine) => {
                    if !coroutine.started() {
                        coroutine.start(sword_coroutine);
                    }
                    coroutine.step(delta_time)
                },
            }
        } else {
            true
        }
    })
}

pub struct ItemCoroutine {
    executor: LocalExecutor<'static>,
    timer: Timer,
    coroutine: Option<Task<()>>,
}

impl Default for ItemCoroutine {
    fn default() -> Self {
        Self {
            executor: LocalExecutor::new(),
            timer: Timer::default(),
            coroutine: None,
        }
    }
}

impl ItemCoroutine {
    fn step(&mut self, delta_time: f32) -> bool {
        self.timer.incr_time(delta_time);
        debug_print(format!("poop {}", self.timer.time.get()));
        if self.coroutine.is_none() {
            return true;
        }
        let r = self.executor.try_tick();
        debug_print(format!("poop {} {}", self.executor.is_empty(), r));
        if self.executor.is_empty() {
            self.coroutine.take();
            true
        } else {
            false
        }
    }

    fn started(&self) -> bool {
        self.coroutine.is_some()
    }

    fn start<F, Fu>(&mut self, f: F)
    where
        Fu: Future<Output=()>,
        F: Fn(Timer) -> Fu + 'static
    {
        let timer = self.timer.clone();
        self.coroutine = Some(self.executor.spawn(async move {
            f(timer).await
        }));
    }
}

pub struct GunState {
    ammo: u32,
}

impl Default for GunState {
    fn default() -> Self {
        Self {
            ammo: 3,
        }
    }
}

#[derive(Clone, Default)]
struct Timer {
    time: Arc<Cell<f32>>,
}

impl Timer {
    async fn wait_seconds(&self, seconds: f32) {
        let timer = self.clone();
        let start_time = timer.time.get();
        future::poll_fn(move |_cx| {
            if timer.time.get() - start_time >= seconds {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        }).await;
    }

    fn incr_time(&self, delta: f32) {
        let current_time = self.time.get();
        self.time.set(current_time + delta);
    }
}

pub struct TimerDelayFuture {
    start_time: f32,
    time: f32,
    timer: Timer,
}
impl Unpin for TimerDelayFuture {}

impl Future for TimerDelayFuture {
    type Output = Option<()>;

    fn poll(self: Pin<&mut Self>, _: &mut Context) -> Poll<Self::Output> {
        debug_print(format!("{} {} {}", self.timer.time.get(),  self.start_time, self.time));
        if self.timer.time.get() - self.start_time >= self.time {
            Poll::Ready(Some(()))
        } else {
            //Poll::Pending
            Poll::Ready(Some(()))
        }
    }
}

async fn gun_coroutine(timer: Timer) {
    spawn_bullet();
    set_sprite_fx(true);
    let mut speed = get_speed();
    speed[0] -= GUN_THROWBACK * facing_dir();
    set_speed(speed);
    set_sprite_animation(1);

    for i in 0u32..3 {
        set_sprite_frame(i);
        set_fx_sprite_frame(i);
        debug_print("gun pre wait".to_string());
        timer.wait_seconds(0.08).await;
        debug_print("gun post wait".to_string());
    }
    set_sprite_animation(0);
    set_sprite_fx(false);
    debug_print("gun done".to_string());
}

async fn sword_coroutine(timer: Timer) {
    set_sprite_animation(1);
    for i in 0u32..3 {
        set_sprite_frame(i);
        timer.wait_seconds(0.08).await;
    }
    set_sprite_animation(0);
    debug_print("sword done".to_string());
}
