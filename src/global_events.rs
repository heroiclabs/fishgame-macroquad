use macroquad::{
    experimental::{
        collections::storage,
        scene::{self, Handle, RefMut},
    },
    prelude::*,
};

use crate::{NetSyncronizer, Pickup, Player, RemotePlayer, Resources};

pub struct GlobalEvents {
    last_spawn_time: f64,
    player: Handle<Player>,
    spawned_items: Vec<(usize, Handle<Pickup>)>,

    uid: usize,
    net_syncronizer: Handle<NetSyncronizer>,
}

impl GlobalEvents {
    const SPAWN_INTERVAL: f32 = 2.0;

    pub fn new(player: Handle<Player>, net_syncronizer: Handle<NetSyncronizer>) -> GlobalEvents {
        GlobalEvents {
            player,
            net_syncronizer,
            last_spawn_time: 0.0,
            uid: 0,
            spawned_items: vec![],
        }
    }
}

impl scene::Node for GlobalEvents {
    fn update(mut node: RefMut<Self>) {
        let mut net_syncronizer = scene::get_node(node.net_syncronizer).unwrap();

        if net_syncronizer.is_host() == false {
            return;
        }

        if get_time() - node.last_spawn_time >= Self::SPAWN_INTERVAL as _
            && node.spawned_items.len() < 3
        {
            let resources = storage::get::<Resources>().unwrap();
            let pos = loop {
                let pos = vec2(rand::gen_range(20., 300.), rand::gen_range(20., 120.));
                if resources.collision_world.collide_solids(pos, 8, 8) == false
                    && (resources.collision_world.solid_at(pos + vec2(0., 17.))
                        || resources.collision_world.solid_at(pos + vec2(0., 23.)))
                {
                    break pos;
                }
            };

            node.last_spawn_time = get_time();

            let item_id = node.uid;
            node.spawned_items
                .push((item_id, scene::add_node(Pickup::new(pos))));
            net_syncronizer.spawn_item(item_id, pos);

            node.uid += 1;
        }

        let mut player = scene::get_node(node.player).unwrap();
        let mut others = scene::find_nodes_by_type::<RemotePlayer>();

        node.spawned_items.retain(|(id, item_handle)| {
            let item = scene::get_node(*item_handle);
            // already destroyed itself
            if item.is_none() {
                net_syncronizer.delete_item(*id);
                return false;
            }
            let item = item.unwrap();

            if player.pos().distance(item.pos) < 10.0 {
                player.pick_weapon();
                item.delete();

                net_syncronizer.delete_item(*id);
                net_syncronizer.pick_up_item(*id, None);
                return false;
            }

            let other = others.find(|other| other.fish.pos().distance(item.pos) < 10.0);

            if let Some(other) = other {
                item.delete();

                net_syncronizer.delete_item(*id);
                net_syncronizer.pick_up_item(*id, Some(&other.network_id));
                return false;
            }

            return true;
        });
    }
}
