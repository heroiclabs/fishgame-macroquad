use macroquad::{
    color,
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        coroutines::{start_coroutine, wait_seconds, Coroutine},
        scene::{self, RefMut},
        state_machine::{State, StateMachine},
    },
    prelude::*,
    ui::{self, hash},
};
use physics_platformer::Actor;

use crate::{consts, Pickup, Resources};

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
    gun_fx_sprite: AnimatedSprite,
    gun_fx: bool,
    pub collider: Actor,
    pos: Vec2,
    speed: Vec2,
    on_ground: bool,
    dead: bool,
    facing: bool,
    weapon: Option<i32>,
    input: Input,
}

impl Fish {
    pub fn new(spawner_pos: Vec2) -> Fish {
        let mut resources = storage::get_mut::<Resources>().unwrap();

        let fish_sprite = AnimatedSprite::new(
            76,
            66,
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
                Animation {
                    name: "death".to_string(),
                    row: 12,
                    frames: 3,
                    fps: 5,
                },
                Animation {
                    name: "death2".to_string(),
                    row: 14,
                    frames: 4,
                    fps: 8,
                },
            ],
            true,
        );
        let gun_sprite = AnimatedSprite::new(
            92,
            32,
            &[
                Animation {
                    name: "idle".to_string(),
                    row: 0,
                    frames: 1,
                    fps: 1,
                },
                Animation {
                    name: "shoot".to_string(),
                    row: 1,
                    frames: 3,
                    fps: 15,
                },
            ],
            false,
        );
        let gun_fx_sprite = AnimatedSprite::new(
            92,
            32,
            &[Animation {
                name: "shoot".to_string(),
                row: 2,
                frames: 3,
                fps: 15,
            }],
            false,
        );
        Fish {
            fish_sprite,
            gun_fx_sprite,
            gun_fx: false,
            gun_sprite,
            collider: resources.collision_world.add_actor(spawner_pos, 30, 54),
            on_ground: false,
            dead: false,
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

    pub fn facing_dir(&self) -> f32 {
        if self.facing {
            1.
        } else {
            -1.
        }
    }

    pub fn draw(&mut self) {
        let resources = storage::get::<Resources>().unwrap();

        self.fish_sprite.update();

        draw_texture_ex(
            resources.whale,
            self.pos.x - 25.,
            self.pos.y - 10.,
            color::WHITE,
            DrawTextureParams {
                source: Some(self.fish_sprite.frame().source_rect),
                dest_size: Some(self.fish_sprite.frame().dest_size),
                flip_x: !self.facing,
                ..Default::default()
            },
        );

        if self.dead == false && self.weapon.is_some() {
            let gun_mount_pos = if self.facing {
                vec2(0., 16.)
            } else {
                vec2(-60., 16.)
            };
            self.gun_sprite.update();
            draw_texture_ex(
                resources.gun,
                self.pos.x + gun_mount_pos.x,
                self.pos.y + gun_mount_pos.y,
                color::WHITE,
                DrawTextureParams {
                    source: Some(self.gun_sprite.frame().source_rect),
                    dest_size: Some(self.gun_sprite.frame().dest_size),
                    flip_x: !self.facing,
                    ..Default::default()
                },
            );

            if self.gun_fx {
                self.gun_fx_sprite.update();
                draw_texture_ex(
                    resources.gun,
                    self.pos.x + gun_mount_pos.x,
                    self.pos.y + gun_mount_pos.y,
                    color::WHITE,
                    DrawTextureParams {
                        source: Some(self.gun_fx_sprite.frame().source_rect),
                        dest_size: Some(self.gun_fx_sprite.frame().dest_size),
                        flip_x: !self.facing,
                        ..Default::default()
                    },
                );
            }
        }
    }
}

pub struct Player {
    pub fish: Fish,

    deathmatch: bool,
    win: bool,
    pub want_quit: bool,
    jump_grace_timer: f32,
    state_machine: StateMachine<RefMut<Player>>,
}

impl Player {
    const ST_NORMAL: usize = 0;
    const ST_DEATH: usize = 1;
    const ST_SHOOT: usize = 2;
    const ST_AFTERMATCH: usize = 3;

