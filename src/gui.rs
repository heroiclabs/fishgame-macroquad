use macroquad::{
    color::{Color, BLACK},
    experimental::collections::storage,
    math::{vec2, RectOffset, Vec2},
    texture::Image,
    time::get_frame_time,
    ui::{hash, root_ui, widgets, Skin},
    window::{clear_background, next_frame, screen_height, screen_width},
};

use crate::nakama::ApiClient;

const WINDOW_WIDTH: f32 = 700.0;
const WINDOW_HEIGHT: f32 = 300.0;

pub enum Scene {
    MainMenu,
    MatchmakingLobby,
    Credits,
    Login,
    QuickGame,
    MatchmakingGame { private: bool },
    WaitingForMatchmaking { private: bool },
}

pub struct GuiResources {
    pub title_skin: Skin,
    pub login_skin: Skin,
    pub authenticating_skin: Skin,
    pub error_skin: Skin,
    pub cheat_skin: Skin,
}

impl GuiResources {
    pub fn new() -> GuiResources {
        let title_skin = {
            let label_style = root_ui()
                .style_builder()
                .font(include_bytes!("../assets/ui/MinimalPixel v2.ttf"))
                .text_color(Color::from_rgba(255, 255, 255, 255))
                .font_size(130)
                .build();

            let button_style = root_ui()
                .style_builder()
                .background(Image::from_file_with_format(
                    include_bytes!("../assets/ui/button_background_2.png"),
                    None,
                ))
                .background_margin(RectOffset::new(8.0, 8.0, 12.0, 12.0))
                .margin(RectOffset::new(8.0, 8.0, 110.0, 12.0))
                .background_hovered(Image::from_file_with_format(
                    include_bytes!("../assets/ui/button_hovered_background_2.png"),
                    None,
                ))
                .background_clicked(Image::from_file_with_format(
                    include_bytes!("../assets/ui/button_clicked_background_2.png"),
                    None,
                ))
                .font(include_bytes!("../assets/ui/MinimalPixel v2.ttf"))
                .text_color(Color::from_rgba(200, 200, 160, 255))
                .font_size(45)
                .build();

            Skin {
                label_style,
                button_style,
                ..root_ui().default_skin()
            }
        };

        let login_skin = {
            let label_style = root_ui()
                .style_builder()
                .font(include_bytes!("../assets/ui/MinimalPixel v2.ttf"))
                .text_color(Color::from_rgba(200, 200, 160, 255))
                .font_size(20)
                .build();

            let window_style = root_ui()
                .style_builder()
                .background(Image::from_file_with_format(
                    include_bytes!("../assets/ui/window_background_2.png"),
                    None,
                ))
                .background_margin(RectOffset::new(52.0, 52.0, 52.0, 52.0))
                .margin(RectOffset::new(-30.0, -30.0, -30.0, -30.0))
                .build();

            let button_style = root_ui()
                .style_builder()
                .background(Image::from_file_with_format(
                    include_bytes!("../assets/ui/button_background_2.png"),
                    None,
                ))
                .background_margin(RectOffset::new(8.0, 8.0, 12.0, 12.0))
                .background_hovered(Image::from_file_with_format(
                    include_bytes!("../assets/ui/button_hovered_background_2.png"),
                    None,
                ))
                .background_clicked(Image::from_file_with_format(
                    include_bytes!("../assets/ui/button_clicked_background_2.png"),
                    None,
                ))
                .font(include_bytes!("../assets/ui/MinimalPixel v2.ttf"))
                .text_color(Color::from_rgba(200, 200, 160, 255))
                .font_size(25)
                .build();

            let tabbar_style = root_ui()
                .style_builder()
                .background_margin(RectOffset::new(8.0, 8.0, 12.0, 12.0))
                .font(include_bytes!("../assets/ui/MinimalPixel v2.ttf"))
                .color(Color::from_rgba(58, 68, 102, 255))
                .color_hovered(Color::from_rgba(149, 165, 190, 255))
                .color_clicked(Color::from_rgba(129, 145, 170, 255))
                .color_selected(Color::from_rgba(139, 155, 180, 255))
                .color_selected_hovered(Color::from_rgba(149, 165, 190, 255))
                .text_color(Color::from_rgba(255, 255, 255, 255))
                .font_size(20)
                .build();

            let editbox_style = root_ui()
                .style_builder()
                .background(Image::from_file_with_format(
                    include_bytes!("../assets/ui/editbox_background2.png"),
                    None,
                ))
                .background_clicked(Image::from_file_with_format(
                    include_bytes!("../assets/ui/editbox_background.png"),
                    None,
                ))
                .font(include_bytes!("../assets/ui/MinimalPixel v2.ttf"))
                .background_margin(RectOffset::new(2., 2., 2., 2.))
                .text_color(Color::from_rgba(120, 120, 120, 255))
                .font_size(20)
                .build();

            Skin {
                label_style,
                button_style,
                tabbar_style,
                window_style,
                editbox_style,
                ..root_ui().default_skin()
            }
        };

        let authenticating_skin = {
            let label_style = root_ui()
                .style_builder()
                .font(include_bytes!("../assets/ui/MinimalPixel v2.ttf"))
                .text_color(Color::from_rgba(200, 200, 160, 255))
                .font_size(35)
                .build();

            Skin {
                label_style,
                ..root_ui().default_skin()
            }
        };
        let error_skin = {
            let label_style = root_ui()
                .style_builder()
                .font(include_bytes!("../assets/ui/MinimalPixel v2.ttf"))
                .text_color(Color::from_rgba(255, 0, 0, 255))
                .font_size(20)
                .build();

            Skin {
                label_style,
                ..root_ui().default_skin()
            }
        };

        let cheat_skin = root_ui().default_skin();

        GuiResources {
            title_skin,
            login_skin,
            authenticating_skin,
            error_skin,
            cheat_skin,
        }
    }
}

