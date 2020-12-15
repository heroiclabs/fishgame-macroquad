use macroquad::prelude::*;

use macroquad_particles as particles;
use macroquad_profiler as profiler;
use macroquad_tiled as tiled;

use physics_platformer::*;

use macroquad::telemetry;

use particles::Emitter;
use quad_net::client::QuadSocket;

struct Player {
    collider: Actor,
    speed: Vec2,
    facing_right: bool,
    shooting: bool,
}

mod consts {
    pub const GRAVITY: f32 = 900.0;
    pub const JUMP_SPEED: f32 = 250.0;
    pub const RUN_SPEED: f32 = 150.0;
    pub const PLAYER_SPRITE: u32 = 120;
}

const SHOOTING_FX: &str = r#"
{"local_coords":false,"emission_shape":{"Sphere":{"radius":1}},"one_shot":false,"lifetime":0.4,"lifetime_randomness":0.2,"explosiveness":0,"amount":27,"shape":{"Circle":{"subdivisions":20}},"emitting":true,"initial_direction":{"x":1,"y":0},"initial_direction_spread":0.1,"initial_velocity":337.3,"initial_velocity_randomness":0.3,"linear_accel":0,"size":3.3,"size_randomness":0,"size_curve":{"points":[[0,0.44000006],[0.22,0.72],[0.46,0.84296143],[0.7,1.1229614],[1,0]],"interpolation":{"Linear":[]},"resolution":30},"blend_mode":{"Additive":[]},"colors_curve":{"start":{"r":0.89240015,"g":0.97,"b":0,"a":1},"mid":{"r":1,"g":0.11639989,"b":0.059999943,"a":1},"end":{"r":0.1500001,"g":0.03149999,"b":0,"a":1}},"gravity":{"x":0,"y":0},"post_processing":{}}
"#;

