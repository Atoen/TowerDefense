use std::fmt::Debug;
use std::ops::Add;

use bevy::core_pipeline::bloom::BloomSettings;
use bevy::input::common_conditions::input_just_pressed;
use bevy::math::vec2;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};
use bevy::sprite::{Anchor, MaterialMesh2dBundle, Mesh2dHandle};
use bevy::window::PrimaryWindow;
use bevy_framepace::{FramepaceSettings, Limiter};
use bevy_prng::{ChaCha8Rng, WyRand};
use bevy_rand::plugin::EntropyPlugin;
use button::Button;
use components::turrets::Target;
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{Display, EnumCount, EnumIter, IntoStaticStr};
use systems::turrets::*;
use bevy_lunex::prelude::*;

const ARROW_SPRITE: &str = "arrow.png";
const ARROW_SIZE: (f32, f32) = (50., 50.);

const BULLET_SPRITE: &str = "bullet.png";
const LASER_BEAM_SPRITE: &str = "laser_beam.png";
const RAIL_GUN_SPRITE: &str = "rail_gun.png";
const RAIL_GUN_BEAM_SPRITE: &str = "rail_gun_beam.png";
const CURSOR_SHEET: &str = "cursor.png"; 

const EXPLOSION_LEN: usize = 16;

mod components;
mod systems;
mod turret_bundles;

mod ui;
use ui::*;

mod assets;
use assets::*;

mod routes;
use routes::*;

#[derive(Debug, Default, Clone, PartialEq, Eq, Display, EnumIter, EnumCount)]
pub enum TurretType {
    #[default] 
    PulseBlaster,   // ✔
    IonCannon,      // ✔
    SwarmTurret,    // ✔
    PlasmaRay,      // ❌
    RailGun,        // ❌
    CryoGenerator,  // ✔
    Tesla,          // ✔
    SeekerLauncher, // ✔
    AcidSprayer,    // ✔ 
    FireThrower,    // ✔
    Sentinel        // ❌
}

pub struct SelectedWeapon(pub Option<TurretType>);

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash, Display)]
pub enum PausedState {
    #[default]
    Running,
    Paused
}

#[derive(Resource)]
pub struct WinSize {
    pub width: f32,
    pub height: f32,
}

#[derive(Resource)]
pub struct GameTextures {
    pub arrow: Handle<Image>,
    pub rail_gun: Handle<Image>,
    pub bullet: Handle<Image>,
    pub laser_beam: Handle<Image>,
    pub rail_gun_beam: Handle<Image>,
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    let default_plugins = DefaultPlugins.set(RenderPlugin {
        render_creation: RenderCreation::Automatic(WgpuSettings {
            backends: Some(Backends::VULKAN),
            ..default()
        }),
        ..default()
    });

    #[cfg(target_arch = "wasm32")]
    let default_plugins = DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            canvas: Some("#game-canvas".into()),
            ..default()
        }),
        ..default()
    });

    App::new()

        .add_plugins((default_plugins, EntropyPlugin::<ChaCha8Rng>::default(), UiPlugin))
        // .add_plugins(UiDebugPlugin::<MainUi>::new())
        .add_plugins(bevy_framepace::FramepacePlugin)
        
        .add_plugins(ComponentPlugin)
        .add_plugins(RoutePlugin)
        .add_systems(Startup, setup)
        .init_state::<PausedState>()
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
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut settings: ResMut<FramepaceSettings>,
    query: Query<&Window, With<PrimaryWindow>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>
) {

    settings.limiter = Limiter::Auto;

    let Ok(primary) = query.get_single() else {
        return;
    };
    
    let win_size = WinSize { width: primary.width(), height: primary.height() };
    commands.insert_resource(win_size);
    
    let game_textures = GameTextures {
        arrow: asset_server.load(ARROW_SPRITE),
        bullet: asset_server.load(BULLET_SPRITE),
        laser_beam: asset_server.load(LASER_BEAM_SPRITE),
        rail_gun: asset_server.load(RAIL_GUN_SPRITE),
        rail_gun_beam: asset_server.load(RAIL_GUN_BEAM_SPRITE)
    };
    
    commands.insert_resource(game_textures);
    commands.insert_resource(GameCash(10000));
    commands.spawn((
        MainUi,
        BloomSettings::OLD_SCHOOL,
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

    // commands.spawn((
    //     Target {
    //         pos: Vec3::ZERO
    //     }, 
    //     MaterialMesh2dBundle {
            // mesh: Mesh2dHandle(meshes.add(Circle {radius: 20.0})),
    //         material: materials.add(Color::WHITE.with_alpha(0.0)),
    //         ..default()
    //     }
    // ));

    commands.spawn(MainMenuRoute);
}
