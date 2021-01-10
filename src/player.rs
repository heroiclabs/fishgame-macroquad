use macroquad::{
    experimental::{
        collections::storage,
        coroutines::{start_coroutine, wait_seconds, Coroutine},
        scene,
        state_machine::{State, StateMachine},
    },
    prelude::*,
};
use physics_platformer::Actor;

use crate::{consts, Resources};

pub(crate) struct Player {
    collider: Actor,
    pos: Vec2,
    speed: Vec2,
    facing: bool,
    health: i32,

    state_machine: StateMachine<Player>,
}

impl Player {
    const ST_NORMAL: usize = 0;
    const ST_DEATH: usize = 1;

    pub fn new() -> Player {
        let mut resources = storage::get_mut::<Resources>().unwrap();
        let spawner_pos = {
            let objects = &resources.tiled_map.layers["logic"].objects;
            let macroquad_tiled::Object::Rect {
                world_x, world_y, ..
            } = objects[rand::gen_range(0, objects.len()) as usize];

            vec2(world_x, world_y)
        };

        let mut state_machine = StateMachine::new();
        state_machine.add_state(Self::ST_NORMAL, State::new().update(Self::update_normal));
        state_machine.add_state(
            Self::ST_DEATH,
            State::new().coroutine(Self::death_coroutine),
        );

        Player {
            collider: resources.collision_world.add_actor(spawner_pos, 8, 8),
            pos: spawner_pos,
            speed: vec2(0., 0.),
            facing: true,
            health: 100,

            state_machine,
        }
    }

    pub fn pos(&self) -> Vec2 {
        self.pos
    }

    pub fn facing(&self) -> bool {
        self.facing
    }

    pub fn health(&self) -> i32 {
        self.health
    }

    pub fn damage(&mut self, amount: i32) {
        self.health -= amount;
    }

    fn death_coroutine(&mut self) -> Coroutine {
        let pos = self.pos;

        let coroutine = async move {
            {
                let mut resources = storage::get_mut::<Resources>().unwrap();

                resources.explosion_fxses.spawn(pos + vec2(4., 4.));
            }

            wait_seconds(0.5).await;

            let mut resources = storage::get_mut::<Resources>().unwrap();
            let mut this = coroutines::active_node::<Player>().unwrap();

            this.pos = {
                let objects = &resources.tiled_map.layers["logic"].objects;
                let macroquad_tiled::Object::Rect {
                    world_x, world_y, ..
                } = objects[rand::gen_range(0, objects.len()) as usize];

                vec2(world_x, world_y)
            };
            this.health = 100;
            resources
                .collision_world
                .set_actor_position(this.collider, this.pos);
            this.state_machine.set_state(Self::ST_NORMAL);
        };

        start_coroutine(coroutine)
    }

    fn update_normal(&mut self, _dt: f32) {
        let mut resources = storage::get_mut::<Resources>().unwrap();

        if is_key_pressed(KeyCode::Y) {
            self.health = 0;
        }
        self.pos = resources.collision_world.actor_pos(self.collider);

        if self.health <= 0 {
            self.state_machine.set_state(Self::ST_DEATH);
        }

        let on_ground = resources
            .collision_world
            .collide_check(self.collider, self.pos + vec2(0., 1.));

        if self.speed.x < 0.0 {
            self.facing = false;
        }
        if self.speed.x > 0.0 {
            self.facing = true;
        }

        if on_ground == false {
            self.speed.y += consts::GRAVITY * get_frame_time();
        }

        if is_key_down(KeyCode::Right) {
            self.speed.x = consts::RUN_SPEED;
        } else if is_key_down(KeyCode::Left) {
            self.speed.x = -consts::RUN_SPEED;
        } else {
            self.speed.x = 0.;
        }

        if is_key_pressed(KeyCode::Space) {
            if on_ground {
                self.speed.y = -consts::JUMP_SPEED;
            }
        }

        resources
            .collision_world
            .move_h(self.collider, self.speed.x * get_frame_time());
        if !resources
            .collision_world
            .move_v(self.collider, self.speed.y * get_frame_time())
        {
            self.speed.y = 0.0;
        }

        if is_key_pressed(KeyCode::LeftControl) {
            let mut bullets = scene::find_node_by_type::<crate::Bullets>().unwrap();
            let pos = resources.collision_world.actor_pos(self.collider);

            bullets.spawn_bullet(pos, self.facing);
        }
    }

    fn update_state_machine(&mut self) {
        StateMachine::update(self, |player| &mut player.state_machine);
    }
}

impl scene::Node for Player {
    fn draw(&mut self) {
        let resources = storage::get_mut::<Resources>().unwrap();

        draw_rectangle(
            self.pos.x as f32 - 4.0,
            self.pos.y as f32 - 5.0,
            16.0,
            2.0,
            RED,
        );
        draw_rectangle(
            self.pos.x as f32 - 4.0,
            self.pos.y as f32 - 5.0,
            self.health as f32 / 100.0 * 16.0,
            2.0,
            GREEN,
        );

        if self.facing {
            resources.tiled_map.spr(
                "tileset",
                consts::PLAYER_SPRITE,
                Rect::new(self.pos.x, self.pos.y, 8.0, 8.0),
            );
        } else {
            resources.tiled_map.spr(
                "tileset",
                consts::PLAYER_SPRITE,
                Rect::new(self.pos.x + 8.0, self.pos.y, -8.0, 8.0),
            );
        }
    }

    fn update(&mut self) {
        self.update_state_machine();
    }
}
