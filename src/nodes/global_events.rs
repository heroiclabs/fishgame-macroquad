use macroquad::{
    experimental::{
        collections::storage,
        scene::{self, Handle, RefMut},
    },
    prelude::*,
};

use crate::{
    nodes::{pickup::ItemType, NakamaRealtimeGame, Pickup, Player, RemotePlayer},
    Resources,
};

pub struct GlobalEvents {
    last_spawn_time: f64,
    _player: Handle<Player>,
    spawned_items: Vec<(usize, Handle<Pickup>)>,

    uid: usize,
    nakama: Handle<NakamaRealtimeGame>,
}

impl GlobalEvents {
    const SPAWN_INTERVAL: f32 = 2.0;

    pub fn new(player: Handle<Player>, nakama: Handle<NakamaRealtimeGame>) -> GlobalEvents {
        GlobalEvents {
            _player: player,
            nakama,
            last_spawn_time: 0.0,
            uid: 0,
            spawned_items: vec![],
        }
    }
}

impl scene::Node for GlobalEvents {
    fn update(mut node: RefMut<Self>) {
        let mut nakama = scene::get_node(node.nakama);

        if nakama.is_host() == false || nakama.game_started() == false {
            return;
        }

        if get_time() - node.last_spawn_time >= Self::SPAWN_INTERVAL as _
            && node.spawned_items.len() < 3
        {
            let resources = storage::get::<Resources>();

            let tilewidth = resources.tiled_map.raw_tiled_map.tilewidth as f32;
            let w = resources.tiled_map.raw_tiled_map.width as f32;
            let tileheight = resources.tiled_map.raw_tiled_map.tileheight as f32;
            let h = resources.tiled_map.raw_tiled_map.height as f32;

            let pos = loop {
                let x = rand::gen_range(0, w as i32) as f32;
                let y = rand::gen_range(0, h as i32 - 6) as f32;

                let pos = vec2((x + 0.5) * tilewidth, (y - 0.5) * tileheight);

                if resources
                    .collision_world
                    .collide_solids(pos, tilewidth as _, tileheight as _)
                    == false
                    && resources.collision_world.collide_solids(
                        pos,
                        tilewidth as _,
                        tileheight as i32 * 3,
                    )
                    && Rect::new(5. * 32., 12. * 32., 8. * 32., 6. * 32.).contains(pos) == false
                {
                    break pos;
                }
            };

            node.last_spawn_time = get_time();

            let item_type = if rand::gen_range(0, 2) == 0 {
                ItemType::Gun
            } else {
                ItemType::Sword
            };
            let item_id = node.uid;
            node.spawned_items
                .push((item_id, scene::add_node(Pickup::new(pos, item_type))));
            nakama.spawn_item(item_id, pos, item_type);

            node.uid += 1;
        }

        let mut others = scene::find_nodes_by_type::<RemotePlayer>();

        node.spawned_items.retain(|(id, item_handle)| {
            let item = scene::try_get_node(*item_handle);
            // already destroyed itself.
            if item.is_none() {
                nakama.delete_item(*id);
                return false;
            }
            let item = item.unwrap();

            let collide = |player: Vec2, pickup: Vec2| {
                (player + vec2(16., 32.)).distance(pickup + vec2(16., 16.)) < 90.
            };

            let other = others.find(|other| collide(other.pos(), item.pos));

            if other.is_some() {
                item.delete();
                nakama.delete_item(*id);
                return false;
            }

            return true;
        });
    }
}
