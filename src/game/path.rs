use std::{default, time::Duration};

use bevy_tweening::{lens::TransformScaleLens, Animator, EaseFunction, RepeatCount, RepeatStrategy, Tween};
use game::GameLayerOrder;

use crate::*;

#[derive(Component)]
struct PathElement;

#[derive(Component, Default)]
pub struct PathFollower {
    pub speed: f32,
    pub current_segment: usize,
    pub t: f32,
    pub total_progress: f32,
    pub direction: PathDirection
}

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum PathDirection {
    #[default]
    Horizontal,
    Vertical
}

#[derive(Resource)]
struct PathFollowerSpawnTimer(Timer);

impl Default for PathFollowerSpawnTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(4.0, TimerMode::Repeating))
    }
}

fn path_changed_system(
    mut commands: Commands,
    mut spawn_timer: ResMut<PathFollowerSpawnTimer>,
    grid: Res<GameGrid>,
    query: Query<Entity, With<PathElement>>,
    game_textures: Res<GameTextures>
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }

    let Some(path) = &grid.path else { 
        warn!("Grid has no determined path!");
        return
    };

    match path.points.as_slice() {
        [_, middle @ .., _] => {
            for point in middle {
                commands.spawn((
                    SpriteBundle {
                        texture: game_textures.path_dot.clone(),
                        transform: Transform {
                            translation: cell_to_world_pos(point).on(GameLayer::PATH_DOT),
                            scale: Vec3::splat(0.4),
                            ..default()
                        },
                        ..default()
                    },
                    PathElement,
                    RenderLayers::layer(1)
                ));
            }
        }
        _ => return
    };

    spawn_timer.0.reset();
}

fn clear_path(
    mut commands: Commands,
    query: Query<Entity, With<PathElement>>
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn spawn_path_followers(
    mut commands: Commands,
    time: Res<Time>,
    mut spawn_timer: ResMut<PathFollowerSpawnTimer>,
    grid: Res<GameGrid>,
    game_textures: Res<GameTextures>
) {
    let Some(path) = &grid.path else { 
        warn!("Grid has no determined path!");
        return
    };

    if !spawn_timer.0.tick(time.delta()).finished() {
        return
    }

    let tween = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_millis(500),
        TransformScaleLens {
            start: Vec3::splat(0.8),
            end: Vec3::splat(0.9)
        }
    ).with_repeat_count(RepeatCount::Infinite)
        .with_repeat_strategy(RepeatStrategy::MirroredRepeat);

    commands.spawn((
        SpriteBundle {
            texture: game_textures.path_alien.clone(),
            transform: Transform {
                translation: cell_to_world_pos(path.start()).on(GameLayer::PATH_ALIEN),
                scale: Vec3::splat(0.8),
                ..default()
            },
            ..default()
        },
        PathElement,
        Animator::new(tween),
        PathFollower {
            speed: 50.0,
            ..default()
        },
        RenderLayers::layer(1)
    ));
}

fn move_along_path(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut PathFollower, Option<&CoreDamage>)>,
    grid: Res<GameGrid>,
) {
    let Some(ref path) = grid.path else { return };
    let path_length = path.points.len() as f32 - 1.0;

    for (
        entity,
        mut transform,
        mut follower,
        core_damage
    ) in query.iter_mut() {

        if follower.current_segment < path.points.len() - 1 {
            let start = cell_to_world_pos(&path.points[follower.current_segment]);
            let end = cell_to_world_pos(&path.points[follower.current_segment + 1]);

            let distance = end - start;
            let direction = if distance.x.abs() >= distance.y.abs() {
                PathDirection::Horizontal
            } else { PathDirection::Vertical };

            follower.direction = direction;
            follower.t += (follower.speed * time.delta_seconds()) / GRID_CELL_SIZE as f32;

            let position = start.lerp(end, follower.t);
            transform.translation = Vec3::new(position.x, position.y, transform.translation.z);

            follower.total_progress = (follower.current_segment as f32 + follower.t) / path_length;

            if follower.t >= 1.0 {
                follower.t = 0.0;
                follower.current_segment += 1;
            }
        }
        else {
            commands.entity(entity).despawn_recursive();

            if let Some(damage) = core_damage {
                info!("Core received {} damage!", damage.0); 
            }
        }
    }
}

pub struct PathPlugin;

impl Plugin for PathPlugin {
    fn build(&self, app: &mut App) {
        app

            .init_resource::<PathFollowerSpawnTimer>()

            .add_systems(Update, (
                spawn_path_followers,
                path_changed_system
                    .run_if(on_event::<PathChangedEvent>())
            ).run_if(
                in_state(AppState::InGame).and_then(
                    in_state(GameState::BuildingPhase)
            )))

            .add_systems(Update, move_along_path
                .run_if(in_state(AppState::InGame)))

            .add_systems(OnEnter(GameState::AttackWave), clear_path);
    }
}