pub async fn main_menu() -> Scene {
    loop {
        clear_background(BLACK);

        let resources = storage::get::<GuiResources>().unwrap();
        root_ui().push_skin(&resources.title_skin);

        let title = "FISH GAME";
        let label_size = root_ui().calc_size(title);
        let label_pos = vec2(screen_width() / 2. - label_size.x / 2., 100.);
        root_ui().label(Some(label_pos), title);

        let button_width = 300.0;

        if widgets::Button::new("Quick game")
            .size(vec2(button_width, 300.))
            .position(vec2(
                screen_width() / 2. - ((button_width + 10.) * 3.) / 2.,
                label_pos.y + label_size.y + 50.,
            ))
            .ui(&mut *root_ui())
        {
            root_ui().pop_skin();
            return Scene::QuickGame;
        }

        if widgets::Button::new("      Login")
            .size(vec2(button_width, 300.))
            .position(vec2(
                screen_width() / 2. - button_width / 2.,
                label_pos.y + label_size.y + 50.,
            ))
            .ui(&mut *root_ui())
        {
            root_ui().pop_skin();
            return Scene::Login;
        }

        widgets::Button::new("    Credits")
            .size(vec2(button_width, 300.))
            .position(vec2(
                screen_width() / 2. + button_width / 2. + 10.,
                label_pos.y + label_size.y + 50.,
            ))
            .ui(&mut *root_ui());

        root_ui().pop_skin();

        next_frame().await;
    }
}

pub async fn authentication() -> Scene {
    let mut email = String::new();
    let mut password = String::new();

    let mut email_new = String::new();
    let mut username_new = String::new();
    let mut password_new = String::new();

    let mut authenticating = false;

    let mut dots_amount = 0.;

    loop {
        let resources = storage::get::<GuiResources>().unwrap();
        root_ui().push_skin(&resources.login_skin);

        let mut nakama = storage::get_mut::<ApiClient>().unwrap();

        let mut next_scene = None;
        root_ui().window(
            hash!(),
            Vec2::new(
                screen_width() / 2. - WINDOW_WIDTH / 2.,
                screen_height() / 2. - WINDOW_HEIGHT / 2.,
            ),
            Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            |ui| {
                if nakama.in_progress() {
                    dots_amount += get_frame_time() * 1.5;

                    if dots_amount >= 4. {
                        dots_amount = 0.;
                    }
                    ui.push_skin(&resources.authenticating_skin);
                    ui.label(
                        Some(vec2(190., WINDOW_HEIGHT / 2. - 40.)),
                        &format!("Authenticating{}", ".".repeat(dots_amount as usize)),
                    );
                    ui.pop_skin();
                    return;
                }
                ui.group(hash!(), vec2(WINDOW_WIDTH / 2. - 28., 170.), |ui| {
                    ui.label(None, "Login");
                    widgets::InputText::new(hash!())
                        .ratio(3. / 4.)
                        .label("Email")
                        .ui(ui, &mut email);

                    widgets::InputText::new(hash!())
                        .ratio(3. / 4.)
                        .password(true)
                        .label("Password")
                        .ui(ui, &mut password);

                    ui.separator();

                    if ui.button(None, "Login") {
                        nakama.authenticate(&email, &password);
                    }
                    ui.push_skin(&resources.cheat_skin);
                    if ui.button(None, "Fast cheating login") {
                        email = "super@heroes.com".to_owned();
                        password = "batsignal".to_owned();
                    }
                    ui.pop_skin();
                });
                ui.group(hash!(), vec2(WINDOW_WIDTH / 2. - 28., 170.), |ui| {
                    ui.label(None, "Create an account");
                    widgets::InputText::new(hash!())
                        .ratio(3. / 4.)
                        .label("Email")
                        .ui(ui, &mut email_new);

                    widgets::InputText::new(hash!())
                        .ratio(3. / 4.)
                        .label("Username")
                        .ui(ui, &mut username_new);

                    widgets::InputText::new(hash!())
                        .ratio(3. / 4.)
                        .password(true)
                        .label("Password")
                        .ui(ui, &mut password_new);

                    ui.separator();

                    if ui.button(None, "Register") {
                        authenticating = true;
                        nakama.register(&email_new, &password_new, &username_new);
                    }
                });

                ui.push_skin(&resources.error_skin);
                {
                    ui.label(None, &nakama.error().as_deref().unwrap_or(""));
                }
                ui.pop_skin();

                ui.same_line(570.);
                if ui.button(None, "Back") {
                    next_scene = Some(Scene::MainMenu);
                }
            },
        );

        root_ui().pop_skin();

        if nakama.authenticated() {
            return Scene::MatchmakingLobby;
        }

        if let Some(next_scene) = next_scene {
            return next_scene;
        }

        drop(nakama);

        next_frame().await;
    }
}

