use macroquad::{
    math::vec2,
    time::get_time,
    ui::{Skin, Ui},
};

mod authentication;
mod lobby;
mod main_menu;
mod style;
mod waitscreen;

pub use authentication::authentication;
pub use lobby::matchmaking_lobby;
pub use main_menu::main_menu;
pub use style::GuiResources;
pub use waitscreen::waitscreen;

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

pub fn in_progress_gui(ui: &mut Ui, label: &str, skin: &Skin) {
    let dots_amount = (get_time() as i32) % 4;

    ui.push_skin(skin);
    ui.label(
        Some(vec2(190., WINDOW_HEIGHT / 2. - 40.)),
        &format!("{}{}", label, ".".repeat(dots_amount as usize)),
    );
    ui.pop_skin();
}
