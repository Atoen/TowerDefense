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
use components::turrets::Target;
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{Display, EnumCount, EnumIter, IntoStaticStr};
use systems::turrets::*;
use bevy_lunex::prelude::*;
use ui::button::TurretPickerButton;
use crate::ui::button::ButtonPlugin;

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

#[derive(Resource)]
pub struct WeaponPageNumber(pub u8);

pub struct SelectedWeapon(pub Option<TurretType>);

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
        .add_plugins(ButtonPlugin)
        .add_plugins(bevy_framepace::FramepacePlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (
            move_target,
            projectile_system,
            decaying_projectile_system,
            homing_projectile_system,
            explosion_spawn_system,
            aoe_animation_system,
            aoe_turret_attack_system,
            turret_targeting_system,
            projectile_turret_attack_system,
            flag_idle_turrets,
            idle_rotation_system,
            // spawn_projectile_turret.run_if(
            //     input_just_pressed(MouseButton::Left)
            // ),
            // spawn_aoe_turret.run_if(
            //     input_just_pressed(MouseButton::Right)
            // )
        ))
        .add_systems(
            Update, 
            (read_turret_button_events, read_game_arena_events)
                .distributive_run_if(on_event::<UiClickEvent>())
                .distributive_run_if(input_just_pressed(MouseButton::Left)))
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
    commands.insert_resource(WeaponPageNumber(0));  
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
        camera.spawn (
            StyledCursorBundle {
                cursor: Cursor2d::new()
                    .set_index(CursorIcon::Default, 0, (14.0, 14.0))
                    .set_index(CursorIcon::Pointer, 1, (10.0, 12.0))
                    .set_index(CursorIcon::Grab, 2, (40.0, 40.0)),
                atlas: TextureAtlas {
                    layout: texture_atlases.add(TextureAtlasLayout::from_grid(UVec2::splat(80), 3, 1, None, None)),
                    index: 0,
                },
                sprite: SpriteBundle {
                    texture: asset_server.load(CURSOR_SHEET),
                    transform: Transform { scale: Vec3::new(0.45, 0.45, 1.0), ..default() },
                    sprite: Sprite {
                        color: Color::BEVYPUNK_YELLOW.with_alpha(2.0),
                        anchor: Anchor::TopLeft,
                        ..default()
                    },
                    ..default()
                },
                ..default()
            }
        );
    });

    commands.spawn((
        Target {
            pos: Vec3::ZERO
        }, 
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Circle {radius: 20.0})),
            material: materials.add(Color::WHITE.with_alpha(0.0)),
            ..default()
        }
    ));

    set_ui(commands, asset_server, materials, meshes);
}

// const TURRET_SELEC_BUTTON_SIZE: Vp<Vec2> = Vp(vec2(10., 10.));

// const TURRET_BUTTON_WIDTH

fn set_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>
) {
    commands.spawn((
        MovableByCamera,
        UiTreeBundle::<MainUi>::from(UiTree::new2d("Game")),
    )).with_children(|ui| {

        let root = UiLink::<MainUi>::path("Root");
        ui.spawn((
            root.clone(),
            GameArena,
            UiLayout::window_full().pack::<Base>(),
            UiZoneBundle::default(),
            UiClickEmitter::SELF
        ));

        let turret_selector = root.add("Turret Selector");
        ui.spawn((
            turret_selector.clone(),
            UiLayout::boundary()
                .pos1((0., Vh(90.)))
                .pos2(Rl(100.))
                .pack::<Base>(),
            UiMaterial2dBundle {
                material: materials.add(Color::BLACK.with_alpha(0.5)),
                ..default()
            }
        ));

        let button_size = (Rh(50.), Rh(50.));
        for (i, turret) in TurretType::iter().enumerate() {
            ui.spawn((
                turret_selector.add(format!("{i}")),
                UiLayout::window()
                    .size(button_size)
                    .pos(get_pos(i, TurretType::COUNT, button_size.into()))
                    .pack::<Base>(),

                TurretPickerButton {
                    text: turret.to_string(),
                    image: asset_server.load(
                        match turret {
                            TurretType::PulseBlaster => AssetPath::PULSE_BLASTER,
                            TurretType::PlasmaRay => AssetPath::PLASMA_RAY,
                            TurretType::RailGun => AssetPath::RAIL_GUN,
                            TurretType::AcidSprayer => AssetPath::ACID_SPRAYER,
                            _ => AssetPath::PULSE_BLASTER,
                        }
                    ),
                    hover_enlarge: true,
                    turret
                },
                ActionButton,
            ));
        }
    });
}