    pub fn new(deathmatch: bool) -> Player {
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
            State::new()
                .update(Self::update_shoot)
                .coroutine(Self::shoot_coroutine),
        );
        state_machine.add_state(
            Self::ST_AFTERMATCH,
            State::new().update(Self::update_aftermatch),
        );

        Player {
            fish: Fish::new(spawner_pos),
            deathmatch,
            win: false,
            want_quit: false,
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
        self.fish.dead
    }

    pub fn kill(&mut self, direction: bool) {
        self.fish.facing = direction;
        self.state_machine.set_state(Self::ST_DEATH);
    }

    pub fn armed(&self) -> bool {
        self.fish.weapon.is_some()
    }

    fn death_coroutine(node: &mut RefMut<Player>) -> Coroutine {
        let handle = node.handle();
        let coroutine = async move {
            {
                let mut node = scene::get_node(handle).unwrap();
                node.fish.speed.x = -300. * node.fish.facing_dir();
                node.fish.speed.y = -150.;

                node.fish.dead = true;
                node.fish.fish_sprite.set_animation(2);
            }
            // give some take for a dead fish to take off the ground
            wait_seconds(0.1).await;

            // wait until it lands
            while scene::get_node(handle).unwrap().fish.on_ground == false {
                next_frame().await;
            }

            {
                let mut node = scene::get_node(handle).unwrap();
                node.fish.fish_sprite.set_animation(3);
                node.fish.speed = vec2(0., 0.);
            }

            wait_seconds(0.5).await;

            {
                let mut resources = storage::get_mut::<Resources>().unwrap();
                let mut node = scene::get_node(handle).unwrap();
                let pos = node.fish.pos;

                node.fish.fish_sprite.playing = false;
                node.fish.speed = vec2(0., 0.);
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
            this.fish.fish_sprite.playing = true;
            this.fish.disarm();

            // in deathmatch we can just get back to normal after death
            if this.deathmatch {
                this.state_machine.set_state(Self::ST_NORMAL);
                this.fish.dead = false;
                resources
                    .collision_world
                    .set_actor_position(this.fish.collider, this.fish.pos);
            }
        };

        start_coroutine(coroutine)
    }

    fn shoot_coroutine(node: &mut RefMut<Player>) -> Coroutine {
        let handle = node.handle();
        let coroutine = async move {
            {
                let mut node = &mut *scene::get_node(handle).unwrap();

                node.fish.gun_fx = true;
                let mut net_syncronizer =
                    scene::find_node_by_type::<crate::NetSyncronizer>().unwrap();
                net_syncronizer.shoot();

                let mut bullets = scene::find_node_by_type::<crate::Bullets>().unwrap();
                bullets.spawn_bullet(node.fish.pos, node.fish.facing);
                node.fish.speed.x = -consts::GUN_THROWBACK * node.fish.facing_dir();
            }
            {
                let node = &mut *scene::get_node(handle).unwrap();
                node.fish.gun_sprite.set_animation(1);
            }
            for i in 0u32..3 {
                {
                    let node = &mut *scene::get_node(handle).unwrap();
                    node.fish.gun_sprite.set_frame(i);
                    node.fish.gun_fx_sprite.set_frame(i);
                }

                wait_seconds(0.08).await;
            }
            {
                let mut node = scene::get_node(handle).unwrap();
                node.fish.gun_sprite.set_animation(0);
            }

            let mut node = &mut *scene::get_node(handle).unwrap();

            node.fish.gun_fx = false;
            let weapon = node.fish.weapon.as_mut().unwrap();
            *weapon -= 1;

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

    fn update_shoot(node: &mut RefMut<Player>, _dt: f32) {
        node.fish.speed.x *= 0.9;
    }

    fn update_aftermatch(node: &mut RefMut<Player>, _dt: f32) {
        let resources = storage::get::<crate::gui::GuiResources>().unwrap();

        node.fish.speed.x = 0.0;

        ui::root_ui().push_skin(&resources.login_skin);
        ui::root_ui().window(
            hash!(),
            Vec2::new(
                screen_width() / 2. - 500. / 2.,
                screen_height() / 2. - 200. / 2.,
            ),
            Vec2::new(500., 200.),
            |ui| {
                if node.win {
                    ui.label(vec2(190., 30.), "You win!");
                    //crate::nakama::add_leaderboard_win();
                } else {
                    ui.label(vec2(190., 30.), "You lost!");
                }
                if ui.button(vec2(130., 60.), "Return to lobby") {
                    node.want_quit = true;
                }
            },
        );
        ui::root_ui().pop_skin();
    }

    fn update_normal(node: &mut RefMut<Player>, _dt: f32) {
        // self destruct, for debugging only
        if is_key_pressed(KeyCode::Y) {
            node.kill(true);
        }
        if is_key_pressed(KeyCode::U) {
            node.kill(false);
        }

        let node = &mut **node;
        let fish = &mut node.fish;

        if fish.input.right {
            fish.fish_sprite.set_animation(1);
            fish.speed.x = consts::RUN_SPEED;
            fish.facing = true;
        } else if fish.input.left {
            fish.fish_sprite.set_animation(1);
            fish.speed.x = -consts::RUN_SPEED;
            fish.facing = false;
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

        if fish.input.fire {
            if fish.weapon.is_some() {
                node.state_machine.set_state(Self::ST_SHOOT);
            }
        }
    }

    fn draw_hud(&self) {
        if self.is_dead() {
            return;
        }
        if let Some(bullets) = self.fish.weapon {
            let full_color = Color::new(0.8, 0.9, 1.0, 1.0);
            let empty_color = Color::new(0.8, 0.9, 1.0, 0.8);
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
        let game_started = scene::find_node_by_type::<crate::NetSyncronizer>()
            .unwrap()
            .game_started;

        if game_started {
            node.fish.input.jump = is_key_pressed(KeyCode::Space)
                || is_key_pressed(KeyCode::W)
                || is_key_pressed(KeyCode::Up);
            node.fish.input.fire =
                is_key_pressed(KeyCode::LeftControl) || is_key_pressed(KeyCode::F);
            node.fish.input.left = is_key_down(KeyCode::Left) || is_key_down(KeyCode::A);
            node.fish.input.right = is_key_down(KeyCode::Right) || is_key_down(KeyCode::D);
        }

        // win condition
        if node.deathmatch == false && game_started {
            let others = scene::find_nodes_by_type::<crate::RemotePlayer>();
            let alive_enemies = others.filter(|player| player.dead == false).count();

            if node.fish.dead {
                node.win = false;
                node.state_machine.set_state(Self::ST_AFTERMATCH);
            }

            if alive_enemies == 0 {
                node.win = true;
                node.state_machine.set_state(Self::ST_AFTERMATCH);
            }
        }

        {
            let node = &mut *node;
            let fish = &mut node.fish;

            let mut resources = storage::get_mut::<Resources>().unwrap();
            fish.pos = resources.collision_world.actor_pos(fish.collider);

            fish.on_ground = resources
                .collision_world
                .collide_check(fish.collider, fish.pos + vec2(0., 1.));

            if fish.on_ground == false {
                fish.speed.y += consts::GRAVITY * get_frame_time();
            }

            if fish.on_ground {
                node.jump_grace_timer = consts::JUMP_GRACE_TIME;
            } else if node.jump_grace_timer > 0. {
                node.jump_grace_timer -= get_frame_time();
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
        }
        StateMachine::update_detached(&mut node, |node| &mut node.state_machine);

        for pickup in scene::find_nodes_by_type::<Pickup>() {
            let collide = |player: Vec2, pickup: Vec2| {
                (player + vec2(16., 32.)).distance(pickup + vec2(16., 16.)) < 90.
            };

            if collide(node.pos(), pickup.pos) {
                pickup.delete();
                node.pick_weapon();
            }
        }
    }
}
