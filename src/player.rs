use macroquad::{
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        coroutines::{start_coroutine, wait_seconds, Coroutine},
        scene::{self, RefMut},
        state_machine::{State, StateMachine},
    },
    prelude::*,
};
use physics_platformer::Actor;

use crate::{consts, Resources};

#[derive(Default, Debug, Clone)]
pub struct Input {
    jump: bool,
    fire: bool,
    left: bool,
    right: bool,
}

pub struct Fish {
    fish_sprite: AnimatedSprite,
    gun_sprite: AnimatedSprite,
    collider: Actor,
    pos: Vec2,
    speed: Vec2,
    facing: bool,
    weapon: Option<i32>,
    input: Input,
}

impl Fish {
    pub fn new(spawner_pos: Vec2) -> Fish {
        let mut resources = storage::get_mut::<Resources>().unwrap();

        let fish_sprite = AnimatedSprite::new(
            (resources.whale, 76, 66),
            &[
                Animation {
                    name: "idle".to_string(),
                    row: 0,
                    frames: 7,
                    fps: 12,
                },
                Animation {
                    name: "run".to_string(),
                    row: 2,
                    frames: 6,
                    fps: 10,
                },
            ],
        );
        let gun_sprite = AnimatedSprite::new(
            (resources.gun, 92, 32),
            &[
                Animation {
                    name: "idle".to_string(),
                    row: 0,
                    frames: 1,
                    fps: 1,
                },
                Animation {
                    name: "shoot".to_string(),
                    row: 2,
                    frames: 3,
                    fps: 15,
                },
            ],
        );
        Fish {
            fish_sprite,
            gun_sprite,
            collider: resources.collision_world.add_actor(spawner_pos, 30, 54),
            pos: spawner_pos,
            speed: vec2(0., 0.),
            facing: true,
            weapon: None,
            input: Default::default(),
        }
    }

    pub fn pos(&self) -> Vec2 {
        self.pos
    }

    pub fn set_pos(&mut self, pos: Vec2) {
        self.pos = pos;
    }

    pub fn set_animation(&mut self, animation: usize) {
        self.fish_sprite.set_animation(animation);
    }

    pub fn set_facing(&mut self, facing: bool) {
        self.facing = facing;
    }

    pub fn disarm(&mut self) {
        self.weapon = None;
    }

    pub fn pick_weapon(&mut self) {
        self.weapon = Some(3);
    }

    pub fn armed(&self) -> bool {
        self.weapon.is_some()
    }

    pub fn draw(&mut self) {
        self.fish_sprite
            .draw(self.pos - vec2(25., 10.), self.facing, false);

        if self.weapon.is_some() {
            let gun_mount_pos = if self.facing {
                vec2(0., 18.)
            } else {
                vec2(-60., 18.)
            };
            self.gun_sprite
                .draw(self.pos + gun_mount_pos, self.facing, false);
        }
    }
}

pub struct Player {
    pub fish: Fish,

    jump_grace_timer: f32,
    state_machine: StateMachine<RefMut<Player>>,
}

impl Player {
    const ST_NORMAL: usize = 0;
    const ST_DEATH: usize = 1;
    const ST_SHOOT: usize = 1;

