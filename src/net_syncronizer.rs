use macroquad::{
    experimental::{
        collections::storage,
        coroutines::{start_coroutine, wait_seconds},
        scene::{self, Handle, RefMut},
    },
    prelude::*,
    ui::{self, hash, root_ui, widgets},
};
use nanoserde::DeBin;
use std::collections::{BTreeMap, BTreeSet};

use crate::{
    consts,
    nakama::{self, ApiClient},
    GameType, Pickup, Player, RemotePlayer, Resources,
};

struct NetworkCache {
    sent_position: [u8; 3],
    last_send_time: f64,
}

impl NetworkCache {
    fn flush(&mut self) {
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
    pub struct Ready;
    impl Ready {
        pub const OPCODE: i32 = 6;
    }

    #[derive(Debug, Clone, SerBin, DeBin, PartialEq)]
    pub struct Idle;
    impl Idle {
        pub const OPCODE: i32 = 7;
    }

    #[derive(Debug, Clone, SerBin, DeBin, PartialEq)]
    pub struct StartGame;
    impl StartGame {
        pub const OPCODE: i32 = 8;
    }
}

pub struct NetSyncronizer {
    network_cache: NetworkCache,
    network_id: String,
    others: BTreeMap<String, Handle<RemotePlayer>>,
    pickups: BTreeMap<usize, Handle<Pickup>>,
    network_ids: BTreeSet<String>,
    shoot_pending: bool,
    ready: bool,
    pub game_type: GameType,
    pub game_started: bool,
}

impl NetSyncronizer {
    pub(crate) fn new(network_id: String, game_type: GameType) -> NetSyncronizer {
        NetSyncronizer {
            game_type,
            network_cache: NetworkCache {
                sent_position: [0; 3],
                last_send_time: 0.0,
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
            ready: false,
            game_started: game_type == GameType::Deathmatch,
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
        let mut nakama = storage::get_mut::<nakama::ApiClient>().unwrap();

        nakama.socket_send(
            message::SpawnItem::OPCODE,
            &message::SpawnItem {
                id: id as _,
                x: pos.x as _,
                y: pos.y as _,
            },
        );
    }

    pub fn delete_item(&mut self, id: usize) {
        let mut nakama = storage::get_mut::<nakama::ApiClient>().unwrap();

        nakama.socket_send(
            message::DeleteItem::OPCODE,
            &message::DeleteItem { id: id as _ },
        );
    }
}

impl scene::Node for NetSyncronizer {
    fn ready(_: RefMut<Self>) {
        let idle = async move {
            loop {
                {
                    let mut nakama = storage::get_mut::<ApiClient>().unwrap();

                    nakama.socket_send(message::Idle::OPCODE, &message::Idle);
                }
                wait_seconds(1.0).await;
            }
        };
        start_coroutine(idle);
    }

    fn draw(mut node: RefMut<Self>) {
        if node.is_host() {
            root_ui().label(None, "You are the host");
        }

        if node.game_type != GameType::Deathmatch && node.game_started == false {
            let resources = storage::get::<crate::gui::GuiResources>().unwrap();
            let mut nakama = storage::get_mut::<ApiClient>().unwrap();

            ui::root_ui().push_skin(&resources.login_skin);
            ui::root_ui().window(
                hash!(),
                Vec2::new(
                    screen_width() / 2. - 500. / 2.,
                    screen_height() / 2. - 200. / 2.,
                ),
                Vec2::new(500., 200.),
                |ui| {
                    if let GameType::LastFishStanding { private: true } = node.game_type {
                        let mut match_id = nakama.match_id().unwrap_or("".to_string());

                        widgets::InputText::new(hash!())
                            .ratio(3. / 4.)
                            .label("Match ID")
                            .ui(ui, &mut match_id);
                    }
                    for player in node.others.values() {
                        let player = scene::get_node(*player).unwrap();
                        ui.label(None, &format!("{}: ", player.username));
                        ui.same_line(300.0);
                        if player.ready {
                            ui.label(None, "Ready");
                        } else {
                            ui.label(None, "Not ready");
                        }
                    }

                    let everyone_ready = {
                        let others = scene::find_nodes_by_type::<RemotePlayer>();
                        let (ready, notready) = others.fold((0, 0), |(ready, notready), player| {
                            if player.ready {
                                (ready + 1, notready)
                            } else {
                                (ready, notready + 1)
                            }
                        });

                        ready != 0 && notready == 0
                    } && node.ready;

                    if node.ready == false && ui.button(vec2(180.0, 100.0), "Ready") {
                        node.ready = true;
                        nakama.socket_send(message::Ready::OPCODE, &message::Ready);
                    }

                    if node.is_host() && everyone_ready {
                        if ui.button(vec2(150.0, 100.0), "Start match!") {
                            node.game_started = true;
                            nakama.socket_send(message::StartGame::OPCODE, &message::StartGame);
                        }
                    } else if node.ready {
                        ui.label(
                            vec2(20.0, 110.0),
                            "You are ready, waiting for other players!",
                        );
                    }
                },
            );
            ui::root_ui().pop_skin();
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
                    let mut nakama = storage::get_mut::<ApiClient>().unwrap();

                    nakama.socket_send(message::Move::OPCODE, &message::Move(state.0));
                }
            }
        }

        while let Some(event) = {
            let mut nakama = storage::get_mut::<ApiClient>().unwrap();

            nakama.try_recv()
        } {
            match event {
                nakama::Event::Presence { joins, leaves } => {
                    for leaver in leaves {
                        let leaver = leaver.session_id;
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

                    let self_session_id = node.network_id.clone();
                    for join in joins
                        .into_iter()
                        .filter(|join| &join.session_id != &self_session_id)
                    {
                        let joined = join.session_id;
                        let username = join.username;

                        node.network_cache.flush();
                        node.network_ids.insert(joined.clone());
                        if node.others.contains_key(&joined) == false {
                            node.others
                                .insert(joined, scene::add_node(RemotePlayer::new(username)));
                        }
                    }
                }
                nakama::Event::MatchData {
                    user_id,
                    opcode,
                    data,
                } => {
                    if let Some(other) = node.others.get(&user_id) {
                        let mut other = scene::get_node(*other).unwrap();

                        match opcode as i32 {
                            message::Move::OPCODE => {
                                let message::Move(data) = DeBin::deserialize_bin(&data).unwrap();
                                let state = PlayerStateBits(data);
                                let pos = vec2(state.x() as f32, state.y() as f32);

                                other.set_pos(pos);
                                other.set_facing(state.facing());

                                if state.dead() && other.dead != state.dead() {
                                    warn!("state.dead() && other.dead != state.dead()");
                                    let mut resources = storage::get_mut::<Resources>().unwrap();
                                    resources
                                        .explosion_fxses
                                        .spawn(other.pos() + vec2(15., 33.));
                                }
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
                                    let mut bullets =
                                        scene::find_node_by_type::<crate::Bullets>().unwrap();
                                    bullets.spawn_bullet(pos, state.facing());
                                }
                            }
                            message::Ready::OPCODE => {
                                other.ready = true;
                            }
                            message::StartGame::OPCODE => {
                                node.game_started = true;
                            }
                            message::SelfDamage::OPCODE => {
                                let message::SelfDamage(_health) =
                                    DeBin::deserialize_bin(&data).unwrap();
                            }
                            message::SpawnItem::OPCODE => {
                                let message::SpawnItem { id, x, y } =
                                    DeBin::deserialize_bin(&data).unwrap();
                                let pos = vec2(x as f32, y as f32);

                                let new_node = scene::add_node(Pickup::new(pos));
                                if let Some(pickup) = node.pickups.insert(id as _, new_node) {
                                    if let Some(node) = scene::get_node(pickup) {
                                        node.delete();
                                    }
                                }
                            }
                            message::DeleteItem::OPCODE => {
                                let message::DeleteItem { id } =
                                    DeBin::deserialize_bin(&data).unwrap();

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

        if is_key_pressed(KeyCode::U) {
            for id in &node.network_ids {
                warn!("id: {}", id);
            }
            for player in scene::find_nodes_by_type::<RemotePlayer>() {
                warn!("players: {} {:?}", &player.username, player.pos());
            }
        }
    }
}
