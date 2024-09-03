pub mod components;

use std::time::Duration;
use game::GameLayerOrder;

use bevy_tweening::{lens::TransformScaleLens, Animator, EaseFunction, Tween};
pub use components::*;

pub mod wave;
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

#[derive(Component)]
struct HealthBar {
    alien_entity: Entity
}

#[derive(Component)]
struct HealthBarBackground;

#[derive(Resource)]
struct AlienSpawnTimer(Timer);

#[derive(Component)]
struct AlienSpawnAnimation(Timer);

impl Default for AlienSpawnTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.4, TimerMode::Repeating))
    }
}

fn spawn_animation(
    time: Res<Time>,
    mut query: Query<(&mut AlienSpawnAnimation, &mut Transform)>
) {
    for (mut animation, mut transform) in &mut query {
        animation.0.tick(time.delta());

        transform.scale = Vec3::splat(animation.0.fraction());
    }
}

fn alien_spawn_system(
    mut commands: Commands,
    time: Res<Time>,
    mut spawn_timer: ResMut<AlienSpawnTimer>,
    game_textures: Res<GameTextures>,
    grid: Res<GameGrid>
) {
    if !spawn_timer.0.tick(time.delta()).finished() {
        return
    }

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
        DamageResistance::new(Some(0.2), Some(0.1), None, None),
        AlienSpawnAnimation(Timer::from_seconds(0.5, TimerMode::Once)),
        RenderLayers::layer(1)
    ));
}

fn after_alien_spawn_system(
    mut commands: Commands,
    mut query: Query<(Entity, &AlienSpawnAnimation, &mut Transform)>,
    grid: Res<GameGrid>
) {
    for (alien_entity, animation, mut transform) in &mut query {

        if !animation.0.finished() { continue }

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
            CoreDamage(1),
        )).id();
        
        let alien = commands.entity(alien_entity)
            .remove::<AlienSpawnAnimation>()
            .insert((
                Alien::new(100.0),
                CashReward(2),
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
    mut aliens: Query<(&Parent, &CashReward, &mut Alien, Option<&DamageResistance>)>
) {
    for (
        parent,
        cash_reward,
        mut alien,
        damage_resistance
    ) in &mut aliens {

        let mut health = alien.health;

        for damage in alien.received_damages_mut() {

            let multiplier = damage_resistance.map_or(1.0, |dr| dr.calculate_vulnerability(damage.damage_type));

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
                commands.trigger(CashChangedEvent { change: cash_reward.0 });

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
                alien_spawn_system,
                spawn_animation,
                after_alien_spawn_system,
                take_damage,
                update_healthbar,
                update_path_progress,
                zigzag_movement_system
            )
                .run_if(
                    in_state(GameState::AttackWave).and_then(in_state(PauseState::Running))
                ))

            ;
    }
}

