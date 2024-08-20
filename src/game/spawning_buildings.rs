use components::turrets::*;
use game::GameLayerOrder;

use crate::*;

#[derive(Event)]
pub struct BuildEvent {
    pub buildable: Buildable,
    pub cell_pos: UVec2
}

pub(crate) fn build_clicked_system(
    mut commands: Commands,
    mut reader: EventReader<BuildEvent>,
    game_textures: Res<GameTextures>,
    mut grid: ResMut<GameGrid>,
    mut cash_event_writer: EventWriter<CashChangedEvent>
) {
    for BuildEvent { cell_pos, buildable } in reader.read() {
        if !grid.can_place(cell_pos, buildable) {
            info!("Can't place {} there", buildable);
            continue;
        }
        
        let Some(cell) = grid.get_cell_mut(cell_pos) else { continue };

        let price = match buildable {
            Buildable::Standalone(StandaloneBuildable::Module) => {
                warn!("Module building should be handled before!"); 0
            }
            Buildable::Standalone(StandaloneBuildable::Consumable(consumable)) => {
                spawn_consumable(cell, consumable, &mut commands, &game_textures)
            }
            Buildable::Turret(turret) => {
                spawn_turret(cell, turret, &mut commands, &game_textures)
            }
        };

        info!("Placed {} at {} for {}$", buildable, cell_pos, price);

        cash_event_writer.send(CashChangedEvent { change: -price });
    }
}

fn spawn_consumable(
    cell: &mut Cell,
    consumable: &Consumable,
    commands: &mut Commands,
    game_textures: &Res<GameTextures>
) -> i32 {
    let (entity, price) = match consumable {
        Consumable::ProximityMine => {

            let mine = commands.spawn((
                SpriteBundle {
                    texture: game_textures.mine_texture.clone(),
                    transform: Transform {
                        translation: cell.world_pos().on(GameLayer::STANDALONE),
                        scale: Vec3::splat(0.3),
                        ..default()
                    },
                    ..default()
                },
                ConstantRotation { speed: -std::f32::consts::PI / 16.0 },
                TextureAtlas {
                    layout: game_textures.mine_atlas.clone(),
                    index: 1,
                },
                ProximityMine {
                    on_timer: Timer::from_seconds(0.1, TimerMode::Once),
                    off_timer: Timer::from_seconds(2.0, TimerMode::Once),
                    light_on: false
                },
                RenderLayers::layer(1)
            )).id();

            (mine, 15)
        }

        Consumable::RotorBlades => {

            let parent = commands.spawn((
                SpatialBundle {
                    transform: Transform {
                        translation: cell.world_pos().on(GameLayer::STANDALONE),
                        scale: Vec3::splat(0.4),
                        ..default()
                    },
                    ..default()
                },
                RotorBlades,
                RenderLayers::layer(1)
            )).id();

            commands.spawn((
                SpriteBundle {
                    texture: game_textures.blades_big.clone(),
                    transform: Transform {
                        translation: Vec3::new(12.0, 12.0, 0.0),
                        ..default()
                    },
                    ..default()
                },
                ConstantRotation { speed: std::f32::consts::PI },
                RenderLayers::layer(1)
            )).set_parent(parent);

            commands.spawn((
                SpriteBundle {
                    texture: game_textures.blades_small.clone(),
                    transform: Transform {
                        translation: Vec3::new(-38.0, -38.0, 0.0),
                        ..default()
                    },
                    ..default()
                },
                ConstantRotation { speed: -std::f32::consts::TAU },
                RenderLayers::layer(1)
            )).set_parent(parent);

            (parent, 40)
        }
    };

    cell.set_standalone(StandaloneBuildable::Consumable(*consumable), entity);
    price
}

fn spawn_turret(
    cell: &mut Cell,
    turret: &Turret,
    commands: &mut Commands,
    game_textures: &Res<GameTextures>
) -> i32 {
    let (entity, price) = match turret {
        Turret::PulseBlaster => {

            let base_entity = commands.spawn((
                SpriteBundle {
                    texture: game_textures.turret_ring_texture.clone(),
                    transform: Transform {
                        translation: cell.world_pos().on(GameLayer::TURRET_RING),
                        ..default()
                    },
                    sprite: Sprite {
                        color: Color::RED,
                        custom_size: Some(Vec2::splat(50.0)),
                        ..default()
                    },
                    ..default()
                },
                TextureAtlas {
                    layout: game_textures.turret_ring_atlas.clone(),
                    index: 3,
                },
                RenderLayers::layer(1)
            )).id();

            let turret_entity = commands.spawn((
                AttackDispersion(std::f32::consts::PI / 16.0),
                AttackDelay(Timer::from_seconds(0.2, TimerMode::Repeating)),
                TargetingTurret {
                    targeting_radius: 200.0,
                    mode: TargetingMode::Last,
                    ..default()
                },
                ProjectileTurret,
                ProjectileSpawnOffset::from_vec2(Vec2::new(0.0, GRID_CELL_SIZE as f32 / 2.0)),
                RotationSpeed(std::f32::consts::TAU),
                IdleRotation::default(),
                SpriteBundle {
                    texture: game_textures.pulse_blaster.clone(),
                    transform: Transform {
                        translation: Vec3::ZERO.on(GameLayer::TURRET),
                        ..default()
                    },
                    ..default()
                },
                RenderLayers::layer(1)
            )).set_parent(base_entity).id();

            (turret_entity, 100)
        },
        _ => (commands.spawn_empty().id(), 0)
    };

    cell.set_turret(*turret, entity);
    price
}
