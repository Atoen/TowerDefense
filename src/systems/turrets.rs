use std::time::Duration;

use bevy::{math::{Vec2, Vec3, VectorSpace}, prelude::*, window::{PrimaryWindow, Window}};
use bevy_prng::{ChaCha8Rng, WyRand};
use bevy_rand::prelude::GlobalEntropy;
use rand_core::RngCore;

use crate::{components::turrets::*, GameTextures, WinSize, ARROW_SIZE};

pub fn move_target(
    mut target_query: Query<(&mut Transform, &mut Target)>,
    windows: Query<&Window, With<PrimaryWindow>>
) {
    let (mut transform, mut target) = target_query.single_mut();
    let window = windows.single();

    if let Some(pos) = window.cursor_position() {
        let pos = window_to_world_coords(pos, window.size());

        target.pos = pos;
        transform.translation.x = pos.x;
        transform.translation.y = pos.y; 
    }
}

fn window_to_world_coords(cursor_pos: Vec2, window_size: Vec2) -> Vec3 {
    Vec3 { 
        x: cursor_pos.x - window_size.x / 2.0,
        y: window_size.y / 2.0 - cursor_pos.y,
        z: 0.0
    }
}

const PROJECTILE_SPEED: f32 = 300.0;
const TARGET_RADIUS: f32 = 20.0;
const DESPAWN_MARGIN: f32 = 200.0;

pub fn projectile_system(
    mut commands: Commands,
    time: Res<Time>,
    win_size: Res<WinSize>,
    mut projectiles: Query<(Entity, &mut Transform, &Velocity, &Projectile), Without<Target>>,
    targets: Query<&Transform, With<Target>>,
) {
    let delta = time.delta_seconds();
    let target_transform = targets.single();

    for (
        entity,
        mut transform,
        velocity,
        projectile
    ) in &mut projectiles {
        let translation = &mut transform.translation;

        translation.x += velocity.x * delta * PROJECTILE_SPEED;
        translation.y += velocity.y * delta * PROJECTILE_SPEED;

        let distance = translation.distance(target_transform.translation);
        if distance < projectile.radius + TARGET_RADIUS {
            commands.entity(entity).despawn();
        }

        if projectile.auto_despawn {
            if translation.y > win_size.height / 2. + DESPAWN_MARGIN
                || translation.y < -win_size.height / 2. - DESPAWN_MARGIN
                || translation.x > win_size.width / 2. + DESPAWN_MARGIN
                || translation.x < -win_size.width / 2. - DESPAWN_MARGIN
            {
                commands.entity(entity).despawn();
            }
        }
    }
}

pub fn flag_idle_turrets(
    time: Res<Time>,
    mut query: Query<(&mut IdleRotation, &TargetingTurret)>
) {
    for (mut idle_rotation, turret) in &mut query {
        if turret.has_target {
            idle_rotation.idle_timer.reset();
        } else {
            idle_rotation.idle_timer.tick(time.delta());
        }

        if idle_rotation.idle_timer.just_finished() {
            idle_rotation.target_angle = turret.rotation;
        }

        idle_rotation.is_idle = idle_rotation.idle_timer.finished();
    }
}

const ROTATION_EPSILON: f32 = 0.01;
const DEFAULT_ROTATION_SPEED: f32 = std::f32::consts::PI;
const MAX_RANDOM_ROTATION_ANGLE: f32 = std::f32::consts::FRAC_PI_2;

