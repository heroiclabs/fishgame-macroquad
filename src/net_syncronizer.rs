use macroquad::{
    experimental::{
        collections::storage,
        coroutines::{start_coroutine, wait_seconds},
        scene::{self, Handle, RefMut},
    },
    prelude::*,
};
use nanoserde::DeBin;
use std::collections::{BTreeMap, BTreeSet};

use crate::{consts, nakama, Pickup, Player, RemotePlayer, Resources};

struct NetworkCache {
    sent_death_state: bool,
    sent_position: [u8; 3],
    last_send_time: f64,
}

impl NetworkCache {
    fn flush(&mut self) {
        self.sent_death_state = false;
        self.sent_position = [0; 3];
        self.last_send_time = 0.0;
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
    weapon, set_weapon: 22;
    dead, set_dead: 23;
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

    #[derive(Debug, Clone, SerBin, DeBin, PartialEq)]
    pub struct SpawnItem {
        pub id: u32,
        pub x: u16,
        pub y: u16,
    }
    impl SpawnItem {
        pub const OPCODE: i32 = 4;
    }

    #[derive(Debug, Clone, SerBin, DeBin, PartialEq)]
    pub struct DeleteItem {
        pub id: u32,
    }
    impl DeleteItem {
        pub const OPCODE: i32 = 5;
    }

    #[derive(Debug, Clone, SerBin, DeBin, PartialEq)]
    pub struct Idle;
    impl Idle {
        pub const OPCODE: i32 = 7;
    }
}

pub struct NetSyncronizer {
    network_cache: NetworkCache,
    network_id: String,
    others: BTreeMap<String, Handle<RemotePlayer>>,
    pickups: BTreeMap<usize, Handle<Pickup>>,
    network_ids: BTreeSet<String>,
    shoot_pending: bool,
}

impl NetSyncronizer {
    pub fn new(network_id: String) -> NetSyncronizer {
        NetSyncronizer {
            network_cache: NetworkCache {
                sent_position: [0; 3],
                last_send_time: 0.0,
                sent_death_state: false,
            },
            others: BTreeMap::new(),
            network_ids: {
                let mut network_ids = BTreeSet::new();
                network_ids.insert(network_id.clone());
                network_ids
            },
            network_id,
            pickups: BTreeMap::new(),
            shoot_pending: false,
        }
    }

    pub fn is_host(&self) -> bool {
        // no other players connected
        if self.others.len() == 0 {
            return true;
        }

        self.network_id < *self.others.keys().nth(0).unwrap()
    }

    pub fn shoot(&mut self) {
        self.shoot_pending = true;
    }
    pub fn spawn_item(&mut self, id: usize, pos: Vec2) {
        nakama::send_bin(
            message::SpawnItem::OPCODE,
            &message::SpawnItem {
                id: id as _,
                x: pos.x as _,
                y: pos.y as _,
            },
        );
    }

    pub fn delete_item(&mut self, id: usize) {
        nakama::send_bin(
            message::DeleteItem::OPCODE,
            &message::DeleteItem { id: id as _ },
        );
    }

}

impl scene::Node for NetSyncronizer {
    fn ready(_: RefMut<Self>) {
        let idle = async move {
            loop {
                nakama::send_bin(message::Idle::OPCODE, &message::Idle);
                wait_seconds(1.0).await;
            }
        };
        start_coroutine(idle);
    }
    fn draw(node: RefMut<Self>) {
        if node.is_host() {
            draw_text_ex(
                "You are the host",
                0.0,
                3.0,
                TextParams {
                    font_size: 20,
                    font_scale: 0.25,
                    ..Default::default()
                },
            );
        }
    }

