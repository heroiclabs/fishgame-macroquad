pub struct MatchData {
    pub data: Vec<u8>,
    pub opcode: i32,
    pub user_id: String,
}

#[allow(dead_code)]
pub enum Event {
    Join(String),
    Leave(String),
}

#[cfg(target_arch = "wasm32")]
mod nakama {
    use super::{Event, MatchData};
    use sapp_jsutils::JsObject;

    extern "C" {
        fn nakama_is_connected() -> bool;
        fn nakama_self_id() -> JsObject;
        fn nakama_send(opcode: i32, data: JsObject);
        fn nakama_try_recv() -> JsObject;
        fn nakama_events() -> JsObject;

    }

    #[no_mangle]
    pub extern "C" fn quad_nakama_crate_version() -> u32 {
        (0 << 24) + (1 << 16) + 1
    }

    pub fn connected() -> bool {
        unsafe { nakama_is_connected() }
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
                1 => return Some(Event::Join(user_id)),
                2 => return Some(Event::Leave(user_id)),
                _ => panic!("Unknown nakama event type"),
            }
        }
        None
    }
}

// just enough of stubs to run the game on PC, but no real networking involved
#[cfg(not(target_arch = "wasm32"))]
mod nakama {
    use super::{Event, MatchData};
    use macroquad::experimental::collections::storage;

    struct Wtf {
        spawned: bool,
    }

    pub fn self_id() -> String {
        "aaa self".to_string()
    }

    pub fn send(_opcode: i32, _data: &[u8]) {}

    pub fn send_bin<T: nanoserde::SerBin>(opcode: i32, data: &T) {
        use nanoserde::SerBin;

        send(opcode, &SerBin::serialize_bin(data));
    }

    pub fn try_recv() -> Option<MatchData> {
        if storage::get::<Wtf>().is_none() {
            storage::store(Wtf { spawned: false });
        }
        let wtf = storage::get_mut::<Wtf>().unwrap();

        if wtf.spawned {}
        None
    }

    pub fn events() -> Option<Event> {
        if storage::get::<Wtf>().is_none() {
            storage::store(Wtf { spawned: false });
        }
        let mut wtf = storage::get_mut::<Wtf>().unwrap();

        if wtf.spawned {
            return None;
        }
        wtf.spawned = true;
        return Some(Event::Join("other".to_string()));
    }
}

pub use nakama::*;
