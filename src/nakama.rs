//! Nakama client
//! Right now supports only web and only very few nakama calls
//! Eventually going to be replaced with nakama crate

use nanoserde::DeJson;

pub struct MatchData {
    pub data: Vec<u8>,
    pub opcode: i32,
    pub user_id: String,
}

#[allow(dead_code)]
pub enum Event {
    Join {
        network_id: String,
        username: String,
    },
    Leave(String),
}

#[derive(Debug, Clone, DeJson, PartialEq)]
pub struct LeaderboardRecord {
    pub username: String,
    pub score: i32,
}

#[cfg(target_arch = "wasm32")]
mod nakama {
    use super::{Event, LeaderboardRecord, MatchData};

    use sapp_jsutils::JsObject;

    extern "C" {
        fn nakama_connect(key: JsObject, server: JsObject, port: u32, protocol: JsObject);
        fn nakama_authenticate(email: JsObject, password: JsObject);
        fn nakama_logout();
        fn nakama_create_private_match();
        fn nakama_add_matchmaker();
        fn nakama_register(email: JsObject, password: JsObject, username: JsObject);
        fn nakama_self_id() -> JsObject;
        fn nakama_error() -> JsObject;
        fn nakama_username() -> JsObject;
        fn nakama_send(opcode: i32, data: JsObject);
        fn nakama_authenticated() -> bool;
        fn nakama_in_progress() -> bool;
        fn nakama_try_recv() -> JsObject;
        fn nakama_events() -> JsObject;
        fn nakama_join_match(match_id: JsObject);
        fn nakama_join_quick_match();
        fn nakama_leave_match();
        fn nakama_match_id() -> JsObject;
        fn nakama_add_leaderboard_win();
        fn nakama_load_leaderboard_records();
        fn nakama_leaderboard_records() -> JsObject;
    }

    #[no_mangle]
    pub extern "C" fn quad_nakama_crate_version() -> u32 {
        (0 << 24) + (1 << 16) + 1
    }

    pub fn connect(key: &str, server: &str, port: u32, protocol: &str) {
        unsafe {
            nakama_connect(
                JsObject::string(key),
                JsObject::string(server),
                port,
                JsObject::string(protocol),
            );
        }
    }

    pub fn authenticate(email: &str, password: &str) {
        unsafe {
            nakama_authenticate(JsObject::string(email), JsObject::string(password));
        }
    }

    pub fn logout() {
        unsafe {
            nakama_logout();
        }
    }

    pub fn leave_match() {
        unsafe {
            nakama_leave_match();
        }
    }

    pub fn create_private_match() {
        unsafe {
            nakama_create_private_match();
        }
    }

    pub fn add_matchmaker() {
        unsafe {
            nakama_add_matchmaker();
        }
    }

    pub fn register(email: &str, password: &str, username: &str) {
        unsafe {
            nakama_register(
                JsObject::string(email),
                JsObject::string(password),
                JsObject::string(username),
            );
        }
    }

    pub fn async_in_progress() -> bool {
        unsafe { nakama_in_progress() }
    }

    pub fn authenticated() -> bool {
        unsafe { nakama_authenticated() }
    }

    pub fn add_leaderboard_win() {
        unsafe { nakama_add_leaderboard_win() }
    }

    pub fn error() -> Option<String> {
        let js_obj = unsafe { nakama_error() };
        if js_obj.is_nil() {
            return None;
        }
        let mut error = String::new();
        js_obj.to_string(&mut error);
        Some(error)
    }

    pub fn username() -> Option<String> {
        let js_obj = unsafe { nakama_username() };
        if js_obj.is_nil() {
            return None;
        }
        let mut username = String::new();
        js_obj.to_string(&mut username);
        Some(username)
    }

    pub fn match_id() -> Option<String> {
        let js_obj = unsafe { nakama_match_id() };
        if js_obj.is_nil() {
            return None;
        }
        let mut match_id = String::new();
        js_obj.to_string(&mut match_id);
        Some(match_id)
    }

