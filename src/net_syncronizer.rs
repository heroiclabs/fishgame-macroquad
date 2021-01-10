use macroquad::{
    experimental::{collections::storage, scene},
    prelude::*,
};
use nanoserde::DeBin;
use std::collections::HashMap;

use crate::{consts, nakama, Player, Resources};

struct NetworkCache {
    sent_health: i32,
    sent_position: [u8; 3],
    last_send_time: f64,
}

impl NetworkCache {
    fn flush(&mut self) {
        self.sent_health = 100;
        self.sent_position = [0; 3];
        self.last_send_time = 0.0;
    }
}

pub struct Other {
    pub pos: Vec2,
    pub facing: bool,
    pub health: i32,
}

impl Other {
    fn new() -> Other {
        Other {
            pos: vec2(0., 0.),
            facing: true,
            health: 100,
        }
    }
}

bitfield::bitfield! {
    struct PlayerStateBits([u8]);
    impl Debug;
    u32;
    x, set_x: 9, 0;
    y, set_y: 19, 10;
    facing, set_facing: 20;
    shooting, set_shooting: 21;
}

#[test]
fn test_bitfield() {
    let mut bits = PlayerStateBits([0; 3]);

    bits.set_x(345);
    bits.set_y(567);
    bits.set_facing(true);
    bits.set_shooting(false);

    assert_eq!(bits.x(), 345);
    assert_eq!(bits.y(), 567);
    assert_eq!(bits.facing(), true);
    assert_eq!(bits.shooting(), false);
    assert_eq!(std::mem::size_of_val(&bits), 3);
}

mod message {
    use nanoserde::{DeBin, SerBin};

    #[derive(Debug, Clone, SerBin, DeBin, PartialEq)]
    pub struct Move(pub [u8; 3]);
    impl Move {
        pub const OPCODE: i32 = 1;
    }

    #[derive(Debug, Clone, SerBin, DeBin, PartialEq)]
    pub struct SelfDamage(pub u8);
    impl SelfDamage {
        pub const OPCODE: i32 = 2;
    }

    #[derive(Debug, Clone, SerBin, DeBin, PartialEq)]
    pub struct Died;
    impl Died {
        pub const OPCODE: i32 = 3;
    }
}

pub struct NetSyncronizer {
    network_cache: NetworkCache,
    others: HashMap<String, Other>,
}

impl NetSyncronizer {
    pub fn new() -> NetSyncronizer {
        NetSyncronizer {
            network_cache: NetworkCache {
                sent_position: [0; 3],
                last_send_time: 0.0,
                sent_health: 100,
            },
            others: HashMap::new(),
        }
    }

    pub fn others(&self) -> impl Iterator<Item = &Other> {
        self.others.values()
    }
}

impl scene::Node for NetSyncronizer {
    fn draw(&mut self) {
        let resources = storage::get_mut::<Resources>().unwrap();

        // draw other others
        for (
            other_id,
            Other {
                pos: Vec2 { x, y },
                facing,
                health,
                ..
            },
        ) in self.others.values().enumerate()
        {
            draw_text_ex(
                &format!("player {}", other_id),
                *x as f32 - 4.0,
                *y as f32 - 6.0,
                TextParams {
                    font_size: 30,
                    font_scale: 0.15,
                    ..Default::default()
                },
            );
            draw_rectangle(*x as f32 - 4.0, *y as f32 - 5.0, 16.0, 1.5, RED);
            draw_rectangle(
                *x as f32 - 4.0,
                *y as f32 - 5.0,
                *health as f32 / 100.0 * 16.0,
                1.5,
                GREEN,
            );

            if *facing {
                resources.tiled_map.spr(
                    "tileset",
                    consts::PLAYER_SPRITE,
                    Rect::new(*x as f32, *y as f32, 8.0, 8.0),
                );
            } else {
                resources.tiled_map.spr(
                    "tileset",
                    consts::PLAYER_SPRITE,
                    Rect::new(*x as f32 + 8.0, *y as f32, -8.0, 8.0),
                );
            }
        }
    }

    fn update(&mut self) {
        let mut resources = storage::get_mut::<Resources>().unwrap();

        {
            let shooting = is_key_pressed(KeyCode::LeftControl);
            let network_frame =
                get_time() - self.network_cache.last_send_time > (1. / consts::NETWORK_FPS) as f64;

            if shooting || network_frame {
                let player = scene::find_node_by_type::<Player>().unwrap();

                self.network_cache.last_send_time = get_time();

                let mut state = PlayerStateBits([0; 3]);

                state.set_x(player.pos().x as u32);
                state.set_y(player.pos().y as u32);
                state.set_facing(player.facing());
                state.set_shooting(shooting);

                if self.network_cache.sent_position != state.0 {
                    self.network_cache.sent_position = state.0;
                    nakama::send_bin(message::Move::OPCODE, &message::Move(state.0));
                }

                if self.network_cache.sent_health != player.health() {
                    if player.health() >= 0 {
                        nakama::send_bin(
                            message::SelfDamage::OPCODE,
                            &message::SelfDamage(player.health() as u8),
                        );
                    } else {
                        nakama::send_bin(message::Died::OPCODE, &message::Died);
                    }
                    self.network_cache.sent_health = player.health();
                }
            }
        }

        while let Some(event) = nakama::events() {
            match event {
                nakama::Event::Leave(leaver) => {
                    if let Some(leaver) = self.others.get(&leaver) {
                        resources.explosion_fxses.spawn(leaver.pos + vec2(4., 4.));
                    }
                    self.others.remove(&leaver);
                }
                nakama::Event::Join(joined) => {
                    self.network_cache.flush();
                    self.others.insert(joined, Other::new());
                }
            }
        }

        while let Some(msg) = nakama::try_recv() {
            if let Some(other) = self.others.get_mut(&msg.user_id) {
                match msg.opcode as i32 {
                    message::Move::OPCODE => {
                        let message::Move(data) = DeBin::deserialize_bin(&msg.data).unwrap();
                        let state = PlayerStateBits(data);

                        let facing = state.facing();
                        let shooting = state.shooting();
                        let pos = vec2(state.x() as f32, state.y() as f32);

                        other.pos = pos;
                        other.facing = facing;
                        if shooting {
                            let mut bullets = scene::find_node_by_type::<crate::Bullets>().unwrap();
                            bullets.spawn_bullet(pos, facing);
                        }
                    }
                    message::SelfDamage::OPCODE => {
                        let message::SelfDamage(health) =
                            DeBin::deserialize_bin(&msg.data).unwrap();

                        other.health = health as i32;
                    }
                    message::Died::OPCODE => {
                        resources.explosion_fxses.spawn(other.pos + vec2(4., 4.));
                    }
                    opcode => {
                        warn!("Unknown opcode: {}", opcode);
                    }
                }
            }
        }
    }
}
