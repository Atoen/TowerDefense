use std::fmt::Debug;

use bevy::core_pipeline::bloom::BloomSettings;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::view::RenderLayers;
use bevy::render::RenderPlugin;
use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};
use bevy::window::{PresentMode, PrimaryWindow};
use bevy_prng::WyRand;
use bevy_rand::plugin::EntropyPlugin;
use bevy_tweening::TweeningPlugin;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use bevy_mod_picking::prelude::*;

const GRID_CELL_SIZE: u32 = 50;
const GRID_WIDTH: u32 = 40;
const GRID_HEIGHT: u32 = 20;
const MAX_TURRET_LEVEL: u8 = 4;

mod ui;
use ui::*;

mod assets;
use assets::*;

mod routes;
use routes::*;

mod utils;
use utils::*;

mod game;
use game::*;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash, Display)]
pub enum PauseState {
    #[default]
    Running,
    Paused
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash, Display)]
pub enum AppState {
    #[default]
    InMenu,
    InGame
}

#[derive(States, Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Display)]
pub enum GameState {
    #[default]
    BuildingPhase,
    AttackWave
}

#[derive(Resource)]
pub struct GridSize(pub UVec2);

#[derive(Component)]
pub struct Grid;

#[derive(Resource)]
pub struct WinSize {
    pub width: f32,
    pub height: f32,
}

#[derive(Resource)]
struct RenderTarget(Handle<Image>);

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    let default_plugins = DefaultPlugins.set(RenderPlugin {
        render_creation: RenderCreation::Automatic(WgpuSettings {
            backends: Some(Backends::VULKAN),
            ..default()
        }),
        ..default()
    }).set(WindowPlugin {
        primary_window: Some(Window {
            present_mode: PresentMode::AutoNoVsync,
            ..default()
        }),
        ..default()
    });

    #[cfg(target_arch = "wasm32")]
    let default_plugins = DefaultPlugins;

    App::new()
        .add_plugins(default_plugins.set(LogPlugin {
            filter: "wgpu_core=warn,wgpu_hal::vulkan::instance=off,tower_defense=debug".into(),
            level: bevy::log::Level::INFO,
            ..default()
        }))
        .add_plugins(EntropyPlugin::<WyRand>::default())
        .add_plugins(bevy_framepace::FramepacePlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(DefaultPickingPlugins)
        .add_plugins(UtilPlugin)
        .add_plugins(ComponentPlugin)
        .add_plugins(RoutePlugin)
        .add_plugins(TweeningPlugin)
        .add_plugins(GamePlugin)
        .add_systems(Startup, setup)

        .add_systems(OnEnter(AppState::InGame), spawn_game_camera)
        
        .init_state::<PauseState>()
        .init_state::<AppState>()
        .init_state::<GameState>()

        .insert_resource(GridSize(UVec2::new(GRID_WIDTH, GRID_HEIGHT)))
        .run();
}

fn setup(
    asset_server: Res<AssetServer>,
    query: Query<&Window, With<PrimaryWindow>>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    texture_atlases: ResMut<Assets<TextureAtlasLayout>>
) {
    let Ok(primary) = query.get_single() else {
        return;
    };

    let size = Extent3d {
        width: 1920,
        height: 1080,
        ..default()
    };

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[]
        },
        ..default()
    };

    image.resize(size);
    let image_handle = images.add(image);
    commands.insert_resource(RenderTarget(image_handle.clone()));

    commands.insert_resource(GameTextures::load(&asset_server, texture_atlases));
    commands.insert_resource(UiTextures::load(&asset_server));

    let win_size = WinSize { width: primary.width(), height: primary.height() };
    commands.insert_resource(win_size);
    
    commands.insert_resource(Cash(2000));

    commands.spawn((
        BloomSettings::NATURAL,
        InheritedVisibility::default(),
        Camera2dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 1000.0),
            camera: Camera {
                hdr: true,
                ..default()
            },
            ..default()
        }
    ));

    commands.spawn(MainMenuPage);
}

fn spawn_game_camera(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    render_target: Res<RenderTarget>
) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: -1,
                target: render_target.0.clone().into(),
                clear_color: Color::NONE.into(),
                ..default()
            },
            ..default()
        },
        RenderLayers::layer(1),
        GameCamera
    ));

    commands.spawn((
        SpriteBundle {
            texture: game_textures.grid_cell.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::new((GRID_CELL_SIZE * GRID_WIDTH) as f32, (GRID_CELL_SIZE * GRID_HEIGHT) as f32)),
                ..default()
            },
            ..default()
        },
        ImageScaleMode::Tiled {
            tile_x: true,
            tile_y: true,
            stretch_value: 1.0
        },
        RenderLayers::layer(1),
        Grid
    ));
}

#[derive(Component)]
struct GameCamera;
