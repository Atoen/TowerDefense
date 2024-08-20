use ::core::f32;

use bevy_rand::prelude::GlobalEntropy;
use rand::RngCore;

use crate::*;

pub fn flag_idle_turrets(
    time: Res<Time>,
    mut query: Query<(&mut IdleRotation, &TargetingTurret)>
) {
    for (mut idle_rotation, turret) in &mut query {
        if turret.current_target.is_some() {
            idle_rotation.idle_timer.reset();
        } else {
            idle_rotation.idle_timer.tick(time.delta());
        }

        if idle_rotation.idle_timer.just_finished() {
            idle_rotation.target_angle = turret.current_angle;
        }

        idle_rotation.is_idle = idle_rotation.idle_timer.finished();
    }
}

const ROTATION_EPSILON: f32 = 0.0001;
const DEFAULT_ROTATION_SPEED: f32 = std::f32::consts::PI;
const MAX_RANDOM_ROTATION_ANGLE: f32 = std::f32::consts::FRAC_PI_2;

pub fn idle_rotation_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut IdleRotation, &mut TargetingTurret, Option<&RotationSpeed>)>,
    mut rng: ResMut<GlobalEntropy<ChaCha8Rng>>
) {
    for (
        mut transform,
        mut idle_rotation,
        mut turret,
        rotation_speed
    ) in &mut query {
        if !idle_rotation.is_idle {
            continue;
        }

        let current_angle = transform.rotation.to_euler(EulerRot::XYZ).2;

        let timer = &mut idle_rotation.rotation_timer;
        if timer.tick(time.delta()).finished() {
            let random_rotation = rng.next_u32();
            let random_angle = (random_rotation as f32 / u32::MAX as f32) * MAX_RANDOM_ROTATION_ANGLE - MAX_RANDOM_ROTATION_ANGLE / 2.;

            idle_rotation.target_angle = current_angle + random_angle;
            continue;
        }

        let angle_diff = shortest_angle_diff(current_angle, idle_rotation.target_angle);
        if angle_diff.abs() < ROTATION_EPSILON {
            continue;
        }

        let rotation_speed = rotation_speed.map_or(DEFAULT_ROTATION_SPEED, |rs| rs.0);

        let rotation_step = rotation_speed * time.delta_seconds();
        let new_angle = turret.current_angle + smaller_magnitude(rotation_step, angle_diff);

        transform.rotation = Quat::from_rotation_z(new_angle);
        turret.current_angle = new_angle;
    }
}

pub fn turret_targeting_system(
    time: Res<Time>,
    mut turrets: Query<(&mut TargetingTurret, &mut Transform, &GlobalTransform, Option<&RotationSpeed>), Without<Alien>>,
    aliens: Query<(Entity, &GlobalTransform, &Alien)>
) {
    fn get_target_weight(alien: &Alien, targeting_mode: &TargetingMode) -> f32 {
        match targeting_mode {
            TargetingMode::First => alien.path_progress,
            TargetingMode::Last => -alien.path_progress,
            TargetingMode::Strongest => alien.max_health,
        }
    }

    for (
        mut turret,
        mut turret_transform,
        turret_global_transform,
        rotation_speed
    ) in &mut turrets {

        let turret_radius_2 = turret.targeting_radius * turret.targeting_radius;
        let mut displacement = Vec2::ZERO; 

        // Does turret already have a target
        if let Some(target) = turret.current_target {
            if let Ok((_, target_transform, _)) = aliens.get(target) {

                displacement = (target_transform.translation() - turret_global_transform.translation()).truncate();

                // Current target moved out of range
                if displacement.length_squared() > turret_radius_2 {
                    turret.current_target = None;
                }
            }
        }

        if turret.current_target.is_none() {
            let (
                mut target_weight,
                mut target_displacement,
                mut target_entity
            ) = (f32::MIN, Vec2::ZERO, None::<Entity>);
    
            for (
                alien_entity,
                alien_transform,
                alien
            ) in &aliens {
        
                let alien_displacement = (alien_transform.translation() - turret_global_transform.translation()).truncate();
                let distance_squared = alien_displacement.length_squared();
        
                if distance_squared <= turret_radius_2 {
                    let weight = get_target_weight(alien, &turret.mode);
                    if weight > target_weight {
                        target_weight = weight;
                        target_displacement = alien_displacement;
                        target_entity = Some(alien_entity);
                    }
                }
            }
        
            if target_entity.is_some() {
                turret.current_target = target_entity;
                displacement = target_displacement;
            }
        }

        if turret.current_target.is_none() { 
            return
        }

        let target_angle = displacement.y.atan2(displacement.x) - f32::consts::FRAC_PI_2;
        let angle_diff = shortest_angle_diff(turret.current_angle, target_angle);

        if angle_diff.abs() < ROTATION_EPSILON {
            continue;
        }

        let rotation_speed = rotation_speed.map_or(DEFAULT_ROTATION_SPEED, |rs| rs.0);

        let rotation_step = rotation_speed * time.delta_seconds();
        let new_angle = turret.current_angle + smaller_magnitude(rotation_step, angle_diff);

        turret_transform.rotation = Quat::from_rotation_z(new_angle);
        turret.current_angle = new_angle;
    }
}
