use macroquad::{
    experimental::{
        collections::storage,
        scene::{self, Handle},
    },
    math::{vec2, Vec2},
    ui::{hash, root_ui, widgets},
    window::{next_frame, screen_height, screen_width},
};

use nakama_rs::matchmaker::{Matchmaker, QueryItemBuilder};

use crate::nodes::Nakama;

use super::{GuiResources, Scene, WINDOW_HEIGHT, WINDOW_WIDTH};

pub async fn matchmaking_lobby(nakama: Handle<Nakama>) -> Scene {
    let username: String = scene::get_node(nakama)
        .unwrap()
        .api_client
        .username()
        .unwrap();

    let mut minimum_players = "2".to_string();
    let mut maximum_players = "4".to_string();

    let mut match_id = String::new();

    let resources = storage::get::<GuiResources>().unwrap();

    let mut leaderboard_loaded = false;

    loop {
        root_ui().push_skin(&resources.login_skin);

        let mut nakama = scene::get_node(nakama).unwrap();

        let mut next_scene = None;

        root_ui().window(
            hash!(),
            Vec2::new(
                screen_width() / 2. - WINDOW_WIDTH / 2.,
                screen_height() / 2. - WINDOW_HEIGHT / 2.,
            ),
            Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            |ui| {
                ui.label(None, &format!("Welcome, {}", username));

                match ui.tabbar(
                    hash!(),
                    vec2(WINDOW_WIDTH - 50., 50.),
                    &["Matchmaking", "Private match", "Leaderboards"],
                ) {
                    0 => {
                        widgets::InputText::new(hash!())
                            .ratio(1. / 4.)
                            .filter_numbers()
                            .label("Minimum players")
                            .ui(ui, &mut minimum_players);

                        widgets::InputText::new(hash!())
                            .ratio(1. / 4.)
                            .filter_numbers()
                            .label("Maximum players")
                            .ui(ui, &mut maximum_players);

                        if ui.button(None, "Start matchmaking") {
                            let mut matchmaker = Matchmaker::new();

                            matchmaker
                                .min(minimum_players.parse::<u32>().unwrap())
                                .max(maximum_players.parse::<u32>().unwrap())
                                .add_string_property("engine", "macroquad_matchmaking")
                                .add_query_item(
                                    &QueryItemBuilder::new("engine")
                                        .required()
                                        .term("macroquad_matchmaking")
                                        .build(),
                                );

                            nakama.api_client.socket_add_matchmaker(&matchmaker);

                            next_scene = Some(Scene::WaitingForMatchmaking { private: false });
                        }
                    }
                    1 => {
                        ui.group(hash!(), vec2(WINDOW_WIDTH / 2. - 38., 70.), |ui| {
                            if ui.button(None, "Create match") {
                                nakama.api_client.socket_create_match();
                                next_scene = Some(Scene::WaitingForMatchmaking { private: true });
                            }
                        });
                        ui.group(hash!(), vec2(WINDOW_WIDTH / 2. - 38., 80.), |ui| {
                            widgets::InputText::new(hash!())
                                .ratio(3. / 4.)
                                .label("Match ID")
                                .ui(ui, &mut match_id);
                            if ui.button(None, "Join match by ID") {
                                nakama.api_client.socket_join_match_by_id(&match_id);
                                next_scene = Some(Scene::WaitingForMatchmaking { private: true });
                            }
                        });
                    }
                    2 => {
                        if leaderboard_loaded == false {
                            nakama
                                .api_client
                                .list_leaderboard_records("fish_game_macroquad_wins");
                            leaderboard_loaded = true;
                        }
                        if let Some(leaderboard) = nakama
                            .api_client
                            .leaderboard_records("fish_game_macroquad_wins")
                        {
                            for record in &leaderboard.records {
                                ui.label(None, &format!("{}", record.username));
                                ui.same_line(300.0);
                                ui.label(None, &format!("{}", record.score));
                            }
                        }
                        // ui.push_skin(&resources.cheat_skin);
                        // if ui.button(None, "Add record") {
                        //     nakama
                        //         .api_client
                        //         .write_leaderboard_record("fish_game_macroquad_wins", 1);
                        // }
                        // ui.pop_skin();
                    }
                    _ => unreachable!(),
                }

                if ui.button(vec2(560.0, 200.0), "Back") {
                    next_scene = Some(Scene::MainMenu);
                }
            },
        );
        drop(nakama);

        root_ui().pop_skin();

        if let Some(scene) = next_scene {
            return scene;
        }

        next_frame().await;
    }
}
