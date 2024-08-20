pub mod components;
pub use components::*;

pub mod systems;
pub use systems::*;

use crate::*;

const DEFAULT_PROJECTILE_SPEED: f32 = 300.0;
const TARGET_RADIUS: f32 = 20.0;
const DESPAWN_MARGIN: f32 = 200.0;

fn projectile_system(
    mut commands: Commands,
    time: Res<Time>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut projectiles: Query<
        (Entity, &mut Transform, &mut Projectile, Option<&LinearVelocity>, Option<&Explosive>),
        Without<Alien>>,
    mut aliens: Query<(&GlobalTransform, &mut Alien)>
) {
    let delta = time.delta_seconds();
    let Ok(window) = windows.get_single() else { return };

    let (window_width, window_hegiht) = (window.width(), window.height());

    for (
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

        for (alien_transform, mut alien) in &mut aliens {
            let distance_2 = projectile_transform.translation.distance_squared(alien_transform.translation());

            if distance_2 < (projectile.radius + TARGET_RADIUS) * (projectile.radius + TARGET_RADIUS) {

                alien.add_damage(&projectile.damage);

                if let Some(_explosive) = explosive {
                    commands.entity(projectile_entity).despawn();
                } 
                else if let Pierce::Finite(ref mut pierce) = projectile.pierce {
                    *pierce -= 1;

                    if *pierce == 0 {
                        commands.entity(projectile_entity).despawn();
                    }
                }
            }
        }
    }
}

fn consumable_damage_system(
    time: Res<Time>,
    mut commands: Commands,
    mut aliens: Query<(&GlobalTransform, &mut Alien)>,
    mut consumables: Query<(Entity, &Transform, AnyOf<(&mut RotorBlades, &ProximityMine)>)>
) {
    'consumables: for (consumable_entity, consumable_transform, mut blades_or_mine) in &mut consumables {
        for (alien_transform, mut alien) in &mut aliens {

            let distance_2 = alien_transform.translation().distance_squared(consumable_transform.translation);

            match blades_or_mine {
                (None, Some(mine)) => {
                    if mine.trigger_radius * mine.trigger_radius >= distance_2 {
                        commands.entity(consumable_entity).despawn();

                        continue 'consumables;
                    }
                }
                (Some(ref mut blades), None) => {
                    if blades.durability <= 0.0 {
                        commands.entity(consumable_entity).despawn();

                        continue 'consumables;
                    }

                    if blades.radius * blades.radius >= distance_2 {
                        alien.add_damage(&blades.damage);
                        blades.durability -= time.delta_seconds();

                        if blades.durability <= 0.0 {
                            commands.entity(consumable_entity).despawn_recursive();

                            continue 'consumables;
                        }
                    }
                }
                _ => { }
            }
        }
    } 
}

pub struct TurretsPlugin;

impl Plugin for TurretsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                projectile_system,
                turret_targeting_system,
                projectile_turret_attack_system,
                consumable_damage_system
            ).run_if(in_state(GameState::AttackWave)))

            .add_systems(Update, (
                flag_idle_turrets,
                idle_rotation_system,
            ).run_if(in_state(AppState::InGame)))

            ;
    }
}