    fn update(mut node: RefMut<Self>) {
        {
            let shooting = node.shoot_pending;
            node.shoot_pending = false;
            let network_frame =
                get_time() - node.network_cache.last_send_time > (1. / consts::NETWORK_FPS) as f64;

            if shooting || network_frame {
                let player = scene::find_node_by_type::<Player>().unwrap();

                node.network_cache.last_send_time = get_time();

                let mut state = PlayerStateBits([0; 3]);

                state.set_x(player.pos().x as u32);
                state.set_y(player.pos().y as u32);
                state.set_facing(player.facing());
                state.set_shooting(shooting);
                state.set_weapon(player.armed());
                state.set_dead(player.is_dead());

                if node.network_cache.sent_position != state.0 {
                    node.network_cache.sent_position = state.0;
                    nakama::send_bin(message::Move::OPCODE, &message::Move(state.0));
                }

                if node.network_cache.sent_death_state != player.is_dead() {
                    if player.is_dead() {
                        nakama::send_bin(message::Died::OPCODE, &message::Died);
                    }
                    node.network_cache.sent_death_state = player.is_dead();
                }
            }
        }

        while let Some(event) = nakama::events() {
            match event {
                nakama::Event::Leave(leaver) => {
                    if let Some(leaver) = node.others.remove(&leaver) {
                        let mut resources = storage::get_mut::<Resources>().unwrap();

                        let leaver = scene::get_node::<RemotePlayer>(leaver).unwrap();
                        resources
                            .explosion_fxses
                            .spawn(leaver.pos() + vec2(15., 33.));

                        leaver.delete();
                    }
                    node.network_ids.remove(&leaver);
                }
                nakama::Event::Join(joined) => {
                    node.network_cache.flush();
                    node.network_ids.insert(joined.clone());
                    if node.others.contains_key(&joined) == false {
                        node.others
                            .insert(joined.clone(), scene::add_node(RemotePlayer::new(joined)));
                    }
                }
            }
        }

        if is_key_pressed(KeyCode::U) {
            for id in &node.network_ids {
                warn!("id: {}", id);
            }
            for player in scene::find_nodes_by_type::<RemotePlayer>() {
                warn!("players: {} {:?}", &player.network_id, player.pos());
            }
        }

        while let Some(msg) = nakama::try_recv() {
            if let Some(other) = node.others.get(&msg.user_id) {
                let mut other = scene::get_node(*other).unwrap();

                match msg.opcode as i32 {
                    message::Move::OPCODE => {
                        let message::Move(data) = DeBin::deserialize_bin(&msg.data).unwrap();
                        let state = PlayerStateBits(data);
                        let pos = vec2(state.x() as f32, state.y() as f32);

                        other.set_pos(pos);
                        other.set_facing(state.facing());
                        other.set_dead(state.dead());

                        if other.armed() && state.weapon() == false {
                            let mut resources = storage::get_mut::<Resources>().unwrap();
                            resources.disarm_fxses.spawn(pos + vec2(16., 33.));
                            other.disarm();
                        }
                        if other.armed() == false && state.weapon() {
                            other.pick_weapon();
                        }
                        if state.shooting() {
                            let mut bullets = scene::find_node_by_type::<crate::Bullets>().unwrap();
                            bullets.spawn_bullet(pos, state.facing());
                        }
                    }
                    message::SelfDamage::OPCODE => {
                        let message::SelfDamage(_health) =
                            DeBin::deserialize_bin(&msg.data).unwrap();
                    }
                    message::Died::OPCODE => {
                        let mut resources = storage::get_mut::<Resources>().unwrap();

                        resources
                            .explosion_fxses
                            .spawn(other.pos() + vec2(15., 33.));
                    }
                    message::SpawnItem::OPCODE => {
                        let message::SpawnItem { id, x, y } =
                            DeBin::deserialize_bin(&msg.data).unwrap();
                        let pos = vec2(x as f32, y as f32);

                        let new_node = scene::add_node(Pickup::new(pos));
                        if let Some(pickup) = node.pickups.insert(id as _, new_node) {
                            if let Some(node) = scene::get_node(pickup) {
                                node.delete();
                            }
                        }
                    }
                    message::DeleteItem::OPCODE => {
                        let message::DeleteItem { id } = DeBin::deserialize_bin(&msg.data).unwrap();

                        if let Some(pickup) = node.pickups.remove(&(id as usize)) {
                            if let Some(node) = scene::get_node(pickup) {
                                node.delete();
                            }
                        }
                    }
                    message::Idle::OPCODE => {}
                    opcode => {
                        warn!("Unknown opcode: {}", opcode);
                    }
                }
            }
        }
    }
}
