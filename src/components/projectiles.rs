use crate::*;

#[derive(Component)]
pub struct KineticDamage(pub f32);

#[derive(Component)]
pub struct EnergyDamage(pub f32);

#[derive(Component)]
pub struct LinearVelocity(pub f32);

#[derive(Component)]
pub struct Decaying {
    pub decay_timer: Timer,
    pub decay_type: DecayType
}

#[derive(Debug, PartialEq, Eq)]
pub enum DecayType {
    Transparency,
    Despawn
}

#[derive(Component)]
pub struct Homing{
    pub homing_distance: f32,
    pub homing_angle: f32,
    pub homing_speed: f32
}

#[derive(Component)]
pub struct Explosive {
    pub radius: f32,
    pub damage: f32
}

#[derive(Component)]
pub struct ExplosionToSpawn {
    pub radius: f32,
    pub damage: f32,
    pub pos: Vec3
}

#[derive(Component)]
pub struct Explosion {
    pub radius: f32,
    pub pos: Vec3
}

#[derive(Component)]
pub struct AoEAttack {
    pub radius: f32,
    pub pos: Vec3
}

#[derive(Component)]
pub struct Projectile {
    pub auto_despawn: bool,
    pub radius: f32
}
