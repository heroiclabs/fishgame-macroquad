use macroquad::{
    experimental::{
        collections::storage,
        coroutines::{start_coroutine, wait_seconds},
        scene::{self, Handle, Node, RefMut},
    },
    prelude::*,
    ui::{self, hash, root_ui, widgets},
};
use nanoserde::DeBin;
use std::collections::{BTreeMap, BTreeSet};

use nakama_rs::api_client::Event;

use crate::{
    consts,
    nodes::{Nakama, Pickup, Player, RemotePlayer},
    GameType, Resources,
};
use plugin_api::ItemType;

struct NetworkCache {
    sent_position: [u8; 11],
    last_send_time: f64,
}

impl NetworkCache {
    fn flush(&mut self) {
        self.sent_position = [0; 11];
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
    u64, weapon, set_weapon: 85, 22;
    dead, set_dead: 86;
}

#[test]
fn test_bitfield() {
    let mut bits = PlayerStateBits([0; 11]);

    bits.set_x(345);
    bits.set_y(567);
    bits.set_facing(true);
    bits.set_shooting(false);
    bits.set_weapon(11527428624421318257);

    assert_eq!(bits.x(), 345);
    assert_eq!(bits.y(), 567);
    assert_eq!(bits.facing(), true);
    assert_eq!(bits.shooting(), false);
    assert_eq!(bits.weapon(), 11527428624421318257);
    assert_eq!(bits.dead(), false);
    assert_eq!(std::mem::size_of_val(&bits), 11);
}

mod message {
    use nanoserde::{DeBin, SerBin};

    #[derive(Debug, Clone, SerBin, DeBin, PartialEq)]
    pub struct State(pub [u8; 11]);
    impl State {
        pub const OPCODE: i32 = 1;
    }

    #[derive(Debug, Clone, SerBin, DeBin, PartialEq)]
    pub struct Damage {
        pub target: String,
        pub direction: bool,
    }
    impl Damage {
        pub const OPCODE: i32 = 2;
    }

    #[derive(Debug, Clone, SerBin, DeBin, PartialEq)]
    pub struct SpawnItem {
        pub id: u32,
        pub x: u16,
        pub y: u16,
        pub item_type: u64,
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

/// Node with per-session data, responsible for syncronisation
/// game state through nakama's socket connection
pub struct NakamaRealtimeGame {
    pub game_type: GameType,
    pub game_started: bool,

    network_id: String,
    network_cache: NetworkCache,
    remote_players: BTreeMap<String, Handle<RemotePlayer>>,
    pickups: BTreeMap<usize, Handle<Pickup>>,
    network_ids: BTreeSet<String>,
    shoot_pending: bool,
    ready: bool,
    nakama: Handle<Nakama>,
}

impl NakamaRealtimeGame {
    pub fn new(
        nakama: Handle<Nakama>,
        game_type: GameType,
        network_id: String,
    ) -> NakamaRealtimeGame {
        NakamaRealtimeGame {
            game_type,
            network_cache: NetworkCache {
                sent_position: [0; 11],
                last_send_time: 0.0,
            },
            remote_players: BTreeMap::new(),
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
            nakama,
        }
    }

    pub fn game_started(&self) -> bool {
        self.game_started
    }

    pub fn shoot(&mut self) {
        self.shoot_pending = true;
    }

    pub fn spawn_item(&mut self, id: usize, pos: Vec2, item_type: ItemType) {
        let mut nakama = scene::get_node(self.nakama);
        nakama.api_client.socket_send(
            message::SpawnItem::OPCODE,
            &message::SpawnItem {
                id: id as _,
                x: pos.x as _,
                y: pos.y as _,
                item_type: item_type.into(),
            },
        );
    }

    pub fn delete_item(&mut self, id: usize) {
        let mut nakama = scene::get_node(self.nakama);
        nakama.api_client.socket_send(
            message::DeleteItem::OPCODE,
            &message::DeleteItem { id: id as _ },
        );
    }

    pub fn kill(&mut self, target: &str, direction: bool) {
        let mut nakama = scene::get_node(self.nakama);
        nakama.api_client.socket_send(
            message::Damage::OPCODE,
            &message::Damage {
                target: target.to_string(),
                direction,
            },
        );
    }