pub async fn matchmaking_lobby() -> Scene {
    let username: String = storage::get::<ApiClient>().unwrap().username().unwrap();
    let mut minimum_players = "2".to_string();
    let mut maximum_players = "4".to_string();

    let mut match_id = String::new();

    let resources = storage::get::<GuiResources>().unwrap();

    let mut leaderboard_loaded = false;

    loop {
        root_ui().push_skin(&resources.login_skin);

        let mut next_scene = None;

        let mut nakama = storage::get_mut::<ApiClient>().unwrap();

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
                            nakama.socket_add_matchmaker(
                                minimum_players.parse::<u32>().unwrap(),
                                maximum_players.parse::<u32>().unwrap(),
                                "+properties.engine:\\\"macroquad_matchmaking\\\"",
                                "{\"engine\":\"macroquad_matchmaking\"}",
                            );

                            next_scene = Some(Scene::WaitingForMatchmaking { private: false });
                        }
                    }
                    1 => {
                        ui.group(hash!(), vec2(WINDOW_WIDTH / 2. - 38., 70.), |ui| {
                            if ui.button(None, "Create match") {
                                nakama.socket_create_match();
                                next_scene = Some(Scene::WaitingForMatchmaking { private: true });
                            }
                        });
                        ui.group(hash!(), vec2(WINDOW_WIDTH / 2. - 38., 80.), |ui| {
                            widgets::InputText::new(hash!())
                                .ratio(3. / 4.)
                                .label("Match ID")
                                .ui(ui, &mut match_id);
                            if ui.button(None, "Join match by ID") {
                                nakama.socket_join_match_by_id(&match_id);
                                next_scene = Some(Scene::WaitingForMatchmaking { private: true });
                            }
                        });
                    }
                    2 => {
                        if leaderboard_loaded == false {
                            //nakama::load_leaderboard_records();
                            leaderboard_loaded = true;
                        }
                        // if let Some(records) = nakama::leaderboard_records() {
                        //     for record in records {
                        //         ui.label(None, &format!("{}", record.username));
                        //         ui.same_line(300.0);
                        //         ui.label(None, &format!("{}", record.score));
                        //     }
                        // }
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

pub async fn waiting_for_matchmaking(private: bool) -> Scene {
    let mut dots_amount = 0.;

    let resources = storage::get::<GuiResources>().unwrap();

    enum State {
        WaitingForMatchmaking,
        WaitingForMatchJoin,
    }
    let mut state = if private {
        State::WaitingForMatchJoin
    } else {
        State::WaitingForMatchmaking
    };

    loop {
        root_ui().push_skin(&resources.login_skin);

        let mut next_scene = None;

        let mut nakama = storage::get_mut::<ApiClient>().unwrap();

        root_ui().window(
            hash!(),
            Vec2::new(
                screen_width() / 2. - WINDOW_WIDTH / 2.,
                screen_height() / 2. - WINDOW_HEIGHT / 2.,
            ),
            Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            |ui| match state {
                State::WaitingForMatchmaking => {
                    let token = nakama.matchmaker_token.clone();
                    if token.is_none() {
                        dots_amount += get_frame_time() * 1.5;

                        if dots_amount >= 4. {
                            dots_amount = 0.;
                        }
                        ui.push_skin(&resources.authenticating_skin);
                        ui.label(
                            Some(vec2(150., WINDOW_HEIGHT / 2. - 40.)),
                            &format!("Looking for a match{}", ".".repeat(dots_amount as usize)),
                        );
                        ui.pop_skin();
                        return;
                    }
                    if let Some(_error) = nakama.error().clone() {
                        ui.label(None, "Invalid match ID");
                        if ui.button(None, "Back to matchmaking") {
                            next_scene = Some(Scene::MatchmakingLobby);
                        }
                    } else {
                        nakama.socket_join_match_by_token(&token.unwrap());
                        state = State::WaitingForMatchJoin;
                    }
                }
                State::WaitingForMatchJoin => {
                    if nakama.match_id().is_some() {
                        next_scene = Some(Scene::MatchmakingGame { private });
                    }
                }
            },
        );

        root_ui().pop_skin();

        drop(nakama);

        if let Some(scene) = next_scene {
            return scene;
        }

        next_frame().await;
    }
}
