use std::time::Duration;

use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};
use bevy::scene::ron::de;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::window::PrimaryWindow;
use bevy_prng::WyRand;
use bevy_rand::plugin::EntropyPlugin;
use components::{AttackDelay, ProjectileTurret, Target, TargetingRadius, Turret};
use systems::{movable_system, move_target, rotate_turrets, projectile_turret_attack_system};

const ARROW_SPRITE: &str = "arrow.png";
const ARROW_SIZE: (f32, f32) = (50., 50.);

const BULLET_SPRITE: &str = "bullet.png";

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
    bullet: Handle<Image>
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
        .add_plugins((default_plugins, EntropyPlugin::<WyRand>::default()))
        .add_systems(Startup, setup)
        .add_systems(Update, (
            move_target,
            movable_system,
            rotate_turrets,
            projectile_turret_attack_system))
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
        bullet: asset_server.load(BULLET_SPRITE)
    };
    
    commands.insert_resource(game_textures);
    
    commands.spawn(Camera2dBundle::default());

    commands.spawn((Target, MaterialMesh2dBundle {
        mesh: Mesh2dHandle(meshes.add(Circle {radius: 20.0})),
        material: materials.add(Color::WHITE),
        ..default()
    }));

    commands.spawn((Turret { attacking: true },
         SpriteBundle {
        texture: asset_server.load(ARROW_SPRITE),
        transform: Transform {
            translation: Vec3 { x: 20., y: 143., z: 0. },
            ..default()
        },
        ..default()
    }))
    .insert(TargetingRadius(400.0))
    .insert(AttackDelay { timer: Timer::new(Duration::from_millis(50), TimerMode::Repeating) })
    .insert(ProjectileTurret { projectile_spawn_offset: Vec3 { x: 0., y: ARROW_SIZE.1 / 2., z: 0. }});
}