pub fn idle_rotation_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut IdleRotation, Option<&RotationSpeed>)>,
    mut rng: ResMut<GlobalEntropy<ChaCha8Rng>>
) {
    for (
        mut transform,
        mut idle_rotation,
        rotation_speed
    ) in &mut query {
        if !idle_rotation.is_idle {
            continue;
        }

        let current_angle = transform.rotation.to_euler(EulerRot::XYZ).2;

        let timer = &mut idle_rotation.rotation_timer;
        timer.tick(time.delta());
        if timer.finished() {
            let random_rotation = rng.next_u32();
            let random_angle = (random_rotation as f32 / u32::MAX as f32) * MAX_RANDOM_ROTATION_ANGLE - MAX_RANDOM_ROTATION_ANGLE / 2.;

            idle_rotation.target_angle = current_angle + random_angle;
            continue;
        }

        let angle_diff = shortest_angle_diff(current_angle, idle_rotation.target_angle);
        if angle_diff.abs() < ROTATION_EPSILON {
            continue;
        }

        let rotation_speed = rotation_speed.map_or(DEFAULT_ROTATION_SPEED, |rs| rs.0) / 2.;
        let rotation_step = rotation_speed * time.delta_seconds();

        let new_angle = current_angle + rotation_step * angle_diff.signum();
        transform.rotation = Quat::from_rotation_z(new_angle);
    }
}

fn smaller_magnitude(a: f32, b: f32) -> f32 {
    if a.abs() < b.abs() {
        a
    } else {
        b
    }
}

fn shortest_angle_diff(from: f32, to: f32) -> f32 {
    let diff = (to - from).rem_euclid(2.0 * std::f32::consts::PI);
    if diff > std::f32::consts::PI {
        diff - 2.0 * std::f32::consts::PI
    } else {
        diff
    }
}

pub fn targeting_turret_system(
    time: Res<Time>,
    mut turrets: Query<(&mut TargetingTurret, &mut Transform, Option<&RotationSpeed>)>,
    targets: Query<&Target>
) {
    let target_position = targets.single().pos;

    for (
        mut turret,
        mut turret_transform,
        rotation_speed
    ) in &mut turrets {
        let target_distance = (target_position - turret_transform.translation).truncate();
        let is_inside_radius = match turret.targeting_radius {
            Some(radius) => radius >= target_distance.length(),
            None => true
        };

        turret.has_target = is_inside_radius;
        if !is_inside_radius {
            continue;
        }

        let target_angle = target_distance.y.atan2(target_distance.x) - std::f32::consts::FRAC_PI_2;
        let current_angle = turret_transform.rotation.to_euler(EulerRot::XYZ).2;
        let angle_diff = shortest_angle_diff(current_angle, target_angle);

        if angle_diff.abs() < ROTATION_EPSILON {
            continue;
        }

        let rotation_speed = match rotation_speed {
            Some(speed) => speed.0,
            None => DEFAULT_ROTATION_SPEED
        };

        let rotation_step = smaller_magnitude(rotation_speed * time.delta_seconds(), angle_diff);
        let new_angle = current_angle + rotation_step * angle_diff.signum();

        turret_transform.rotation = Quat::from_rotation_z(new_angle);
        turret.rotation = new_angle;
    }
}

pub fn projectile_turret_attack_system(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    time: Res<Time>,
    mut turrets: Query<(&TargetingTurret, &Transform, Option<&mut AttackDelay>, Option<&SpawnOffset>, Option<&AttackDispersion>), With<ProjectileTurret>>,
    mut rng: ResMut<GlobalEntropy<ChaCha8Rng>>
) {
    for (
        turret,
        turret_transform,
        attack_delay,
        spawn_offset,
        attack_dispersion
    ) in &mut turrets {
        if !turret.has_target {
            continue;
        }

        if let Some(mut attack_delay) = attack_delay {
            attack_delay.0.tick(time.delta());
            if !attack_delay.0.finished() {
                continue;
            }
        }

        let direction = match attack_dispersion {
            Some(dispersion) => {
                let random_angle = map_u32_to_range(rng.next_u32(), -dispersion.0, dispersion.0);
                let rotation = Quat::from_rotation_z(random_angle);
                rotation * turret_transform.rotation * Vec3::Y
            },
            None => turret_transform.rotation * Vec3::Y
        };

        let spawn_translation = turret_transform.translation + turret_transform.rotation * spawn_offset.map_or(Vec3::ZERO, |off| off.0);

        commands.spawn((
            Projectile {
                auto_despawn: true,
                radius: 1.
            },
            InstantDamage(1.),
            Velocity {
                x: direction.x,
                y: direction.y
            },
            SpriteBundle {
                texture: game_textures.bullet.clone(),
                transform: Transform {
                    translation: spawn_translation,
                    rotation: turret_transform.rotation,
                    ..default()
                },
                ..default()
            }
        ));
    }
}

