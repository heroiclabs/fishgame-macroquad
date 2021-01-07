    use sapp_jsutils::JsObject;

    extern "C" {
        fn nakama_is_connected() -> bool;
        fn nakama_send(opcode: i32, data: JsObject);
        fn nakama_try_recv() -> JsObject;

    }

    #[no_mangle]
    pub extern "C" fn quad_nakama_crate_version() -> u32 {
        (0 << 24) + (1 << 16) + 0
    }

    pub fn connected() -> bool {
        unsafe { nakama_is_connected() }
    }

    pub fn send(data: &[u8]) {
        unsafe { nakama_send(1, JsObject::buffer(data)) }
    }

    pub fn send_bin<T: nanoserde::SerBin>(data: &T) {
        use nanoserde::SerBin;

        send(&SerBin::serialize_bin(data));
    }

    pub fn try_recv() -> Option<Vec<u8>> {
        let data = unsafe { nakama_try_recv() };
        if data.is_nil() == false {
            let mut buf = vec![];
            data.to_byte_buffer(&mut buf);
            return Some(buf);
        }
        None
    }

    pub fn try_recv_bin<T: nanoserde::DeBin + std::fmt::Debug>() -> Option<T> {
        let bytes = try_recv()?;
        let data: T = nanoserde::DeBin::deserialize_bin(&bytes).expect("Cant parse message");

        Some(data)
    }
