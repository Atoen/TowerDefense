use bevy::{math::Vec3, prelude::Component, time::Timer};


#[derive(Component)]
pub struct AttackDelay {
    pub timer: Timer
}

#[derive(Component)]
pub struct Movable {
    pub auto_despawn: bool
}

#[derive(Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct RotationSpeed(pub f32);

#[derive(Component)]
pub struct TargetingRadius(pub f32);

#[derive(Component)]
pub struct Arrow {
    pub attacking: bool
}


#[derive(Component)]
pub struct Turret {
    pub attacking: bool
}

#[derive(Component)]
pub struct ProjectileTurret {
    pub projectile_spawn_offset: Vec3
}

#[derive(Component)]
pub struct Target;

#[derive(Component)]
pub struct HitTimer {
    pub timer: Timer
}