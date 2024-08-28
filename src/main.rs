use std::fmt::Debug;

use bevy::core_pipeline::bloom::BloomSettings;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::input::common_conditions::input_just_pressed;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::view::RenderLayers;
use bevy::render::RenderPlugin;
use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};
use bevy::sprite::Anchor;
use bevy::window::{PresentMode, PrimaryWindow};
use bevy_prng::WyRand;
use bevy_rand::plugin::EntropyPlugin;
use bevy_tweening::TweeningPlugin;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use bevy_lunex::prelude::*;

const GRID_CELL_SIZE: u32 = 50;
const GRID_WIDTH: u32 = 40;
const GRID_HEIGHT: u32 = 20;

mod components;
mod systems;
mod turret_bundles;

mod ui;
use ui::*;

mod assets;
use assets::*;

mod routes;
use routes::*;

mod utils;
use utils::*;

mod turrets;
use turrets::*;

mod game;
use game::*;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash, Display)]
pub enum PausedState {
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

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash, Display)]
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
        // .add_plugins(UiDebugPlugin::<MainUi>::new())
        .add_plugins(EntropyPlugin::<WyRand>::default())
        .add_plugins(UiPlugin)
        .add_plugins(bevy_framepace::FramepacePlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(UtilPlugin)
        .add_plugins(ComponentPlugin)
        .add_plugins(RoutePlugin)
        .add_plugins(TweeningPlugin)
        .add_plugins(GamePlugin)
        .add_systems(Startup, setup)
        
        .init_state::<PausedState>()
        .init_state::<AppState>()
        .init_state::<GameState>()

        .insert_resource(GridSize(UVec2::new(GRID_WIDTH, GRID_HEIGHT)))
        // .add_systems(Update, (
        //     // move_target,
        //     projectile_system,
        //     decaying_projectile_system,
        //     homing_projectile_system,
        //     explosion_spawn_system,
        //     aoe_animation_system,
        //     aoe_turret_attack_system,
        //     turret_targeting_system,
        //     projectile_turret_attack_system,
        //     flag_idle_turrets,
        //     idle_rotation_system,
        // ))
        // .add_systems(
        //     Update, 
        //     (read_turret_button_events, read_game_arena_events, read_page_button_events)
        //         .distributive_run_if(on_event::<UiClickEvent>())
        //         .distributive_run_if(input_just_pressed(MouseButton::Left)))
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
    let first_pass_layer = RenderLayers::layer(1);

    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: -1,
                target: image_handle.clone().into(),
                clear_color: Color::NONE.into(),
                ..default()
            },
            ..default()
        },
        first_pass_layer.clone(),
        GameCamera
    ));

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("images/grid_cell.png"),
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
        first_pass_layer.clone(),
        Grid
    ));

    commands.insert_resource(GameTextures::load(&asset_server, texture_atlases));
    commands.insert_resource(UiTextures::load(&asset_server));

    let win_size = WinSize { width: primary.width(), height: primary.height() };
    commands.insert_resource(win_size);
    
    commands.insert_resource(GameCash(10000));

    commands.spawn((
        MainUi,
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
    )).with_children(|camera| {
        camera.spawn(
            CursorBundle::default()

            // StyledCursorBundle {
            //     cursor: Cursor2d::new()
            //         .set_index(CursorIcon::Default, 0, (14.0, 14.0))
            //         .set_index(CursorIcon::Pointer, 1, (10.0, 12.0))
            //         .set_index(CursorIcon::Grab, 2, (40.0, 40.0)),
            //     atlas: TextureAtlas {
            //         layout: texture_atlases.add(TextureAtlasLayout::from_grid(UVec2::splat(80), 3, 1, None, None)),
            //         index: 0,
            //     },
            //     sprite: SpriteBundle {
            //         texture: asset_server.load(CURSOR_SHEET),
            //         transform: Transform { scale: Vec3::new(0.45, 0.45, 1.0), ..default() },
            //         sprite: Sprite {
            //             color: Color::YELLOW.with_alpha(2.0),
            //             anchor: Anchor::TopLeft,
            //             ..default()
            //         },
            //         ..default()
            //     },
            //     ..default()
            // }
        );
    });

    commands.spawn(MainMenuPage);
}

#[derive(Component)]
struct GameCamera;
