use bevy::{ecs::entity, prelude::*, transform::commands, window::PrimaryWindow};

use crate::{components::{AttackDelay, Movable, ProjectileTurret, RotationSpeed, Target, TargetingRadius, Turret, Velocity}, GameTextures, WinSize};

pub fn move_target(
    mut target_query: Query<&mut Transform, With<Target>>,
    windows: Query<&Window, With<PrimaryWindow>>
) {
    let mut target = target_query.single_mut();

    if let Some(pos) = windows.single().cursor_position() {
        let pos = Vec2::new(
            pos.x - windows.single().width() / 2.0,
            windows.single().height() / 2.0 - pos.y,
        );
        target.translation.x = pos.x;
        target.translation.y = pos.y; 
    }
}

const PROJECTILE_SPEED: f32 = 200.0;
const MOVABLE_RADIUS: f32 = 1.0;
const TARGET_RADIUS: f32 = 20.0;

pub fn movable_system(
    mut commands: Commands,
    time: Res<Time>,
    win_size: Res<WinSize>,
    mut movable_query: Query<(Entity, &mut Transform, &Velocity, &Movable), Without<Target>>,
    mut target_query: Query<&Transform, With<Target>>,
) {
    let delta = time.delta_seconds();
    let target_transform = target_query.single_mut();

    for (entity, mut transform, velocity, movable) in &mut movable_query.iter_mut() {
        let translation = &mut transform.translation;

        translation.x += velocity.x * delta * PROJECTILE_SPEED;
        translation.y += velocity.y * delta * PROJECTILE_SPEED;

        let distance = translation.distance(target_transform.translation);
        if distance < MOVABLE_RADIUS + TARGET_RADIUS {
            commands.entity(entity).despawn();
        }

        if movable.auto_despawn {
            const MARGIN: f32 = 200.0;
            if translation.y > win_size.height / 2. + MARGIN
                || translation.y < -win_size.height / 2. - MARGIN
                || translation.x > win_size.width / 2. + MARGIN
                || translation.x < -win_size.width / 2. - MARGIN
            {
                commands.entity(entity).despawn();
            }
        }
    }
}

const ROTATION_EPSILON: f32 = 0.01;
const DEFAULT_ROTATION_SPEED: f32 = std::f32::consts::PI;

pub fn rotate_turrets(
    mut turret_query: Query<(&mut Turret, &mut Transform, Option<&TargetingRadius>, Option<&RotationSpeed>)>,
    target_query: Query<&Transform, (With<Target>, Without<Turret>)>,
    time: Res<Time>,
) {
    let target_transform = target_query.single();
    let target_position = target_transform.translation.truncate();

    
    for (mut turret, mut turret_transform, targeting_radius, rotation_speed) in &mut turret_query.iter_mut() {
        let direction = target_position - turret_transform.translation.truncate();

        let is_inside_radius = match targeting_radius {
            Some(val) => val.0 >= direction.length(),
            None => true
        };

        turret.attacking = is_inside_radius;

        if !is_inside_radius {
            continue;
        }

        let target_angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;
        let current_angle = turret_transform.rotation.to_euler(EulerRot::XYZ).2;
        let angle_diff = shortest_angle_diff(current_angle, target_angle);

        if angle_diff.abs() < ROTATION_EPSILON {
            continue;
        }

        let rotation_speed = match rotation_speed {
            Some(speed) => speed.0,
            None => DEFAULT_ROTATION_SPEED
        };

        let rotation_step = rotation_speed * time.delta_seconds();

        let new_angle = current_angle + rotation_step * angle_diff.signum();
        turret_transform.rotation = Quat::from_rotation_z(new_angle);
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

pub fn projectile_turret_attack_system(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    time: Res<Time>,
    mut attack_query: Query<(&Turret, &ProjectileTurret, &Transform, Option<&mut AttackDelay>), With<Turret>>
) {
    for (turret, projectile_turret, transform, attack_delay) in &mut attack_query.iter_mut() {

        if !turret.attacking {
            continue;
        }

        if let Some(mut attack_delay) = attack_delay {
            attack_delay.timer.tick(time.delta());

            if !attack_delay.timer.finished() {
                continue;
            }
        }
    
        let direction = transform.rotation * Vec3::Y;
        let velocity = Velocity { x: direction.x, y: direction.y };

        let spawn_offset = transform.translation + transform.rotation * projectile_turret.projectile_spawn_offset;
        let spawn_translation = spawn_offset;

        commands.spawn(SpriteBundle {
            texture: game_textures.bullet.clone(),
            transform: Transform {
                translation: spawn_translation,
                rotation: transform.rotation,
                ..default()
            },
            ..default()
        })
            .insert(Movable { auto_despawn: true })
            .insert(velocity);
    
    }
}