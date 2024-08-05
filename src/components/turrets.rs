use std::time::Duration;

use bevy::{color::Color, math::{Vec2, Vec3}, prelude::{Component, Entity}, time::Timer};
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
pub struct AoETurret {
    pub always_attacking: bool,
    pub range: f32
}

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
pub struct LinearVelocity(pub f32);


#[derive(Component)]
pub struct InstantDamage(pub f32);

#[derive(Component)]
pub struct ContinousDamage {
    pub damage_per_second: f32
}

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
pub struct AoEAnimation {
    pub timer: Timer,
    pub radius_animation: Option<RadiusAnimation>,
    pub color_animation: Option<ColorAnimation>,
    pub despawn_on_end: bool
}

pub enum RadiusAnimation {
    FromBaseRadius { grow_speed: f32 },
    FromStartToEnd { start_radius: f32, end_radius: f32 },
}

pub struct ColorAnimation {
    pub start_color: Color,
    pub end_color: Color,
    pub alpha_factor: Option<f32>,
    pub animate_alpha: bool,
}

#[derive(Component)]
pub struct AoEAttack {
    pub radius: f32,
    pub pos: Vec3
}

#[derive(Component)]
pub struct Target {
    pub pos: Vec3
}

pub enum TurretType {
    Projectile(ProjectileEnum),
    Beam,
    AreaOfEffect
}

pub enum ProjectileEnum {
    Regular { damage: f32, speed: f32 },
    Decaying { damage: f32, speed: f32, life_time: Duration },
    Explosive { damage: f32, speed: f32, explosion_range: f32, explosion_damage: f32 },
    Homing { damage: f32, speed: f32, homing_angle: f32 }
}
