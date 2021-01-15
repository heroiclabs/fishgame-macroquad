pub trait Weapon {
    fn is_on_cooldown(&self) -> bool;
    fn shoot(&mut self);
}

struct Pistol {}

impl Weapon for Pistol {
    fn is_on_cooldown(&self) -> bool {
        false
    }

    fn shoot(&mut self) {}
}
