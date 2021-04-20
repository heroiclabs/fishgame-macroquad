use macroquad::experimental::scene::{self, RefMut};

use nakama_rs::api_client::ApiClient;

mod nakama_realtime_game;

pub use nakama_realtime_game::NakamaRealtimeGame;

/// Persisted singleton node keeping nakama connection
/// alive across the whole game
pub struct Nakama {
    pub api_client: ApiClient,
}

impl Nakama {
    pub fn new(key: &str, server: &str, port: u32, protocol: &str) -> Nakama {
        Nakama {
            api_client: ApiClient::new(key, server, port, protocol),
        }
    }
}

impl scene::Node for Nakama {
    fn ready(node: RefMut<Self>) {
        node.persist();
    }

    fn update(mut node: RefMut<Self>) {
        let node = &mut *node;
        node.api_client.tick();
    }
}
