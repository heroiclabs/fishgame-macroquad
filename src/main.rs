use macroquad::prelude::*;

use macroquad_particles as particles;
use macroquad_profiler as profiler;
use macroquad_tiled as tiled;

use macroquad::{
    experimental::{collections::storage, scene},
    telemetry,
};

use particles::EmittersCache;
use physics_platformer::World as CollisionWorld;

mod nakama;

mod bullets;
mod level_background;
mod net_syncronizer;
mod player;

use bullets::Bullets;
use level_background::LevelBackground;
use net_syncronizer::NetSyncronizer;
use player::Player;

pub mod consts {
    pub const GRAVITY: f32 = 900.0;
    pub const JUMP_SPEED: f32 = 250.0;
    pub const RUN_SPEED: f32 = 150.0;
    pub const PLAYER_SPRITE: u32 = 120;
    pub const BULLET_SPEED: f32 = 300.0;

    pub const NETWORK_FPS: f32 = 15.0;
}

struct Resources {
    hit_fxses: EmittersCache,
    explosion_fxses: EmittersCache,
    tiled_map: tiled::Map,
    collision_world: CollisionWorld,
}

pub const HIT_FX: &'static str = r#"{"local_coords":false,"emission_shape":{"Point":[]},"one_shot":true,"lifetime":0.15,"lifetime_randomness":0,"explosiveness":0.65,"amount":41,"shape":{"Circle":{"subdivisions":10}},"emitting":false,"initial_direction":{"x":0,"y":-1},"initial_direction_spread":6.2831855,"initial_velocity":30,"initial_velocity_randomness":0.2,"linear_accel":0,"size":1.5000002,"size_randomness":0.4,"blend_mode":{"Alpha":[]},"colors_curve":{"start":{"r":0.8200004,"g":1,"b":0.31818175,"a":1},"mid":{"r":0.71000004,"g":0.36210018,"b":0,"a":1},"end":{"r":0.02,"g":0,"b":0.000000007152557,"a":1}},"gravity":{"x":0,"y":0},"post_processing":{}}
"#;

pub const EXPLOSION_FX: &'static str = r#"{"local_coords":false,"emission_shape":{"Sphere":{"radius":0.2}},"one_shot":true,"lifetime":0.3,"lifetime_randomness":0,"explosiveness":0.7,"amount":46,"shape":{"Circle":{"subdivisions":5}},"emitting":false,"initial_direction":{"x":0,"y":-1},"initial_direction_spread":6.2831855,"initial_velocity":132.40001,"initial_velocity_randomness":0.4,"linear_accel":-9.800002,"size":2,"size_randomness":0.3,"size_curve":{"points":[[0,1.5387659],[0.29,1.7387658],[1,0.6629627]],"interpolation":{"Linear":[]},"resolution":30},"blend_mode":{"Additive":[]},"colors_curve":{"start":{"r":0.93039775,"g":1,"b":0.13,"a":1},"mid":{"r":0.69,"g":0.08970088,"b":0.089701094,"a":1},"end":{"r":0.165132,"g":0.21016799,"b":0.18181819,"a":1}},"gravity":{"x":0,"y":-300},"post_processing":{}}
"#;

impl Resources {
    async fn new() -> Resources {
        let mut collision_world = CollisionWorld::new();

        let tileset = load_texture("assets/tileset.png").await;
        set_texture_filter(tileset, FilterMode::Nearest);

        let tiled_map_json = load_string("assets/map.json").await.unwrap();
        let tiled_map = tiled::load_map(&tiled_map_json, &[("tileset.png", tileset)], &[]).unwrap();

        let mut static_colliders = vec![];
        for (_x, _y, tile) in tiled_map.tiles("main layer", None) {
            static_colliders.push(tile.is_some());
        }
        collision_world.add_static_tiled_layer(static_colliders, 8., 8., 40, 1);

        let hit_fxses = EmittersCache::new(nanoserde::DeJson::deserialize_json(HIT_FX).unwrap());
        let explosion_fxses =
            EmittersCache::new(nanoserde::DeJson::deserialize_json(EXPLOSION_FX).unwrap());

        Resources {
            hit_fxses,
            explosion_fxses,
            tiled_map,
            collision_world,
        }
    }
}

#[macroquad::main("Platformer")]
async fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        while nakama::connected() == false {
            clear_background(BLACK);
            draw_text(
                &format!(
                    "Connecting {}",
                    ".".repeat(((get_time() * 2.0) as usize) % 4)
                ),
                screen_width() / 2.0 - 100.0,
                screen_height() / 2.0,
                40.,
                WHITE,
            );

            next_frame().await;
        }
    }

    rand::srand(get_time() as u64);

    let camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, 320.0, 152.0));

    storage::store(Resources::new().await);

    scene::add_node(LevelBackground::new());
    let player = scene::add_node(Player::new());
    scene::add_node(Bullets::new(player));
    scene::add_node(NetSyncronizer::new());

    loop {
        clear_background(BLACK);

        set_camera(camera);

        scene::update();

        {
            let _z = telemetry::ZoneGuard::new("draw particles");

            let mut resources = storage::get_mut::<Resources>().unwrap();

            resources.hit_fxses.draw();
            resources.explosion_fxses.draw();
        }

        set_default_camera();

        profiler::profiler(profiler::ProfilerParams {
            fps_counter_pos: vec2(50.0, 20.0),
        });

        next_frame().await;
    }
}
