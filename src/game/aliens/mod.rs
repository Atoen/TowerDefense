pub mod components;
use std::time::Duration;

use bevy_tweening::{lens::TransformScaleLens, Animator, AnimatorState, EaseFunction, Tween};
pub use components::*;

pub mod wave;
use game::GameLayerOrder;
pub use wave::*;

use crate::*;

pub enum Enemy {
    
}

#[derive(Component)]
struct HealthBar {
    alien_entity: Entity,
    max_health: f32,
}

#[derive(Resource)]
struct AlienSpawnTimer(Timer);

#[derive(Component)]
struct SpawningAlien(Timer);

impl Default for AlienSpawnTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(2.0, TimerMode::Repeating))
    }
}

fn alien_spawn_animation_system(
    mut commands: Commands,
    time: Res<Time>,
    mut spawn_timer: ResMut<AlienSpawnTimer>,
    game_textures: Res<GameTextures>,
    grid: Res<GameGrid>
) {
    if !spawn_timer.0.tick(time.delta()).finished() {
        return
    }

    let tween = Tween::new(
        EaseFunction::QuadraticIn,
        Duration::from_millis(500),
        TransformScaleLens {
            start: Vec3::ZERO,
            end: Vec3::ONE
        }
    );

    commands.spawn((
        SpriteBundle {
            texture: game_textures.path_alien.clone(),
            sprite: Sprite {
                color: Color::GREEN,
                ..default()
            },
            transform: Transform {
                translation: grid.portal_pos.on(GameLayer::ALIEN),
                scale: Vec3::ZERO,
                ..default()
            },
            ..default()
        },
        Animator::new(tween),
        SpawningAlien(Timer::from_seconds(0.5, TimerMode::Once)),
        RenderLayers::layer(1)
    ));
}

fn after_alien_spawn_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut SpawningAlien, &mut Transform)>,
    grid: Res<GameGrid>
) {
    for (alien_entity, mut spawning, mut transform) in &mut query {
        if !spawning.0.tick(time.delta()).finished() { continue }

        transform.translation = Vec3::ZERO;

        let parent = commands.spawn((
            SpatialBundle {
                transform: Transform {
                    translation: grid.portal_pos.on(GameLayer::ALIEN),
                    ..default()
                },
                ..default()
            },
            PathFollower {
                speed: 100.0,
                ..default()
            },
            CoreDamage(20.0),
        )).id();
        
        commands.entity(alien_entity)
            .remove::<(Animator<Transform>, SpawningAlien)>()
            .insert((
                Alien::new(100.0),
            )
        ).set_parent(parent);
    }
}


fn take_damage(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(&Parent, &mut Alien, Option<&DamageResistant>, Option<&KineticDamageResistant>, Option<&EnergyDamageResistant>, Option<&ChemicalDamageResistant>)>
) {
    for (
        parent,
        mut alien,
        damage_resistant,
        kinetic_damage_resistant,
        energy_damage_resistant,
        chemical_damage_resistant
    ) in &mut query {

        let base_vulnerability = 1.0 - damage_resistant.map_or(0.0, |r| r.0);

        let kinetic_vulnerability =
            base_vulnerability * (1.0 - kinetic_damage_resistant.map_or(0.0, |kr| kr.0));

        let energy_vulnerability =
            base_vulnerability * (1.0 - energy_damage_resistant.map_or(0.0, |er| er.0));

        let chemical_vulnerability =
            base_vulnerability * (1.0 - chemical_damage_resistant.map_or(0.0, |cr| cr.0));

        let mut health = alien.health;

        for damage in alien.received_damages_mut() {
            let multiplier = match damage.damage_type {
                DamageType::Energy => energy_vulnerability,
                DamageType::Kinetic => kinetic_vulnerability,
                DamageType::Chemical => chemical_vulnerability,
                DamageType::True => 1.0
            };

            match &mut damage.timing {
                DamageToApplyTiming::Instant(dmg) => {
                    health -= *dmg * multiplier;
                }

                DamageToApplyTiming::OverTime { dps, timer } => {
                    if !timer.tick(time.delta()).finished() {
                        health -= *dps * time.delta_seconds() * multiplier;
                    }
                }
            }

            if health <= 0.0 {
                commands.entity(parent.get()).despawn_recursive();
                break;
            }
        }

        alien.health = health;
        alien.clear_applied_damages();
    }
}

fn update_healthbar(
    mut healthbar_query: Query<(&HealthBar, &mut Sprite)>,
    alien_query: Query<&Alien>,
) {
    for (healthbar, mut sprite) in healthbar_query.iter_mut() {
        if let Ok(alien) = alien_query.get(healthbar.alien_entity) {

            let health_percentage = alien.health / healthbar.max_health;
            if let Some(custom_size) = &mut sprite.custom_size {
                custom_size.x *= health_percentage;
            }
        }
    }
}

fn update_path_progress(
    path_followers: Query<&PathFollower>,
    mut aliens: Query<(&mut Alien, &Parent)>
) {
    for (mut alien, parent) in &mut aliens {
        if let Ok(follower) = path_followers.get(parent.get()) {
            alien.path_progress = follower.total_progress;
        }
    }
}


pub struct AliensPlugin;

impl Plugin for AliensPlugin {
    fn build(&self, app: &mut App) {
        app

            .init_resource::<AlienSpawnTimer>()

            .add_systems(Update, (
                alien_spawn_animation_system,
                after_alien_spawn_system,
                take_damage,
                update_healthbar,
                update_path_progress
            )
                .run_if(in_state(GameState::AttackWave)))

            ;
    }
}