    pub fn self_id() -> String {
        let mut id = String::new();
        let js_obj = unsafe { nakama_self_id() };
        js_obj.to_string(&mut id);

        id
    }

    pub fn send(opcode: i32, data: &[u8]) {
        unsafe { nakama_send(opcode, JsObject::buffer(data)) }
    }

    pub fn send_bin<T: nanoserde::SerBin>(opcode: i32, data: &T) {
        use nanoserde::SerBin;

        send(opcode, &SerBin::serialize_bin(data));
    }

    pub fn try_recv() -> Option<MatchData> {
        let js_obj = unsafe { nakama_try_recv() };
        if js_obj.is_nil() == false {
            let mut buf = vec![];
            let mut user_id = String::new();

            let opcode = js_obj.field_u32("opcode") as i32;
            js_obj.field("data").to_byte_buffer(&mut buf);
            js_obj.field("user_id").to_string(&mut user_id);

            return Some(MatchData {
                opcode,
                user_id,
                data: buf,
            });
        }
        None
    }

    pub fn events() -> Option<Event> {
        let js_obj = unsafe { nakama_events() };
        if js_obj.is_nil() == false {
            let mut user_id = String::new();
            js_obj.field("user_id").to_string(&mut user_id);

            let event_type = js_obj.field_u32("event");

            match event_type {
                1 => {
                    let mut username = String::new();
                    js_obj.field("username").to_string(&mut username);

                    macroquad::prelude::warn!("{}", username);
                    return Some(Event::Join {
                        network_id: user_id,
                        username,
                    });
                }
                2 => return Some(Event::Leave(user_id)),
                _ => panic!("Unknown nakama event type"),
            }
        }
        None
    }

    pub async fn join_quick_match() -> String {
        unsafe { nakama_join_quick_match() };

        loop {
            let js_obj = unsafe { nakama_match_id() };
            if js_obj.is_nil() == false {
                let mut match_id = String::new();

                js_obj.to_string(&mut match_id);

                return match_id;
            }

            macroquad::window::next_frame().await;
        }
    }

    pub fn join_match(match_id: &str) {
        unsafe {
            nakama_join_match(JsObject::string(match_id));
        }
    }

    pub fn load_leaderboard_records() {
        unsafe { nakama_load_leaderboard_records() };
    }

    pub fn leaderboard_records() -> Option<Vec<LeaderboardRecord>> {
        let js_obj = unsafe { nakama_leaderboard_records() };
        let mut json = String::new();
        js_obj.to_string(&mut json);
        nanoserde::DeJson::deserialize_json(&json).ok()?
    }
}

// just enough of stubs to run the game on PC, but no real networking involved
#[cfg(not(target_arch = "wasm32"))]
mod nakama {
    use super::{Event, MatchData};

    pub fn connect(_key: &str, _server: &str, _port: u32, _protocol: &str) {}

    pub fn self_id() -> String {
        "self".to_string()
    }

    pub fn send_bin<T: nanoserde::SerBin>(_opcode: i32, _data: &T) {}

    pub fn try_recv() -> Option<MatchData> {
        None
    }

    pub fn events() -> Option<Event> {
        None
    }

    pub fn authenticate(_email: &str, _password: &str) {}
    pub fn logout() {}
    pub fn create_private_match() {}
    pub fn add_matchmaker() {}

    pub fn register(_email: &str, _password: &str, _username: &str) {}

    pub fn async_in_progress() -> bool {
        false
    }

    pub fn leave_match() {}

    pub fn authenticated() -> bool {
        false
    }

    pub fn match_id() -> Option<String> {
        None
    }

    pub async fn join_quick_match() -> String {
        return "".to_string();
    }

    pub fn join_match(match_id: &str) {}

    pub fn error() -> Option<String> {
        Some("Nakama on desktop is not supported.".to_string())
    }

    pub fn username() -> Option<String> {
        None
    }

    pub fn add_leaderboard_win() {}
    pub fn load_leaderboard_records() {}

    pub fn leaderboard_records() -> Option<Vec<LeaderboardRecord>> {
        None
    }
}

pub use nakama::*;
