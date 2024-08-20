pub mod components;
use std::time::Duration;

use bevy::render::view::visibility;
use bevy_tweening::{lens::TransformScaleLens, Animator, AnimatorState, EaseFunction, Tween};
pub use components::*;

pub mod wave;
use game::GameLayerOrder;
pub use wave::*;

use crate::*;

#[derive(Component, Default)]
pub struct ZigZag {
    pub unwanted_offset: Vec2,
    pub distance_moved: f32,
    pub speed: f32,
    pub range: f32,
    pub moving_positive: bool,
    pub move_direction: PathDirection,
}

pub enum ZigZagSpeed {
    Slow,
    Medium,
    Fast
}

impl ZigZag {
    pub fn new(path_follower_speed: f32, zigzag_speed: ZigZagSpeed, range: f32) -> Self {
        Self { 
            ..default()
        }
    }
}

#[derive(Component)]
struct HealthBar {
    alien_entity: Entity
}

#[derive(Component)]
struct HealthBarBackground;

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
        DamageResistant(0.1),
        KineticDamageResistant(0.2),
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
                speed: 50.0,
                ..default()
            },
            CoreDamage(20.0),
        )).id();
        
        let alien = commands.entity(alien_entity)
            .remove::<(Animator<Transform>, SpawningAlien)>()
            .insert((
                Alien::new(100.0),
                ZigZag {
                    speed: 20.0,
                    range: 10.0,
                    ..default()
                }
            )
        ).set_parent(parent).id();

        let healthbar_backround = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::BLACK,
                    custom_size: Some(Vec2::new(40.0, 5.0)),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(0.0, 30.0, GameLayer::ALIEN_HEALTH_BAR),
                    ..default()
                },
                visibility: Visibility::Hidden,
                ..default()
            },
            HealthBarBackground,
            RenderLayers::layer(1)
        )).set_parent(alien).id();

        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::GREEN,
                    custom_size: Some(Vec2::new(40.0, 5.0)),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(0.0, 0.0, 1.0),
                    ..default()
                },
                visibility: Visibility::Hidden,
                ..default()
            },
            HealthBar { alien_entity: alien },
            RenderLayers::layer(1)
        )).set_parent(healthbar_backround);
    }
}


fn take_damage(
    mut commands: Commands,
    time: Res<Time>,
    mut aliens: Query<(&Parent, &mut Alien, Option<&DamageResistant>, Option<&KineticDamageResistant>, Option<&EnergyDamageResistant>, Option<&ChemicalDamageResistant>)>
) {
    for (
        parent,
        mut alien,
        damage_resistant,
        kinetic_damage_resistant,
        energy_damage_resistant,
        chemical_damage_resistant
    ) in &mut aliens {

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
    mut healthbar_query: Query<(&Parent, &HealthBar, &mut Transform, &mut Sprite, &mut Visibility), Without<HealthBarBackground>>,
    mut healthbar_background: Query<&mut Visibility, With<HealthBarBackground>>,
    alien_query: Query<&Alien>,
) {
    for (
        parent,
        healthbar,
        mut transform,
        mut sprite,
        mut visibility) in &mut healthbar_query {

        if let Ok(alien) = alien_query.get(healthbar.alien_entity) {
            let health_percentage = alien.health / alien.max_health;

            if health_percentage == 1.0 {
                if *visibility == Visibility::Visible {
                    *visibility = Visibility::Hidden;
                    
                    if let Ok(mut background) = healthbar_background.get_mut(parent.get()) {
                        *background = Visibility::Hidden;
                    }
                }
            } else {
                if *visibility == Visibility::Hidden {
                    *visibility = Visibility::Visible; 

                    if let Ok(mut background) = healthbar_background.get_mut(parent.get()) {
                        *background = Visibility::Visible;
                    }
                }

                transform.scale.x = health_percentage;
                transform.translation.x = 0.0 - (1.0 - health_percentage) * (sprite.custom_size.unwrap().x / 2.0);

                sprite.color = if health_percentage >= 0.9 {
                    Color::srgb(0.0, 1.0, 0.0) 
                } else if health_percentage >= 0.5 {
                    Color::srgb(
                        1.0 - (health_percentage - 0.5) / (0.9 - 0.5),
                        1.0,
                        0.0)
                } else {
                    Color::srgb(
                        1.0,
                        health_percentage / 0.5,
                        0.0)
                }
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

fn zigzag_movement_system(
    time: Res<Time>,
    path_followers: Query<&PathFollower>,
    mut query: Query<(&mut ZigZag, &Parent, &mut Transform), With<Alien>>
) {
    fn offset_step(delta: f32, offset: f32) -> f32 {
        let delta_step = -delta * offset.signum();
        smaller_magnitude(delta_step, -offset)
    }

    for (mut zigzag, parent, mut transform) in &mut query {
        
        if let Ok(follower) = path_followers.get(parent.get()) {
            let sign = if zigzag.moving_positive { 1.0 } else { -1.0 };
            let delta = zigzag.speed * time.delta_seconds();

            let translation = &mut transform.translation;

            if zigzag.move_direction != follower.direction {
                zigzag.unwanted_offset = translation.truncate();
                zigzag.move_direction = follower.direction;
                zigzag.distance_moved = 0.0;
            }

            match follower.direction {
                PathDirection::Horizontal => {
                    translation.y += delta * sign;

                    let step = offset_step(delta, zigzag.unwanted_offset.x);

                    zigzag.unwanted_offset.x += step;
                    translation.x += step;
                }
                PathDirection::Vertical => {
                    translation.x += delta * sign;

                    let step = offset_step(delta, zigzag.unwanted_offset.y);

                    zigzag.unwanted_offset.y += step;
                    translation.y += step;
                }
            }

            zigzag.distance_moved += delta;
            if zigzag.distance_moved >= zigzag.range {
                zigzag.moving_positive = !zigzag.moving_positive;
                zigzag.distance_moved = -zigzag.range;
            }
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
                update_path_progress,
                zigzag_movement_system
            )
                .run_if(in_state(GameState::AttackWave)))

            ;
    }
}