    pub fn is_host(&self) -> bool {
        // no other players connected
        if self.remote_players.len() == 0 {
            return true;
        }

        self.network_id < *self.remote_players.keys().nth(0).unwrap()
    }
}

impl Node for NakamaRealtimeGame {
    fn ready(node: RefMut<Self>) {
        let nakama = node.nakama;
        let idle = async move {
            loop {
                {
                    let mut nakama = scene::get_node(nakama);

                    nakama
                        .api_client
                        .socket_send(message::Idle::OPCODE, &message::Idle);
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
            let resources = storage::get::<crate::gui::GuiResources>();
            let nakama = &mut scene::get_node(node.nakama).api_client;

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
                    for player in node.remote_players.values() {
                        let player = scene::get_node(*player);
                        ui.label(None, &format!("{}: ", player.username));
                        ui.same_line(300.0);
                        if player.ready {
                            ui.label(None, "Ready");
                        } else {
                            ui.label(None, "Not ready");
                        }
                    }

                    let everyone_ready = {
                        let remote_players = scene::find_nodes_by_type::<RemotePlayer>();
                        let (ready, notready) =
                            remote_players.fold((0, 0), |(ready, notready), player| {
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
        let api_client = &mut scene::get_node(node.nakama).api_client;

        {
            let shooting = node.shoot_pending;
            node.shoot_pending = false;
            let network_frame =
                get_time() - node.network_cache.last_send_time > (1. / consts::NETWORK_FPS) as f64;

            if shooting || network_frame {
                let player = scene::find_node_by_type::<Player>().unwrap();

                node.network_cache.last_send_time = get_time();

                let mut state = PlayerStateBits([0; 11]);

                state.set_x(player.pos().x as u32);
                state.set_y(player.pos().y as u32);
                state.set_facing(player.facing());
                state.set_shooting(shooting);
                state.set_weapon(player.weapon().map_or(0, |weapon| weapon.into()));
                state.set_dead(player.is_dead());

                if node.network_cache.sent_position != state.0 {
                    node.network_cache.sent_position = state.0;

                    api_client.socket_send(message::State::OPCODE, &message::State(state.0));
                }
            }
        }

        while let Some(event) = api_client.try_recv() {
            match event {
                Event::Presence { joins, leaves } => {
                    for leaver in leaves {
                        let leaver = leaver.session_id;
                        if let Some(leaver) = node.remote_players.remove(&leaver) {
                            let mut resources = storage::get_mut::<Resources>();

                            let leaver = scene::get_node::<RemotePlayer>(leaver);
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
                        if node.remote_players.contains_key(&joined) == false {
                            node.remote_players.insert(
                                joined.clone(),
                                scene::add_node(RemotePlayer::new(&username, &joined)),
                            );
                        }
                    }
                }
                Event::MatchData {
                    user_id,
                    opcode,
                    data,
                } => {
                    if let Some(other) = node.remote_players.get(&user_id) {
                        let mut other = scene::get_node(*other);

                        match opcode as i32 {
                            message::State::OPCODE => {
                                let message::State(data) = DeBin::deserialize_bin(&data).unwrap();
                                let state = PlayerStateBits(data);
                                let pos = vec2(state.x() as f32, state.y() as f32);

                                other.set_pos(pos);
                                other.set_facing(state.facing());

                                if state.dead() && other.dead != state.dead() {
                                    let mut resources = storage::get_mut::<Resources>();
                                    resources
                                        .explosion_fxses
                                        .spawn(other.pos() + vec2(15., 33.));
                                }
                                other.set_dead(state.dead());

                                if other.weapon().is_some() && state.weapon() == 0 {
                                    let mut resources = storage::get_mut::<Resources>();
                                    resources.disarm_fxses.spawn(pos + vec2(16., 33.));
                                    other.disarm();
                                }
                                if other.weapon().map_or(0, |weapon| weapon.into())
                                    != state.weapon()
                                {
                                    other.pick_weapon(state.weapon().into());
                                }
                                if state.shooting() {
                                    let handle = other.handle();
                                    other.shoot(handle);
                                }
                            }
                            message::Ready::OPCODE => {
                                other.ready = true;
                            }
                            message::StartGame::OPCODE => {
                                node.game_started = true;
                            }
                            message::Damage::OPCODE => {
                                let message::Damage { target, direction } =
                                    DeBin::deserialize_bin(&data).unwrap();
                                if target == node.network_id {
                                    let mut player = scene::find_node_by_type::<Player>().unwrap();
                                    player.kill(direction);
                                }
                            }
                            message::SpawnItem::OPCODE => {
                                let message::SpawnItem {
                                    id,
                                    x,
                                    y,
                                    item_type,
                                } = DeBin::deserialize_bin(&data).unwrap();
                                let pos = vec2(x as f32, y as f32);

                                let new_node = scene::add_node(Pickup::new(pos, item_type.into()));
                                if let Some(pickup) = node.pickups.insert(id as _, new_node) {
                                    if let Some(node) = scene::try_get_node(pickup) {
                                        node.delete();
                                    }
                                }
                            }
                            message::DeleteItem::OPCODE => {
                                let message::DeleteItem { id } =
                                    DeBin::deserialize_bin(&data).unwrap();

                                if let Some(pickup) = node.pickups.remove(&(id as usize)) {
                                    if let Some(node) = scene::try_get_node(pickup) {
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
