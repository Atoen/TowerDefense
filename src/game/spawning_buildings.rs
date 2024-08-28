use game::GameLayerOrder;

use crate::*;

#[derive(Event)]
pub struct BuildEvent {
    pub buildable: Buildable,
    pub cell_pos: UVec2
}

pub(crate) fn build_clicked_trigger(
    trigger: Trigger<BuildEvent>,
    game_textures: Res<GameTextures>,
    mut commands: Commands,
    mut grid: ResMut<GameGrid>,
    mut bottom_row: ResMut<BottomRowContent>
) {
    let BuildEvent { cell_pos, buildable } = trigger.event();

    if !grid.can_place(cell_pos, *buildable) {
        info!("Can't place {} there", buildable);
        commands.trigger(InfoMessageAddedEvent(format!("Can't place {} there", buildable)));
        return;
    }
    
    let Some(cell) = grid.get_cell_mut(cell_pos) else { return };

    match buildable {
        Buildable::Standalone(StandaloneBuildable::Module) => {
            warn!("Module building should be handled before!");
            return;
        }
        Buildable::Standalone(StandaloneBuildable::Consumable(consumable)) => {
            spawn_consumable(cell, consumable, &mut commands, &game_textures)
        }
        Buildable::Turret(turret) => {
            spawn_turret(cell, turret, &mut commands, &game_textures)
        }
    };

    let price = get_buildable_cost(*buildable);

    info!("Placed {} at {} for {}$", buildable, cell_pos, price);

    commands.trigger(CashChangedEvent { change: -price });

    *bottom_row = BottomRowContent::Selector;
    commands.trigger(BottomRowContentChangedEvent(*bottom_row));

    commands.trigger(DeselectCell);
}

fn spawn_consumable(
    cell: &mut Cell,
    consumable: &Consumable,
    commands: &mut Commands,
    game_textures: &Res<GameTextures>
) {
    let entity= match consumable {
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
                ProximityMineAnimation {
                    on_timer: Timer::from_seconds(0.1, TimerMode::Once),
                    off_timer: Timer::from_seconds(2.0, TimerMode::Once),
                    light_on: false
                },
                ProximityMine {
                    trigger_radius: get_mine_trigger_radius(),
                    explosion_radius: get_mine_explosion_radius(),
                    damage: Damage {
                        kind: DamageKind::Instant(200.0),
                        source: DamageSource::Consumable(Consumable::ProximityMine),
                        damage_type: DamageType::Energy,
                    }
                },
                CellEntity {
                    pos: cell.position
                },
                RenderLayers::layer(1)
            )).id();

            mine
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
                RotorBlades {
                    radius: get_rotor_blades_radius(),
                    damage: Damage {
                        kind: DamageKind::OverTime { dps: 100.0, duration: 0.1 },
                        source: DamageSource::Consumable(Consumable::RotorBlades),
                        damage_type: DamageType::Kinetic
                    },
                    durability: 100.0,
                },
                RenderLayers::layer(1),
                CellEntity {
                    pos: cell.position
                },
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

            parent
        }
    };

    cell.set_standalone(StandaloneBuildable::Consumable(*consumable), entity);
}

fn spawn_turret(
    cell: &mut Cell,
    turret: &Turret,
    commands: &mut Commands,
    game_textures: &Res<GameTextures>
) {
    let entity = match turret {
        Turret::PulseBlaster => {

            let parent = spawn_turret_base(commands, game_textures, Color::RED, cell);

            commands.spawn((
                AttackDispersion(std::f32::consts::PI / 32.0),
                AttackDelay::from_fire_rate(get_turret_fire_rate(*turret, 0)),
                TargetingTurret {
                    mode: TargetingMode::First,
                    ..default()
                },
                ProjectileTurret,
                ProjectileSpawnOffset::from_vec2(Vec2::new(0.0, GRID_CELL_SIZE as f32 / 2.0)),
                RotationSpeed(get_turret_rotation_speed(*turret)),
                IdleRotation::default(),
                SpriteBundle {
                    texture: game_textures.pulse_blaster.clone(),
                    transform: Transform {
                        translation: Vec3::ZERO.on(GameLayer::TURRET),
                        ..default()
                    },
                    ..default()
                },
                *turret,
                CellEntity {
                    pos: cell.position
                },
                TurretLevel::default(),
                RenderLayers::layer(1)
            )).set_parent(parent);

            parent
        }

        Turret::IonCannon => {
            let parent = spawn_turret_base(commands, game_textures, Color::ORANGE, cell);

            commands.spawn((
                AttackDelay::from_fire_rate(get_turret_fire_rate(*turret, 0)),
                TargetingTurret {
                    mode: TargetingMode::First,
                    ..default()
                },
                ProjectileTurret,
                ProjectileSpawnOffset::from_vec2(Vec2::new(0.0, GRID_CELL_SIZE as f32 / 2.0)),
                RotationSpeed(get_turret_rotation_speed(*turret)),
                IdleRotation::default(),
                SpriteBundle {
                    texture: game_textures.ion_cannon.clone(),
                    transform: Transform {
                        translation: Vec3::ZERO.on(GameLayer::TURRET),
                        ..default()
                    },
                    ..default()
                },
                *turret,
                CellEntity {
                    pos: cell.position
                },
                TurretLevel::default(),
                RenderLayers::layer(1)
            )).set_parent(parent);

            parent
        }

        Turret::SwarmTurret => {
            let parent = spawn_turret_base(commands, game_textures, Color::RED, cell);

            commands.spawn((
                AttackDispersion(std::f32::consts::PI / 12.0),
                AttackDelay::from_fire_rate(get_turret_fire_rate(*turret, 0)),
                TargetingTurret {
                    mode: TargetingMode::First,
                    ..default()
                },
                ProjectileTurret,
                ProjectileSpawnOffset::from_vec2(Vec2::new(0.0, GRID_CELL_SIZE as f32 / 2.0)),
                RotationSpeed(get_turret_rotation_speed(*turret)),
                IdleRotation::default(),
                SpriteBundle {
                    texture: game_textures.swarm_turret.clone(),
                    transform: Transform {
                        translation: Vec3::ZERO.on(GameLayer::TURRET),
                        ..default()
                    },
                    ..default()
                },
                *turret,
                CellEntity {
                    pos: cell.position
                },
                TurretLevel::default(),
                RenderLayers::layer(1)
            )).set_parent(parent);

            parent
        }

        _ => Entity::PLACEHOLDER
    };

    cell.set_turret(*turret, entity);
}

fn spawn_turret_base(commands: &mut Commands, game_textures: &Res<GameTextures>, color: Color, cell: &mut Cell) -> Entity {
    let parent = commands.spawn((
        SpriteBundle {
            texture: game_textures.turret_ring_texture.clone(),
            transform: Transform {
                translation: cell.world_pos().on(GameLayer::TURRET_RING),
                ..default()
            },
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::splat(50.0)),
                ..default()
            },
            ..default()
        },
        TextureAtlas {
            layout: game_textures.turret_ring_atlas.clone(),
            index: 0,
        },
        TurretLevel::default(),
        CellEntity {
            pos: cell.position
        },
        RenderLayers::layer(1)
    )).id();
    parent
}