#[macroquad::main("Platformer")]
async fn main() {
    let mut bullet_emitter =
        Emitter::new(nanoserde::DeJson::deserialize_json(SHOOTING_FX).unwrap());
    bullet_emitter.config.emitting = false;

    // hack for local testing
    // will spawn a local tcp/websocket server from the first client runned
    // second client will got panic from server thread, but this is fine, we need just one server for local tests
    // #[cfg(not(target_arch = "wasm32"))]
    // {
    //     use std::thread;

    //     thread::spawn(|| {
    //         server::tcp_main().unwrap();
    //     });
    //     thread::sleep(std::time::Duration::from_millis(100));
    // }
    // let _tcp_ip = "0.0.0.0:8090";
    // let _ws_ip = "ws://0.0.0.0:8091";

    let _tcp_ip = "173.0.157.169:8090";
    let _ws_ip = "ws://173.0.157.169:8091";

    #[cfg(not(target_arch = "wasm32"))]
    let mut socket = QuadSocket::connect(_tcp_ip).unwrap();
    #[cfg(target_arch = "wasm32")]
    let mut socket = QuadSocket::connect(_ws_ip).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        while socket.is_wasm_websocket_connected() == false {
            next_frame().await;
        }
    }

    socket.send_bin(&shared::Handshake {
        magic: shared::MAGIC,
        version: shared::PROTOCOL_VERSION,
    });

    let message = shared::Message::SpawnRequest;
    socket.send_bin(&message);
    let id = loop {
        if let Some(message) = socket.try_recv() {
            let message = nanoserde::DeBin::deserialize_bin(&message).unwrap();
            match message {
                shared::Message::Spawned(id) => break id,
                _ => panic!("Expected Messag::Spawned"),
            }
        }
        next_frame().await;
    };

    info!("Spawned with id {}", id);

    let tileset = load_texture("client/assets/tileset.png").await;
    set_texture_filter(tileset, FilterMode::Nearest);

    let tiled_map_json = load_string("client/assets/map.json").await.unwrap();
    let tiled_map = tiled::load_map(&tiled_map_json, &[("tileset.png", tileset)], &[]).unwrap();

    let mut static_colliders = vec![];
    for (_x, _y, tile) in tiled_map.tiles("main layer", None) {
        static_colliders.push(tile.is_some());
    }

    socket.send_bin(&shared::Message::Move(0, 0));

    let mut world = World::new();
    world.add_static_tiled_layer(static_colliders, 8., 8., 40, 1);

    let spawner_pos = {
        let objects = &tiled_map.layers["logic"].objects;
        let macroquad_tiled::Object::Rect {
            world_x, world_y, ..
        } = objects[rand::gen_range(0, objects.len()) as usize];

        vec2(world_x, world_y)
    };
    let mut player = Player {
        collider: world.add_actor(spawner_pos, 8, 8),
        speed: vec2(0., 0.),
        facing_right: true,
        shooting: false,
    };

    let camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, 320.0, 152.0));

    let mut players = vec![];

    loop {
        telemetry::begin_zone("Main loop");

        telemetry::begin_zone("network");
        if true {
            let pos = world.actor_pos(player.collider);
            let x = pos.x as u16
                + ((player.facing_right as u16) << 14)
                + ((player.shooting as u16) << 15);
            socket.send_bin(&shared::Message::Move(x, pos.y as u8));
        }

        while let Some(msg) = socket.try_recv_bin() {
            players = match msg {
                shared::Message::Players(players) => players,
                _ => panic!(),
            };
        }

        telemetry::end_zone();

        telemetry::begin_zone("draw world");
        clear_background(BLACK);

        set_camera(camera);

        for _ in 0..1 {
            tiled_map.draw_tiles("main layer", Rect::new(0.0, 0.0, 320.0, 152.0), None);
        }

        // draw player
        {
            let pos = world.actor_pos(player.collider);

            if player.speed.x < 0.0 {
                player.facing_right = false;
            }
            if player.speed.x > 0.0 {
                player.facing_right = true;
            }
            if player.facing_right {
                tiled_map.spr(
                    "tileset",
                    consts::PLAYER_SPRITE,
                    Rect::new(pos.x, pos.y, 8.0, 8.0),
                );
            } else {
                tiled_map.spr(
                    "tileset",
                    consts::PLAYER_SPRITE,
                    Rect::new(pos.x + 8.0, pos.y, -8.0, 8.0),
                );
            }

            player.shooting = is_key_down(KeyCode::LeftControl);
            if player.shooting {
                if player.facing_right {
                    bullet_emitter.config.initial_direction = vec2(1.0, 0.0);
                } else {
                    bullet_emitter.config.initial_direction = vec2(-1.0, 0.0);
                }
                bullet_emitter.emit(pos + vec2(8.0 * player.facing_right as u8 as f32, 4.0), 1);
            }
        }

        // draw other players
        for (other_id, (x, y)) in players.iter().enumerate() {
            let facing_right = ((x >> 14) & 1) != 0;
            let shooting = ((x >> 15) & 1) != 0;

            let x = x & 0x3fff;

            draw_text_ex(
                &format!("player {}", other_id),
                x as f32 - 4.0,
                *y as f32 - 6.0,
                TextParams {
                    font_size: 30,
                    font_scale: 0.15,
                    ..Default::default()
                },
            );
            if facing_right {
                tiled_map.spr(
                    "tileset",
                    consts::PLAYER_SPRITE,
                    Rect::new(x as f32, *y as f32, 8.0, 8.0),
                );
            } else {
                tiled_map.spr(
                    "tileset",
                    consts::PLAYER_SPRITE,
                    Rect::new(x as f32 + 8.0, *y as f32, -8.0, 8.0),
                );
            }
            if shooting {
                if facing_right {
                    bullet_emitter.config.initial_direction = vec2(1.0, 0.0);
                } else {
                    bullet_emitter.config.initial_direction = vec2(-1.0, 0.0);
                }
                bullet_emitter.emit(
                    vec2(x as f32, *y as f32) + vec2(8.0 * facing_right as u8 as f32, 4.0),
                    1,
                );
            }
        }

        // player movement control
        {
            let pos = world.actor_pos(player.collider);
            let on_ground = world.collide_check(player.collider, pos + vec2(0., 1.));

            if on_ground == false {
                player.speed.y += consts::GRAVITY * get_frame_time();
            }

            if is_key_down(KeyCode::Right) {
                player.speed.x = consts::RUN_SPEED;
            } else if is_key_down(KeyCode::Left) {
                player.speed.x = -consts::RUN_SPEED;
            } else {
                player.speed.x = 0.;
            }

            if is_key_pressed(KeyCode::Space) {
                if on_ground {
                    player.speed.y = -consts::JUMP_SPEED;
                }
            }

            world.move_h(player.collider, player.speed.x * get_frame_time());
            if !world.move_v(player.collider, player.speed.y * get_frame_time()) {
                player.speed.y = 0.0;
            }
        }

        telemetry::end_zone();

        telemetry::begin_zone("draw particles");
        bullet_emitter.draw(vec2(0., 0.));
        telemetry::end_zone();

        set_default_camera();

        profiler::profiler(profiler::ProfilerParams {
            fps_counter_pos: vec2(50.0, 20.0),
        });

        telemetry::end_zone();

        next_frame().await;
    }
}
