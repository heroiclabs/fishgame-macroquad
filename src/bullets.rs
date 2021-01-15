use macroquad::{
    experimental::{collections::storage, scene::RefMut},
    prelude::*,
};

use crate::{consts, Player, RemotePlayer, Resources};

struct Bullet {
    pos: Vec2,
    speed: Vec2,
    lived: f32,
    lifetime: f32,
}

pub struct Bullets {
    player: scene::Handle<Player>,
    bullets: Vec<Bullet>,
}

impl Bullets {
    pub fn new(player: scene::Handle<Player>) -> Bullets {
        Bullets {
            player,
            bullets: Vec::with_capacity(200),
        }
    }

    pub fn spawn_bullet(&mut self, pos: Vec2, facing: bool) {
        let dir = if facing {
            vec2(1.0, 0.0)
        } else {
            vec2(-1.0, 0.0)
        };
        self.bullets.push(Bullet {
            pos: pos + vec2(4.0, 4.0) + dir * 8.0,
            speed: dir * consts::BULLET_SPEED,
            lived: 0.0,
            lifetime: 0.7,
        });
    }
}

impl scene::Node for Bullets {
    fn draw(node: RefMut<Self>) {
        for bullet in &node.bullets {
            draw_circle(
                bullet.pos.x,
                bullet.pos.y,
                1.,
                Color::new(1.0, 1.0, 0.8, 1.0),
            );
        }
    }

    fn update(mut node: RefMut<Self>) {
        let mut resources = storage::get_mut::<Resources>().unwrap();
        let mut player = scene::get_node(node.player).unwrap();
        let mut others = scene::find_nodes_by_type::<RemotePlayer>();

        for bullet in &mut node.bullets {
            bullet.pos += bullet.speed * get_frame_time();
            bullet.lived += get_frame_time();
        }

        node.bullets.retain(|bullet| {
            let self_damaged =
                Rect::new(player.pos().x, player.pos().y, 8., 8.).contains(bullet.pos);

            if self_damaged {
                player.kill();
            }

            if resources.collision_world.solid_at(bullet.pos)
                || others.any(|other| {
                    Rect::new(other.fish.pos().x, other.fish.pos().y, 8.0, 8.0).contains(bullet.pos)
                })
                || self_damaged
            {
                resources.hit_fxses.spawn(bullet.pos);
                return false;
            }
            bullet.lived < bullet.lifetime
        });
    }
}
