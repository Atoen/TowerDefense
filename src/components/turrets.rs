use bevy::{math::Vec3, prelude::{Component, Entity}, time::Timer};
use bevy_rand::prelude::EntropyComponent;

#[derive(Component)]
pub struct Turret;

#[derive(Component)]
pub struct TargetingTurret {
    pub targeting_radius: Option<f32>,
    pub rotation: f32,
    pub has_target: bool
}

#[derive(Component)]
pub struct PreciseAttack;

#[derive(Component)]
pub struct AttackDispersion(pub f32);

#[derive(Component)]
pub struct SpawnOffset(pub Vec3);

#[derive(Component)]
pub struct RotationSpeed(pub f32);

#[derive(Component)]
pub struct AttackDelay(pub Timer);

#[derive(Component)]
pub struct IdleRotation {
    pub idle_timer: Timer,
    pub rotation_timer: Timer,
    pub is_idle: bool,
    pub target_angle: f32
}



#[derive(Component)]
pub struct ProjectileTurret;

#[derive(Component)]
pub struct LaserTurret {
    pub laser_beam: Option<Entity>
}



#[derive(Component)]
pub struct Projectile {
    pub auto_despawn: bool,
    pub radius: f32
}

#[derive(Component)]
pub struct LaserBeam;


#[derive(Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct InstantDamage(pub f32);

#[derive(Component)]
pub struct ContinousDamage {
    pub damage_per_second: f32
}

#[derive(Component)]
pub struct Decaying {
    pub decay_timer: Timer
}

#[derive(Component)]
pub struct Target {
    pub pos: Vec3
}