pub mod components;
pub use components::*;

pub mod systems;
pub use systems::*;

use crate::*;

const DEFAULT_PROJECTILE_SPEED: f32 = 300.0;
const TARGET_RADIUS: f32 = 20.0;
const DESPAWN_MARGIN: f32 = 200.0;

#[derive(Event)]
pub struct TurretUpgradedEvent {
    pub entity: Entity,
    pub turret: Turret,
    pub level: u8
}

fn projectile_system(
    mut commands: Commands,
    time: Res<Time>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut projectiles: Query<
        (Entity, &mut Transform, &mut Projectile, Option<&LinearVelocity>, Option<&Explosive>),
        Without<Alien>>,
    mut aliens: Query<(Entity, &GlobalTransform, &mut Alien)>
) {
    let delta = time.delta_seconds();
    let Ok(window) = windows.get_single() else { return };

    let (window_width, window_hegiht) = (window.width(), window.height());

    'projectiles: for (
        projectile_entity,
        mut projectile_transform,
        mut projectile,
        velocity,
        explosive
    ) in &mut projectiles {

        // Despawn projectiles out of bounds
        if  projectile_transform.translation.y > window_hegiht / 2.0 + DESPAWN_MARGIN ||
            projectile_transform.translation.y < -window_hegiht / 2.0 - DESPAWN_MARGIN || 
            projectile_transform.translation.x > window_width / 2.0 + DESPAWN_MARGIN ||
            projectile_transform.translation.x < -window_width / 2.0 - DESPAWN_MARGIN {

            commands.entity(projectile_entity).despawn();
            continue;            
        }

        let speed = velocity.map_or(DEFAULT_PROJECTILE_SPEED, |v| v.0);
        let current_angle = projectile_transform.rotation.to_euler(EulerRot::XYZ).2 + std::f32::consts::FRAC_PI_2;
    
        let velocity_vec = Vec3::new(current_angle.cos() * speed, current_angle.sin() * speed, 0.0);
        projectile_transform.translation += velocity_vec * delta;

        for (
            alien_entity,
            alien_transform,
            mut alien
        ) in &mut aliens {

            if projectile.hit_targets.contains(&alien_entity) {
                continue;
            }

            let distance_2 = projectile_transform.translation.distance_squared(alien_transform.translation());

            if distance_2 < (projectile.radius + TARGET_RADIUS) * (projectile.radius + TARGET_RADIUS) {

                projectile.hit_targets.insert(alien_entity);

                alien.add_damage(&projectile.damage, projectile_entity);

                if let Some(explosive) = explosive {
                    commands.entity(projectile_entity).despawn();

                    commands.spawn(ExplosionToSpawn {
                        damage: Some(explosive.damage),
                        falloff: explosive.fallof,
                        radius: explosive.radius,
                        position: projectile_transform.translation,
                        animation: explosive.animation.clone()
                    });

                    continue 'projectiles;
                } 
                else if let Pierce::Finite(ref mut pierce) = projectile.pierce {
                    *pierce = pierce.saturating_sub(1);

                    if *pierce == 0 {
                        commands.entity(projectile_entity).despawn();
                        continue 'projectiles;
                    }
                }
            }
        }
    }
}

pub fn homing_projectile_system(
    time: Res<Time>,
    mut projectiles: Query<(&mut Homing, &mut Transform), With<Projectile>>,
    mut aliens: Query<(Entity, &GlobalTransform), With<Alien>>
) {
    'projectiles: for (mut homing, mut projectile_transform) in &mut projectiles {

        let mut displacement = Vec2::ZERO;
        let homing_range_2 = homing.homing_distance * homing.homing_distance;
        
        if let Some(target) = homing.current_target {
            if let Ok((_, target_transform)) = aliens.get(target) {
                displacement = (target_transform.translation() - projectile_transform.translation).truncate();

                if displacement.length_squared() > homing_range_2 {
                    homing.current_target = None
                }
            } else {
                homing.current_target = None
            }
        }

        if homing.current_target.is_none() {

            let (
                mut target_weight,
                mut target_displacement,
                mut target_entity
            ) = (f32::MIN, Vec2::ZERO, None::<Entity>);

            for (alien_entity, alien_transform) in &mut aliens {
                let alien_displacement = (alien_transform.translation() - projectile_transform.translation).truncate();
                let distance_squared = alien_displacement.length_squared();

                if distance_squared <= homing_range_2 {
                    let weight = distance_squared;
                    if weight > target_weight {
                        target_weight = weight;
                        target_displacement = alien_displacement;
                        target_entity = Some(alien_entity);
                    }
                }
            }

            if target_entity.is_some() {
                homing.current_target = target_entity;
                displacement = target_displacement;
            }
        }

        if homing.current_target.is_none() {
            continue 'projectiles;
        }

        let target_angle = displacement.y.atan2(displacement.x) - std::f32::consts::FRAC_PI_2;
        let current_angle = projectile_transform.rotation.to_euler(EulerRot::XYZ).2;
        let angle_diff = shortest_angle_diff(current_angle, target_angle);
        
        let abs = angle_diff.abs();
        if abs > homing.homing_angle || abs < ROTATION_EPSILON {
            continue 'projectiles;
        }

        let rotation_step = homing.homing_speed * time.delta_seconds() * angle_diff.signum();
        let new_angle = current_angle + smaller_magnitude(rotation_step, angle_diff);

        projectile_transform.rotation = Quat::from_rotation_z(new_angle);
        
        // let rotation_step = smaller_magnitude(homing.homing_speed * time.delta_seconds(), angle_diff);
        // let new_angle = current_angle + rotation_step;



        // let distance = (target.pos - transform.translation).truncate();
        // if distance.length() > homing.homing_distance {
        //     continue;
        // }

        // let target_angle = distance.y.atan2(distance.x) - std::f32::consts::FRAC_PI_2;

        // transform.rotation = Quat::from_rotation_z(new_angle);
    }
}

pub struct TurretsPlugin;

impl Plugin for TurretsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(FixedUpdate, (
                projectile_system,
                turret_targeting_system,
                projectile_turret_attack_system,
                beam_turret_attack_system,
                homing_projectile_system
            ).run_if(
                in_state(GameState::AttackWave).and_then(in_state(PauseState::Running))
            ))

            .add_systems(Update, (
                flag_idle_turrets,
                idle_rotation_system,
                upgrade_turret_system
            ).run_if(
                in_state(AppState::InGame).and_then(in_state(PauseState::Running))
            ))

            ;
    }
}
