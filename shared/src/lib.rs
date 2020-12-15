use nanoserde::{DeBin, SerBin};

pub const MAGIC: u64 = 16727391561867853824;
pub const PROTOCOL_VERSION: u32 = 2;

#[derive(Debug, Clone, SerBin, DeBin, PartialEq)]
pub struct Handshake {
    pub magic: u64,
    pub version: u32,
}

#[derive(Debug, Clone, SerBin, DeBin, PartialEq)]
pub enum Message {
    Move(u16, u8),
    Players(Vec<(u16, u8)>),
    SpawnRequest,
    Spawned(usize),
}
