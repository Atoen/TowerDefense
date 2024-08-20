pub mod cursor_translation;
use std::time::Duration;

use bevy::{ecs::query, sprite::{MaterialMesh2dBundle, Mesh2dHandle}};
use bevy_tweening::{lens::TransformScaleLens, Animator, EaseFunction, RepeatCount, RepeatStrategy, Tween};
pub use cursor_translation::*;

pub mod cell_selecting;
pub use cell_selecting::*;

pub mod buildables;
pub use buildables::*;

pub mod grid;
pub use grid::*;

pub mod spawning_buildings;
pub use spawning_buildings::*;

pub mod path;
pub use path::*;

pub mod aliens;
pub use aliens::*;

pub mod turrets;
pub use turrets::*;

use crate::*;

#[derive(Component)]
pub struct ConstantRotation {
    pub speed: f32
}

#[derive(Component)]
pub struct Core;

#[derive(Component)]
pub struct Portal;

pub struct GameLayer;

impl GameLayer {

    const SELECTED_CELL: f32 = 100.0;

    const PATH_DOT: f32 = 8.0;
    const PATH_ALIEN: f32 = 9.0;

    const STANDALONE: f32 = 10.0;
    const PORTAL: f32 = 11.0;
    const TURRET_RING: f32 = 12.0;
    const TURRET: f32 = 13.0;

    const ALIEN: f32 = 14.0;
    const ALIEN_HEALTH_BAR: f32 = 14.1;

    const PROJECTILE: f32 = 15.0;

    const DEBUG_TARGET: f32 = 9.1;
}

trait GameLayerOrder {
    fn on(self, layer: f32) -> Vec3;
}

impl GameLayerOrder for Vec3 {
    fn on(mut self, layer: f32) -> Vec3 {
        self.z = layer;
        self
    }
}


fn setup_game(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    mut grid: ResMut<GameGrid>,
    mut writer: EventWriter<PathChangedEvent>
) {
    debug!("Setting up game level...");

    let core_pos = UVec2::new(32, 10);
    if let Some(core_cell) = grid.set_special(&core_pos, SpecialCell::Core) {
        commands.spawn((
            SpriteBundle {
                texture: game_textures.core.clone(),
                transform: Transform {
                    translation: core_cell.world_pos().on(GameLayer::STANDALONE),
                    scale: Vec3::splat(0.5),
                    ..default()
                },
                ..default()
            },
            Core,
            RenderLayers::layer(1)
        ));
    } else {
        error!("Invalid core position!");
    }

    let portal_pos = UVec2::new(8, 10);
    if let Some(portal_cell) = grid.set_special(&portal_pos, SpecialCell::Portal) {
        commands.spawn((
            SpriteBundle {
                texture: game_textures.portal.clone(),
                transform: Transform {
                    translation: portal_cell.world_pos().on(GameLayer::PORTAL),
                    scale: Vec3::splat(0.7),
                    ..default()
                },
                ..default()
            },
            ConstantRotation { speed: -std::f32::consts::FRAC_PI_4 },
            Portal,
            RenderLayers::layer(1)
        ));
    } else {
        error!("Invalid portal position!");
    }

    if let PathState::Updated = grid.calculate_path() {
        writer.send(PathChangedEvent);
        debug!("Level set up complete");
    } else {
        error!("Failed to set up level!");
    }
}

fn constant_rotation_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &ConstantRotation)>
) {
    for (mut transform, rotation) in &mut query {
        transform.rotate_local_z(rotation.speed * time.delta_seconds());
    }
}

fn proximity_mine_animation_system(
    time: Res<Time>,
    mut query: Query<(&mut ProximityMineAnimation, &mut TextureAtlas)>
) {
    for (mut mine, mut atlas) in &mut query {
        if mine.light_on {
            if mine.on_timer.tick(time.delta()).finished() {
                mine.light_on = false;
                mine.on_timer.reset();
                
                atlas.index = 1;
            }
        } else if mine.off_timer.tick(time.delta()).finished() {
            mine.light_on = true;
            mine.off_timer.reset();

            atlas.index = 0;
        }
    } 
}

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app

            .add_plugins((
                PathPlugin,
                CellSelectionPlugin,
                CursorTranslationPlugin,
                AliensPlugin,
                TurretsPlugin
            ))

            .add_systems(Update, (
                constant_rotation_system,
                proximity_mine_animation_system,
                build_clicked_system
                    .run_if(on_event::<BuildEvent>())
            ).run_if(in_state(AppState::InGame)))

            .add_systems(OnEnter(AppState::InGame), setup_game);
    }
}