use nanoserde::DeBin;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use shared::Message;

use quad_net::server::SocketHandle;

struct World {
    players: Vec<Option<(u16, u8)>>,
    unique_id: usize,
}

#[derive(Debug)]
enum ClientState {
    Unknown,
    Connected,
    Spawned { id: usize },
}

impl Default for ClientState {
    fn default() -> ClientState {
        ClientState::Unknown
    }
}

fn handle_message(
    out: &mut SocketHandle,
    state: &mut ClientState,
    msg: Vec<u8>,
    world: &Arc<Mutex<World>>,
) -> Option<()> {
    match state {
        ClientState::Unknown => {
            let handshake: shared::Handshake = DeBin::deserialize_bin(&msg).ok()?;
            if handshake.magic != shared::MAGIC {
                println!("Magic mismatch, not fishgame protocol!");
                return None;
            }
            if handshake.version != shared::PROTOCOL_VERSION {
                println!("Version mismatch, outdataed client!");
                return None;
            }
            *state = ClientState::Connected;
        }
        ClientState::Connected => {
            let msg: Message = DeBin::deserialize_bin(&msg).ok()?;
            match msg {
                Message::SpawnRequest => {
                    let id = world.lock().unwrap().unique_id;
                    world.lock().unwrap().players.push(Some((0, 0)));
                    world.lock().unwrap().unique_id += 1;

                    *state = ClientState::Spawned { id };
                    out.send_bin(&Message::Spawned(id)).ok()?;
                }
                _ => {
                    return None;
                }
            }
        }
        ClientState::Spawned { id } => {
            let msg: Message = DeBin::deserialize_bin(&msg).ok()?;
            match msg {
                Message::Move(x, y) => {
                    world.lock().unwrap().players[*id] = Some((x, y));
                }
                _ => {
                    return None;
                }
            }
        }
    }

    Some(())
}

pub fn tcp_main() -> std::io::Result<()> {
    let world = Arc::new(Mutex::new(World {
        players: vec![],
        unique_id: 0,
    }));

    quad_net::server::listen(
        "0.0.0.0:8090",
        "0.0.0.0:8091",
        quad_net::server::Settings {
            on_message: {
                let world = world.clone();
                move |mut out, state: &mut ClientState, msg| {
                    if handle_message(&mut out, state, msg, &world).is_none() {
                        out.disconnect();
                    }
                }
            },
            on_timer: {
                let world = world.clone();
                move |out, state| match state {
                    ClientState::Spawned { id } => {
                        if out
                            .send_bin(&Message::Players(
                                // remove self and remove dead players
                                world
                                    .lock()
                                    .unwrap()
                                    .players
                                    .iter()
                                    .enumerate()
                                    .filter(|(n, _)| n != id)
                                    .filter_map(|(_, player)| *player)
                                    .collect(),
                            ))
                            .is_err()
                        {
                            out.disconnect();
                        }
                    }
                    _ => {}
                }
            },
            on_disconnect: {
                let world = world.clone();

                move |state| match state {
                    ClientState::Spawned { id } => {
                        world.lock().unwrap().players[*id] = None;
                    }
                    _ => {}
                }
            },
            timer: Some(Duration::from_millis(1000 / 30)),
            _marker: std::marker::PhantomData,
        },
    );
    Ok(())
}
