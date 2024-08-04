use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::window::PrimaryWindow;
use bevy_prng::{ChaCha8Rng, WyRand};
use bevy_rand::plugin::EntropyPlugin;
use components::turrets::Target;
use systems::turrets::*;

const ARROW_SPRITE: &str = "arrow.png";
const ARROW_SIZE: (f32, f32) = (50., 50.);

const BULLET_SPRITE: &str = "bullet.png";
const LASER_BEAM_SPRITE: &str = "laser_beam.png";
const RAIL_GUN_SPRITE: &str = "rail_gun.png";
const RAIL_GUN_BEAM_SPRITE: &str = "rail_gun_beam.png";

mod components;
mod systems;

#[derive(Resource)]
pub struct WinSize {
    pub width: f32,
    pub height: f32,
}

#[derive(Resource)]
struct GameTextures {
    arrow: Handle<Image>,
    rail_gun: Handle<Image>,
    bullet: Handle<Image>,
    laser_beam: Handle<Image>,
    rail_gun_beam: Handle<Image>
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
        .add_plugins((default_plugins, EntropyPlugin::<ChaCha8Rng>::default()))
        .add_systems(Startup, setup)
        .add_systems(Update, (
            move_target,
            projectile_system,
            targeting_turret_system,
            projectile_turret_attack_system,
            flag_idle_turrets,
            idle_rotation_system,
            spawn_projectile_turret.run_if(
                input_just_pressed(MouseButton::Left)
            )
        ))
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<&Window, With<PrimaryWindow>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>
) {
    
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
        rail_gun_beam: asset_server.load(RAIL_GUN_BEAM_SPRITE),
    };
    
    commands.insert_resource(game_textures);
    
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        Target {
            pos: Vec3::ZERO
        }, 
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Circle {radius: 20.0})),
            material: materials.add(Color::WHITE),
            ..default()
        }
    ));
}
