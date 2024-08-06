
use bevy::{math::Vec3, prelude::*, sprite::{MaterialMesh2dBundle, Mesh2dHandle}, window::{PrimaryWindow, Window}};
use bevy_prng::ChaCha8Rng;
use bevy_rand::prelude::GlobalEntropy;
use rand_core::RngCore;

use crate::{components::turrets::*, turret_bundles::PulseBlasterBundle, GameTextures, WinSize};

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

pub fn window_to_world_coords(cursor_pos: Vec2, window_size: Vec2) -> Vec3 {
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
    mut projectiles: Query<(Entity, &mut Transform, &Projectile, Option<&LinearVelocity>, Option<&Explosive>), Without<Target>>,
    targets: Query<&Target>,
) {
    let delta = time.delta_seconds();
    let target = targets.single();

    for (
        entity,
        mut transform,
        projectile,
        velocity,
        explosive
    ) in &mut projectiles {
        let speed = velocity.map_or(PROJECTILE_SPEED, |v| v.0);
        let current_angle = transform.rotation.to_euler(EulerRot::XYZ).2 + std::f32::consts::FRAC_PI_2;

        let velocity_vec = Vec3::new(current_angle.cos() * speed, current_angle.sin() * speed, 0.0);
        transform.translation += velocity_vec * delta;

        let distance = transform.translation.distance(target.pos);

        if distance < projectile.radius + TARGET_RADIUS {
            commands.entity(entity).despawn();
            if let Some(explosive) = explosive {
                commands.spawn(
                    ExplosionToSpawn {
                        damage: explosive.damage,
                        radius: explosive.radius,
                        pos: transform.translation
                    }
                );
            }

            continue;
        }

        if projectile.auto_despawn {
            if transform.translation.y > win_size.height / 2.0 + DESPAWN_MARGIN
                || transform.translation.y < -win_size.height / 2.0 - DESPAWN_MARGIN
                || transform.translation.x > win_size.width / 2.0 + DESPAWN_MARGIN
                || transform.translation.x < -win_size.width / 2.0 - DESPAWN_MARGIN
            {
                commands.entity(entity).despawn();
            }
        }
    }
}

pub fn decaying_projectile_system(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    mut projectiles: Query<(Entity, &mut Decaying, AnyOf<(&mut Sprite, &Handle<ColorMaterial>)>), With<Projectile>>
) {
    for (
        entity,
        mut decaying,
        sprite_or_material
    ) in &mut projectiles {
        decaying.decay_timer.tick(time.delta());

        if decaying.decay_type == DecayType::Transparency {
            let alpha = decaying.decay_timer.fraction_remaining();
            
            match sprite_or_material {
                (Some(mut sprite), None) => sprite.color.set_alpha(alpha),
                (None, Some(handle)) => {
                    if let Some(material) = materials.get_mut(handle) {
                        material.color.set_alpha(alpha)
                    }
                },
                _ => ()
            }
        }

        if decaying.decay_timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn homing_projectile_system(
    time: Res<Time>,
    mut projectiles: Query<(&Homing, &mut Transform), With<Projectile>>,
    targets: Query<&Target>
) {
    let target = targets.single();

    for (homing, mut transform) in &mut projectiles {
        let distance = (target.pos - transform.translation).truncate();
        if distance.length() > homing.homing_distance {
            continue;
        }

        let target_angle = distance.y.atan2(distance.x) - std::f32::consts::FRAC_PI_2;
        let current_angle = transform.rotation.to_euler(EulerRot::XYZ).2;
        let angle_diff = shortest_angle_diff(current_angle, target_angle);

        let abs = angle_diff.abs();
        if abs > homing.homing_angle || abs < ROTATION_EPSILON {
            continue;
        }

        let rotation_step = smaller_magnitude(homing.homing_speed * time.delta_seconds(), angle_diff);
        let new_angle = current_angle + rotation_step * angle_diff.signum();

        transform.rotation = Quat::from_rotation_z(new_angle);
    }
}

pub fn explosion_spawn_system(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
	query: Query<(Entity, &ExplosionToSpawn)>,
) {
    for (entity, explosion_to_spawn) in &query {
		commands
			.spawn((
                Explosion {
                    radius: explosion_to_spawn.radius,
                    pos: explosion_to_spawn.pos
                },
                AoEAnimation {
                    timer: Timer::from_seconds(1., TimerMode::Once),
                    despawn_on_end: true,
                    radius_animation: Some(RadiusAnimation::FromBaseRadius { grow_speed: 1.}),
                    color_animation: Some(ColorAnimation {
                        start_color: Color::srgb(1.0, 1.0, 0.5),
                        end_color: Color::srgb(1.0, 0.5, 0.0),
                        alpha_factor: None,
                        animate_alpha: true
                    })
                },
                MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Circle {radius: explosion_to_spawn.radius })),
                    material: materials.add(Color::srgb(1.0, 1.0, 0.0)),
                    transform: Transform {
                        translation: explosion_to_spawn.pos.with_z(100.),
                        ..default()
                    },
                    ..default()
                }
			));

		commands.entity(entity).despawn();
	}
}