fn map_u32_to_range(value: u32, min: f32, max: f32) -> f32 {
    let normalized = value as f32 / u32::MAX as f32;
    min + normalized * (max - min)
}

pub fn spawn_projectile_turret(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    windows: Query<&Window, With<PrimaryWindow>>
) {
    let window = windows.single();
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let pos = window_to_world_coords(cursor_pos, window.size());

    commands.spawn((
        Turret,
        ProjectileTurret,
        AttackDispersion(std::f32::consts::PI / 16.),
        TargetingTurret {
            targeting_radius: Some(200.),
            rotation: 0.,
            has_target: false
        },
        SpawnOffset(Vec3 { x: 0., y: ARROW_SIZE.1 / 2., z: 0. }),
        RotationSpeed(std::f32::consts::PI * 2.),
        AttackDelay(Timer::new(Duration::from_millis(200), TimerMode::Repeating)),
        IdleRotation {
            idle_timer: Timer::new(Duration::from_secs(3), TimerMode::Once),
            rotation_timer: Timer::new(Duration::from_secs(2), TimerMode::Repeating),
            target_angle: 0.,
            is_idle: false
        },
        SpriteBundle {
            texture: game_textures.arrow.clone(),
            transform: Transform {
                translation: pos,
                ..default()
            },
            ..default()
        }
    ));
}

// pub fn laser_turret_attack_system(
//     mut commands: Commands,
//     game_textures: Res<GameTextures>,
//     target_query: Query<&Target>,
//     mut laser_turrets: Query<(&Turret, &mut LaserTurret, &Transform), Without<LaserBeam>>,
//     mut laser_beams: Query<&mut Transform, With<LaserBeam>>
// ) {
//     let target = target_query.single();

//     for (turret, mut laser_turret, turret_transform) in &mut laser_turrets {
//         if !turret.attacking {
//             let beam = laser_turret.laser_beam.take();
//             if let Some(beam) = beam {
//                 commands.entity(beam).despawn();
//             }

//             continue;
//         }

//         let target_position = target.pos;
//         let direction = (target_position - turret_transform.translation).normalize();
//         let angle = f32::atan2(direction.y, direction.x);
        
//         let target_angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;
//         let current_angle = turret_transform.rotation.to_euler(EulerRot::XYZ).2;
//         let angle_diff = shortest_angle_diff(current_angle, target_angle);

//         if angle_diff.abs() > ROTATION_EPSILON * 2.0 {
//             let beam = laser_turret.laser_beam.take();
//             if let Some(beam) = beam {
//                 commands.entity(beam).despawn();
//             }

//             continue;
//         }
        
//         let beam_start = turret_transform.translation + turret_transform.rotation * turret.spawn_offset;
        
//         let distance = beam_start.distance(target_position);
//         let rotation = Quat::from_rotation_z(angle - PI / 2.0);

//         let midpoint = beam_start + direction * distance / 2.0;    
//         if let Some(exising_beam) = laser_turret.laser_beam {
//             if let Ok(mut laser_transform) = laser_beams.get_mut(exising_beam) {
//                 laser_transform.translation = midpoint;
//                 laser_transform.rotation = rotation;
//                 laser_transform.scale = Vec3::new(1.0, distance / 12.0, 1.0);
//             }
//         } else {
//             let id = commands.spawn(SpriteBundle {
//                 texture: game_textures.laser_beam.clone(),
//                 transform: Transform {
//                     translation: midpoint,
//                     rotation,
//                     scale: Vec3::new(1.0, distance / 12.0, 1.0),
//                     ..default()
//                 },
//                 ..default()
//             })
//             .insert(LaserBeam { damage_per_second: 10. }).id();
    
//             laser_turret.laser_beam = Some(id);
//         }
//     }
// }
