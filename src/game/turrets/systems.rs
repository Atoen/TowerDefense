use ::core::f32;

use bevy_prng::WyRand;
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

const ROTATION_EPSILON: f32 = 0.001;
const DEFAULT_ROTATION_SPEED: f32 = std::f32::consts::PI;

pub fn idle_rotation_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut IdleRotation, &mut TargetingTurret, Option<&RotationSpeed>)>,
    mut rng: ResMut<GlobalEntropy<WyRand>>
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

            let random = rng.next_u32();
            let random_rotation = map_u32_to_range(random, f32::consts::FRAC_PI_3, f32::consts::PI);

            let random_angle = if random % 2 == 0 { random_rotation } else { -random_rotation };

            idle_rotation.target_angle = current_angle + random_angle;
            continue;
        }

        let angle_diff = shortest_angle_diff(current_angle, idle_rotation.target_angle);
        
        if angle_diff.abs() >= f32::consts::PI {
            warn!("Angle diff: {}", angle_diff);
        }
        if angle_diff.abs() < ROTATION_EPSILON {
            continue;
        }

        let rotation_speed = rotation_speed.map_or(DEFAULT_ROTATION_SPEED, |rs| rs.0);

        let rotation_step = rotation_speed * time.delta_seconds() * angle_diff.signum();
        let new_angle = turret.current_angle + smaller_magnitude(rotation_step, angle_diff);

        transform.rotation = Quat::from_rotation_z(new_angle);
        turret.current_angle = new_angle;
    }
}

fn get_target_weight(alien: &Alien, targeting_mode: &TargetingMode) -> f32 {
    match targeting_mode {
        TargetingMode::First => alien.path_progress,
        TargetingMode::Last => -alien.path_progress,
        TargetingMode::Strongest => alien.max_health,
    }
}

pub fn turret_targeting_system(
    time: Res<Time>,
    mut turrets: Query<(&mut TargetingTurret, &mut Transform, &GlobalTransform, Option<&RotationSpeed>), Without<Alien>>,
    aliens: Query<(Entity, &GlobalTransform, &Alien)>
) {
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
            } else {
                turret.current_target = None;
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
            continue
        }

        let target_angle = displacement.y.atan2(displacement.x) - f32::consts::FRAC_PI_2;
        let angle_diff = shortest_angle_diff(turret.current_angle, target_angle);

        if angle_diff.abs() >= f32::consts::PI {
            warn!("Angle diff: {}", angle_diff);
        }

        if angle_diff.abs() < ROTATION_EPSILON {
            continue
        }

        let rotation_speed = rotation_speed.map_or(DEFAULT_ROTATION_SPEED, |rs| rs.0);

        let rotation_step = rotation_speed * time.delta_seconds() * angle_diff.signum();
        let new_angle = turret.current_angle + smaller_magnitude(rotation_step, angle_diff);

        turret_transform.rotation = Quat::from_rotation_z(new_angle);
        turret.current_angle = new_angle;
    }
}

pub fn projectile_turret_attack_system(
    mut commands: Commands,
    time: Res<Time>,
    game_textures: Res<GameTextures>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
    mut turrets: Query<(&TargetingTurret, &GlobalTransform, Option<&mut AttackDelay>, Option<&ProjectileSpawnOffset>, Option<&AttackDispersion>), With<ProjectileTurret>>,
) {
    for (
        turret,
        turret_transform,
        attack_delay,
        spawn_offset,
        attack_dispersion
    ) in &mut turrets {
        if turret.current_target.is_none() {
            continue
        }

        if let Some(mut attack_delay) = attack_delay {
            if !attack_delay.0.tick(time.delta()).finished() {
                continue
            }
        }

        let direction = match attack_dispersion {
            Some(dispersion) => {
                let random_angle = map_u32_to_range(rng.next_u32(), -dispersion.0, dispersion.0);
                turret.current_angle + random_angle
            }
            None => turret.current_angle
        };

        let rotation = Quat::from_rotation_z(direction);

        let offset = match spawn_offset {
            Some(offset) => {
                rotation * offset.0
            }
            None => Vec3::ZERO,
        };

        let spawn_translation = turret_transform.translation() + offset;

        commands.spawn((
            Projectile {
                radius: 1.0,
                damage: Damage {
                    kind: DamageKind::Instant(25.0),
                    source: DamageSource::Turret(Turret::PulseBlaster), 
                    damage_type: DamageType::Kinetic 
                },
                pierce: Pierce::ONE,
            },
            SpriteBundle {
                texture: game_textures.bullet.clone(),
                transform: Transform {
                    translation: spawn_translation,
                    rotation,
                    ..default()
                },
                ..default()
            },
            RenderLayers::layer(1)
        ));
    }
}