use bevy::prelude::*;

use crate::{components::turrets::*, TurretType, ARROW_SIZE};


#[derive(Bundle)]
pub struct PulseBlasterBundle {
    pub marker: Turret,
    pub type_marker: ProjectileTurret,
    pub attack_dispersion: AttackDispersion,
    pub targeting: TargetingTurret,
    pub projectile_spawn_offset: SpawnOffset,
    pub rotation_speed: RotationSpeed,
    pub attack_delay: AttackDelay,
    pub idle_rotation: IdleRotation,
    pub sprite: SpriteBundle 
}

impl Default for PulseBlasterBundle {
    fn default() -> Self {
        Self {
            marker: Turret(TurretType::PulseBlaster),
            type_marker: ProjectileTurret,
            attack_dispersion: AttackDispersion(std::f32::consts::PI / 16.),
            targeting: TargetingTurret {
                targeting_radius: Some(200.),
                rotation: 0.,
                has_target: false
            },
            projectile_spawn_offset: SpawnOffset(Vec3 { x: 0., y: ARROW_SIZE.1 / 2., z: 0. }),
            rotation_speed: RotationSpeed(std::f32::consts::FRAC_PI_2),
            attack_delay: AttackDelay(Timer::from_seconds(0.2, TimerMode::Repeating)),
            idle_rotation: IdleRotation {
                ..default()
            },
            sprite: SpriteBundle {
                ..default()
            }
        }
    }
}
