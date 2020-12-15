use macroquad::prelude::*;

use macroquad_tiled as tiled;

use physics_platformer::*;

struct Player {
    collider: Actor,
    speed: Vec2,
}

mod consts {
    pub const GRAVITY: f32 = 900.0;
    pub const JUMP_SPEED: f32 = 250.0;
    pub const RUN_SPEED: f32 = 150.0;
    pub const PLAYER_SPRITE: u32 = 120;
}

#[macroquad::main("Platformer")]
async fn main() {
    let tileset = load_texture("assets/tileset.png").await;
    set_texture_filter(tileset, FilterMode::Nearest);

    let tiled_map_json = load_string("assets/map.json").await.unwrap();
    let tiled_map = tiled::load_map(&tiled_map_json, &[("tileset.png", tileset)], &[]).unwrap();

    let mut static_colliders = vec![];
    for (_x, _y, tile) in tiled_map.tiles("main layer", None) {
        static_colliders.push(tile.is_some());
    }

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
    };

    let camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, 320.0, 152.0));

    loop {
        clear_background(BLACK);

        set_camera(camera);

        tiled_map.draw_tiles("main layer", Rect::new(0.0, 0.0, 320.0, 152.0), None);

        // draw player
        {
            let pos = world.actor_pos(player.collider);
            if player.speed.x >= 0.0 {
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

        next_frame().await
    }
}
