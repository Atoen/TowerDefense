use ::core::f32;
use std::time::Duration;

use bevy_prng::WyRand;
use bevy_rand::prelude::GlobalEntropy;
use bevy_tweening::{lens::{SpriteColorLens, TransformPositionLens}, Animator, EaseFunction, Tween};
use game::GameLayerOrder;
use rand::RngCore;

use crate::*;

pub fn upgrade_turret_system(
    mut commands: Commands,
    mut parent_query: Query<(Entity, &Children, &mut TurretLevel, &mut TextureAtlas, &Transform), With<RaiseTurretLevel>>,
    mut child_query: Query<(&mut TurretLevel, &Turret), Without<RaiseTurretLevel>>,
    game_textures: Res<GameTextures>
) {
    for (entity, chlidren, mut level, mut atlas, transform) in &mut parent_query {
        level.0 += 1;
        atlas.index += 1;

        let Some(child) = chlidren.first() else { continue };
        let Ok((mut child_level, turret)) = child_query.get_mut(*child) else {
            warn!("Missing turret child entity!");
            return;
        };

        child_level.0 += 1;

        commands.trigger(TurretUpgradedEvent {
            turret: *turret,
            level: level.0
        });
        
        commands.entity(entity).remove::<RaiseTurretLevel>();

        let start_pos = transform.translation.on(GameLayer::UPGRADE_ARROW) + Vec3::new(0.0, -25.0, 0.0);
        let end_pos = start_pos + Vec3::new(0.0, 75.0, 0.0);

        let translation_tween = Tween::new(
            EaseFunction::QuadraticOut,
            Duration::from_secs(3),
            TransformPositionLens {
                start: start_pos,
                end: end_pos
            }
        );

        let alpha_tween = Tween::new(
            EaseFunction::QuadraticOut,
            Duration::from_secs(3),
            SpriteColorLens {
                start: Color::srgb(201./255., 238./255., 252./255.),
                end: Color::NONE
            }
        );

        commands.spawn((
            DespawnAfter(Timer::from_seconds(3.0, TimerMode::Once)),
            Animator::new(translation_tween),
            Animator::new(alpha_tween),
            SpriteBundle {
                texture: game_textures.upgrade_arrow.clone(),
                transform: Transform {
                    translation: start_pos,
                    scale: Vec3::splat(0.5),
                    ..default()
                },
                ..default()
            },
            RenderLayers::layer(1)
        ));
    }
}

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
    mut turrets: Query<(&mut TargetingTurret, &mut Transform, &GlobalTransform, &TurretLevel, &Turret, Option<&RotationSpeed>), Without<Alien>>,
    aliens: Query<(Entity, &GlobalTransform, &Alien)>
) {
    for (
        mut turret,
        mut turret_transform,
        turret_global_transform,
        turret_level,
        turret_type,
        rotation_speed
    ) in &mut turrets {

        let turret_radius_2 = get_turret_range_squared(*turret_type, turret_level.0);
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