pub fn aoe_animation_system(
	mut commands: Commands,
	time: Res<Time>,
    mut materials: ResMut<Assets<ColorMaterial>>,
	mut animations: Query<(Entity, &mut AoEAnimation, &mut Transform, &Handle<ColorMaterial>)>
) {
    for (
        entity,
        mut animation,
        mut animation_transform,
        handle
    ) in &mut animations {
        animation.timer.tick(time.delta());
        let t = animation.timer.fraction();
        let t_1 = animation.timer.fraction_remaining();

        if let Some(radius_animation) = &animation.radius_animation {
            match radius_animation {
                RadiusAnimation::FromBaseRadius { grow_speed } => {
                    let scale = animation_transform.scale.x + grow_speed * time.delta_seconds();
                    animation_transform.scale = Vec3::splat(scale);
                },
                RadiusAnimation::FromStartToEnd { start_radius, end_radius } => {
                    let radius = start_radius.lerp(*end_radius, t);
                    animation_transform.scale = Vec3::splat(radius);
                },
            }
        }

        if let Some(color_animation) = &animation.color_animation {
            if let Some(material) = materials.get_mut(handle) {
                let start = color_animation.start_color.to_srgba();
                let end = color_animation.end_color.to_srgba();

                let color = Color::srgba(
                    start.red * t_1 + end.red * t,
                    start.green * t_1 + end.green * t,
                    start.blue * t_1 + end.blue * t,
                    if color_animation.animate_alpha {
                        color_animation.alpha_factor.unwrap_or(1.0) * t_1
                    } else {
                        material.color.alpha()
                    }
                );

                material.color = color;
            }
        }   

        if animation.despawn_on_end && animation.timer.finished() {
            commands.entity(entity).despawn();
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

pub fn turret_targeting_system(
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
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
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
                rotation * turret_transform.rotation
            },
            None => turret_transform.rotation
        };

        let offset = turret_transform.rotation * spawn_offset.map_or(Vec3::ZERO, |off| off.0);
        let spawn_translation = turret_transform.translation + offset;

        commands.spawn((
            Projectile {
                auto_despawn: true,
                radius: 1.
            },
            // Decaying {
            //     decay_timer: Timer::from_seconds(1., TimerMode::Once),
            //     decay_type: DecayType::Despawn
            // },
            // AoEAnimation {
            //     timer: Timer::from_seconds(1., TimerMode::Once),
            //     radius_animation: Some(RadiusAnimation::FromStartToEnd { start_radius: 2., end_radius: 20. }),
            //     color_animation: Some(ColorAnimation {
            //         start_color: Color::srgb(1., 0.5, 0.),
            //         end_color: Color::srgb(1., 0., 0.1),
            //         alpha_factor: None,
            //         animate_alpha: true
            //     }),
            //     despawn_on_end: false
            // },
            // Homing {
            //     homing_angle: std::f32::consts::PI,
            //     homing_distance: 500.,
            //     homing_speed: 2.
            // },
            // Explosive {
            //     damage: 5.,
            //     radius: 20.
            // },
            InstantDamage(1.),
            LinearVelocity(200.),
            // MaterialMesh2dBundle {
            //     mesh: Mesh2dHandle(meshes.add(Circle { radius: 2. })),
            //     material: materials.add(Color::srgb(0.64, 0.12, 0.36)),
            //     transform: Transform {
            //         translation: spawn_translation.with_z(99.),
            //         rotation: direction,
            //         ..default()
            //     },
            //     ..default()
            // }
            SpriteBundle {
                texture: game_textures.bullet.clone(),
                transform: Transform {
                    translation: spawn_translation,
                    rotation: direction,
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

pub fn aoe_turret_attack_system(
    time: Res<Time>,
    mut commands: Commands,
    mut turrets: Query<(&AoETurret, &Transform, Option<&mut AttackDelay>), With<Turret>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    targets: Query<&Target>
) {
    let target = targets.single();
    for (turret, transform, attack_delay) in &mut turrets {
        if let Some(mut attack_delay) = attack_delay {
            attack_delay.0.tick(time.delta());
            if !attack_delay.0.finished() {
                continue;
            }
        }
        
        if !turret.always_attacking {
            let target_distance = (target.pos - transform.translation).truncate();
            if turret.range <= target_distance.length() {
                continue;
            }
        }

        commands.spawn((
            AoEAttack {
                pos: transform.translation,
                radius: turret.range
            },
            AoEAnimation {
                timer: Timer::from_seconds(0.5, TimerMode::Once),
                despawn_on_end: true,
                radius_animation: Some(RadiusAnimation::FromStartToEnd { start_radius: 1., end_radius: turret.range }),
                color_animation: Some(ColorAnimation {
                    start_color: Color::srgb(0.64, 0.12, 0.36),
                    end_color: Color::srgb(0.48, 0.13, 0.64),
                    alpha_factor: None,
                    animate_alpha: true
                })
            },
            MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Annulus {
                    inner_circle: Circle { radius: 0.95 },
                    outer_circle: Circle { radius: 1. }
                })),
                material: materials.add(Color::srgb(0.64, 0.12, 0.36)),
                transform: Transform {
                    translation: transform.translation.with_z(99.),
                    ..default()
                },
                ..default()
            }
        ));
    }
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

    commands.spawn(PulseBlasterBundle {
        sprite: SpriteBundle {
            texture: game_textures.arrow.clone(),
            transform: Transform {
                translation: pos,
                ..default()
            },
            ..default()
        },
        ..default()
    });
}

pub fn spawn_aoe_turret(
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
        Turret(crate::TurretType::CryoGenerator),
        AoETurret {
            range: 100.,
            always_attacking: false
        },
        AttackDelay(Timer::from_seconds(1., TimerMode::Repeating)),
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