#[derive(Component)]
pub struct DisplayField;

#[derive(Component)]
pub struct ActionButton;

#[derive(Component)]
pub struct  GameArena;

const GAP: f32 = 5.0;

fn get_pos(elememnt_num: usize, total_elements: usize, element_size: UiValue<Vec2>) -> UiValue<Vec2> {

    let width = element_size.get_x();
    let height = element_size.get_y() + Em(1.);

    let fragments = total_elements + 1;
    let fragment_step = Rw(100.) * (1. / fragments as f32);

    let x = -width * 0.5 + fragment_step * (elememnt_num + 1) as f32;
    let y = -height * 0.5 + Rl(50.);

    (x, y).into()
}

fn read_turret_button_events(
    mut events: EventReader<UiClickEvent>,
    buttons: Query<&TurretPickerButton, With<ActionButton>>
) {
    for event in events.read() {
        if let Ok(button) = buttons.get(event.target) {
            info!("Clicked {}!", button.turret);
        }
    }
}

fn read_game_arena_events(
    mut events: EventReader<UiClickEvent>,
    game_arena : Query<Entity, With<GameArena>>,
    windows: Query<&Window, With<PrimaryWindow>>
) {
    for event in events.read() {
        
        if let Ok(_) = game_arena.get(event.target) {

            let window = windows.single();
            let Some(cursor_pos) = window.cursor_position() else {
                return;
            };

            let world_pos = window_to_world_coords(cursor_pos, window.size());

            info!("Clicked at {} ({})!", cursor_pos, world_pos);
        }
    }
}

pub struct AssetPath;
impl AssetPath {
    pub const FONT_LIGHT: &'static str = "fonts/rajdhani/Rajdhani-Light.ttf";
    pub const FONT_REGULAR: &'static str = "fonts/rajdhani/Rajdhani-Regular.ttf";
    pub const FONT_MEDIUM: &'static str = "fonts/rajdhani/Rajdhani-Medium.ttf";
    pub const FONT_SEMIBOLD: &'static str = "fonts/rajdhani/Rajdhani-SemiBold.ttf";
    pub const FONT_BOLD: &'static str = "fonts/rajdhani/Rajdhani-Bold.ttf";

    pub const CURSOR: &'static str = "images/cursor.png";

    pub const ACID_SPRAYER: &'static str = "turrets/AcidSprayer.png";
    pub const PLASMA_RAY: &'static str = "turrets/PlasmaRay.png";
    pub const PULSE_BLASTER: &'static str = "turrets/PulseBlaster.png";
    pub const RAIL_GUN: &'static str = "turrets/RailGun.png";
}

pub trait BevypunkColorPalette {
    const BEVYPUNK_RED: Color;
    const BEVYPUNK_RED_DIM: Color;
    const BEVYPUNK_YELLOW: Color;
    const BEVYPUNK_BLUE: Color;
}
impl BevypunkColorPalette for Color {
    const BEVYPUNK_RED: Color = Color::srgba(255./255., 98./255., 81./255., 1.0);
    const BEVYPUNK_RED_DIM: Color = Color::srgba(172./255., 64./255., 63./255., 1.0);
    const BEVYPUNK_YELLOW: Color = Color::linear_rgba(252./255., 226./255., 8./255., 1.0);
    const BEVYPUNK_BLUE: Color = Color::srgba(8./255., 226./255., 252./255., 1.0);
}