    pub fn new() -> Player {
        let spawner_pos = {
            let resources = storage::get_mut::<Resources>().unwrap();
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
        state_machine.add_state(
            Self::ST_SHOOT,
            State::new().coroutine(Self::shoot_coroutine),
        );

        Player {
            fish: Fish::new(spawner_pos),
            jump_grace_timer: 0.,
            state_machine,
        }
    }

    pub fn pos(&self) -> Vec2 {
        self.fish.pos
    }

    pub fn facing(&self) -> bool {
        self.fish.facing
    }

    pub fn pick_weapon(&mut self) {
        self.fish.weapon = Some(3);
    }

    pub fn is_dead(&self) -> bool {
        self.state_machine.state() == Self::ST_DEATH
    }

    pub fn kill(&mut self) {
        self.state_machine.set_state(Self::ST_DEATH);
    }

    pub fn armed(&self) -> bool {
        self.fish.weapon.is_some()
    }

    fn death_coroutine(node: &mut RefMut<Player>) -> Coroutine {
        let pos = node.fish.pos;

        let handle = node.handle();
        let coroutine = async move {
            {
                let mut resources = storage::get_mut::<Resources>().unwrap();

                resources.explosion_fxses.spawn(pos + vec2(15., 33.));
            }

            wait_seconds(0.5).await;

            let mut resources = storage::get_mut::<Resources>().unwrap();
            let mut this = scene::get_node(handle).unwrap();

            this.fish.pos = {
                let objects = &resources.tiled_map.layers["logic"].objects;
                let macroquad_tiled::Object::Rect {
                    world_x, world_y, ..
                } = objects[rand::gen_range(0, objects.len()) as usize];

                vec2(world_x, world_y)
            };
            resources
                .collision_world
                .set_actor_position(this.fish.collider, this.fish.pos);
            this.state_machine.set_state(Self::ST_NORMAL);
        };

        start_coroutine(coroutine)
    }

    fn shoot_coroutine(node: &mut RefMut<Player>) -> Coroutine {
        let handle = node.handle();
        let coroutine = async move {
            let mut node = &mut *scene::get_node(handle).unwrap();
            let weapon = node.fish.weapon.as_mut().unwrap();
            if *weapon > 0 {
                let mut net_syncronizer =
                    scene::find_node_by_type::<crate::NetSyncronizer>().unwrap();
                net_syncronizer.shoot();

                let mut bullets = scene::find_node_by_type::<crate::Bullets>().unwrap();
                bullets.spawn_bullet(node.fish.pos, node.fish.facing);

                *weapon -= 1;
            }

            if *weapon <= 0 {
                let mut resources = storage::get_mut::<Resources>().unwrap();
                resources.disarm_fxses.spawn(node.fish.pos + vec2(16., 33.));
                node.fish.weapon = None;
            }

            // node.weapon_animation.play(0, 0..5).await;
            // node.weapon_animation.play(0, 5..).await;
            node.state_machine.set_state(Self::ST_NORMAL);
        };

        start_coroutine(coroutine)
    }

    fn update_normal(node: &mut RefMut<Player>, dt: f32) {
        let mut resources = storage::get_mut::<Resources>().unwrap();

        // self destruct, for debugging only
        if is_key_pressed(KeyCode::Y) {
            node.kill();
        }

        let node = &mut **node;
        let fish = &mut node.fish;

        fish.pos = resources.collision_world.actor_pos(fish.collider);

        let on_ground = resources
            .collision_world
            .collide_check(fish.collider, fish.pos + vec2(0., 1.));

        if fish.speed.x < 0.0 {
            fish.facing = false;
        }
        if fish.speed.x > 0.0 {
            fish.facing = true;
        }

        if on_ground {
            node.jump_grace_timer = consts::JUMP_GRACE_TIME;
        } else if node.jump_grace_timer > 0. {
            node.jump_grace_timer -= dt;
        }

        if on_ground == false {
            fish.speed.y += consts::GRAVITY * get_frame_time();
        }

        if fish.input.right {
            fish.fish_sprite.set_animation(1);
            fish.speed.x = consts::RUN_SPEED;
        } else if fish.input.left {
            fish.fish_sprite.set_animation(1);
            fish.speed.x = -consts::RUN_SPEED;
        } else {
            fish.fish_sprite.set_animation(0);
            fish.speed.x = 0.;
        }

        if fish.input.jump {
            if node.jump_grace_timer > 0. {
                node.jump_grace_timer = 0.0;
                fish.speed.y = -consts::JUMP_SPEED;
            }
        }

        resources
            .collision_world
            .move_h(fish.collider, fish.speed.x * get_frame_time());
        if !resources
            .collision_world
            .move_v(fish.collider, fish.speed.y * get_frame_time())
        {
            fish.speed.y = 0.0;
        }

        if fish.input.fire {
            if fish.weapon.is_some() {
                node.state_machine.set_state(Self::ST_SHOOT);
            }
        }
    }

    fn draw_hud(&self) {
        if let Some(bullets) = self.fish.weapon {
            let full_color = Color::new(0.6, 0.6, 0.9, 1.0);
            let empty_color = Color::new(0.6, 0.6, 0.9, 0.8);
            for i in 0..3 {
                let x = self.fish.pos.x + 15.0 * i as f32;

                if i >= bullets {
                    draw_circle_lines(x, self.fish.pos.y - 4.0, 4.0, 2., empty_color);
                } else {
                    draw_circle(x, self.fish.pos.y - 4.0, 4.0, full_color);
                };
            }
        }
    }
}

impl scene::Node for Player {
    fn draw(mut node: RefMut<Self>) {
        node.fish.draw();

        node.draw_hud();
    }

    fn update(mut node: RefMut<Self>) {
        node.fish.input.jump = is_key_pressed(KeyCode::Space)
            || is_key_pressed(KeyCode::W)
            || is_key_pressed(KeyCode::Up);
        node.fish.input.fire = is_key_pressed(KeyCode::LeftControl) || is_key_pressed(KeyCode::F);
        node.fish.input.left = is_key_down(KeyCode::Left) || is_key_down(KeyCode::A);
        node.fish.input.right = is_key_down(KeyCode::Right) || is_key_down(KeyCode::D);

        StateMachine::update_detached(&mut node, |node| &mut node.state_machine)
    }
}
