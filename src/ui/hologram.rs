use std::time::Duration;

use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy_tweening::{lens::TransformScaleLens, Animator, EaseFunction, Tween};

use crate::*;

#[derive(Component)]
struct BuildHologram;

#[derive(Component)]
struct HologramRange;

#[derive(Event)]
struct HologramEvent {
    selected_cell: Option<UVec2>,
    bottom_row: BottomRowContent
}


#[derive(Resource)]
struct RangeHologramHandles {
    range_color: Handle<ColorMaterial>,
    explosion_range_color: Handle<ColorMaterial>,
    range_circle: Mesh2dHandle,
}

const DEFAULT_CIRCLE_RADIUS: f32 = 200.0;

fn get_range_transform_scale(target_range: f32) -> Vec3 {
    Vec3::splat(target_range / DEFAULT_CIRCLE_RADIUS)
}

fn init_handles(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.insert_resource(RangeHologramHandles {
        range_color: materials.add(Color::BLUE.with_alpha(0.6)),
        explosion_range_color: materials.add(Color::RED.with_alpha(0.3)),
        range_circle: Mesh2dHandle(meshes.add(Circle { radius: DEFAULT_CIRCLE_RADIUS })),
    })
}

fn display_build_hologram_trigger(
    trigger: Trigger<HologramEvent>,
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    grid: Res<GameGrid>,
    handles: Res<RangeHologramHandles>
) {
    let BottomRowContent::Build(buildable) = trigger.event().bottom_row else { return };

    if buildable.is_module() { return }

    let Some(cell) = trigger.event().selected_cell.and_then(|cell_pos| grid.get_cell(&cell_pos)) else { return };
    if !cell.can_place(buildable) { return }

    match buildable {
        Buildable::MODULE => (),
        Buildable::Standalone(StandaloneBuildable::Consumable(consumable)) => {
            match consumable {
                Consumable::ProximityMine => spawn_proximity_mine_hologram(
                    &mut commands,
                    &game_textures,
                    &handles,
                    cell,
                ),
                Consumable::RotorBlades => spawn_rotor_blades_hologram(
                    &mut commands,
                    &game_textures,
                    cell,
                ),
            }
        },
        Buildable::Turret(turret) => spawn_turret_hologram(
            &mut commands,
            &game_textures,
            &handles,
            turret,
            cell,
        ),
    }

}

fn spawn_proximity_mine_hologram(
    commands: &mut Commands,
    game_textures: &Res<GameTextures>,
    handles: &Res<RangeHologramHandles>,
    cell: &Cell,
) {
    let range_tween = create_scale_tween(get_mine_trigger_radius());
    let explosion_range_tween = create_scale_tween(get_mine_explosion_radius());

    commands.spawn((
        BuildHologram,
        SpriteBundle {
            texture: game_textures.mine_texture.clone(),
            transform: Transform {
                translation: cell.world_pos().on(GameLayer::STANDALONE),
                scale: Vec3::splat(0.3),
                ..default()
            },
            sprite: Sprite {
                color: Color::WHITE.with_alpha(0.8),
                ..default()
            },
            ..default()
        },
        TextureAtlas {
            layout: game_textures.mine_atlas.clone(),
            index: 1,
        },
        RenderLayers::layer(1)
    ));

    let posistion1 = cell.world_pos().on(GameLayer::RANGE);
    let posistion2 = cell.world_pos().on(GameLayer::EXPLOSION_RANGE);

    spawn_range_circle(commands, &handles.range_circle, &handles.range_color, posistion1, range_tween);
    spawn_range_circle(commands, &handles.range_circle, &handles.explosion_range_color, posistion2, explosion_range_tween);
}

fn spawn_rotor_blades_hologram(
    commands: &mut Commands,
    game_textures: &Res<GameTextures>,
    cell: &Cell,
) {
    let first_pos = cell.world_pos().on(GameLayer::STANDALONE) + Vec3::new(12.0, 12.0, 0.0) * 0.4;
    let second_pos = cell.world_pos().on(GameLayer::STANDALONE) + Vec3::new(-38.0, -38.0, 0.0) * 0.4;
    let scale = Vec3::splat(0.4);

    commands.spawn((
        BuildHologram,
        SpriteBundle {
            texture: game_textures.blades_big.clone(),
            transform: Transform {
                translation: first_pos,
                scale,
                ..default()
            },
            sprite: Sprite {
                color: Color::WHITE.with_alpha(0.6),
                ..default()
            },
            ..default()
        },
        RenderLayers::layer(1),
    ));

    commands.spawn((
        BuildHologram,
        SpriteBundle {
            texture: game_textures.blades_small.clone(),
            transform: Transform {
                translation: second_pos,
                scale,
                ..default()
            },
            sprite: Sprite {
                color: Color::WHITE.with_alpha(0.6),
                ..default()
            },
            ..default()
        },
        RenderLayers::layer(1),
    ));
}

fn spawn_turret_hologram(
    commands: &mut Commands,
    game_textures: &Res<GameTextures>,
    handles: &Res<RangeHologramHandles>,
    turret: Turret,
    cell: &Cell,
) {
    let range_tween = create_scale_tween(get_turret_range(turret, 0));

    commands.spawn((
        BuildHologram,
        SpriteBundle {
            texture: get_turret_sprite(turret, game_textures),
            transform: Transform {
                translation: cell.world_pos().on(GameLayer::TURRET),
                ..default()
            },
            sprite: Sprite {
                color: Color::WHITE.with_alpha(0.6),
                ..default()
            },
            ..default()
        },
        RenderLayers::layer(1),
    ));

    let position = cell.world_pos().on(GameLayer::RANGE);

    spawn_range_circle(commands, &handles.range_circle, &handles.range_color, position, range_tween);
}

fn create_scale_tween(radius: f32) -> Tween<Transform> {
    Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_millis(200),
        TransformScaleLens {
            start: Vec3::ZERO,
            end: get_range_transform_scale(radius),
        },
    )
}

fn spawn_range_circle(
    commands: &mut Commands,
    mesh: &Mesh2dHandle,
    material: &Handle<ColorMaterial>,
    position: Vec3,
    tween: Tween<Transform>,
) {
    commands.spawn((
        BuildHologram,
        HologramRange,
        MaterialMesh2dBundle {
            mesh: mesh.clone(),
            material: material.clone(),
            transform: Transform {
                translation: position,
                scale: Vec3::ZERO,
                ..default()
            },
            ..default()
        },
        Animator::new(tween),
        RenderLayers::layer(1),
    ));
}

fn display_manage_hologram_trigger(
    trigger: Trigger<HologramEvent>,
    mut commands: Commands,
    grid: Res<GameGrid>,
    handles: Res<RangeHologramHandles>,
    turret_level: Query<&TurretLevel>
) {
    let BottomRowContent::Manage { buildable, entity } = trigger.event().bottom_row else { return };

    let Some(cell) = trigger.event().selected_cell.and_then(|cell_pos| grid.get_cell(&cell_pos)) else { return };

    match buildable {
        Buildable::Standalone(StandaloneBuildable::Consumable(Consumable::ProximityMine)) => {
            let range_tween = create_scale_tween(get_mine_trigger_radius());
            let explosion_range_tween = create_scale_tween(get_mine_explosion_radius());

            let posistion1 = cell.world_pos().on(GameLayer::RANGE);
            let posistion2 = cell.world_pos().on(GameLayer::EXPLOSION_RANGE);

            spawn_range_circle(&mut commands, &handles.range_circle, &handles.range_color, posistion1, range_tween);
            spawn_range_circle(&mut commands, &handles.range_circle, &handles.explosion_range_color, posistion2, explosion_range_tween);

        }
        Buildable::Turret(turret) => {
            let Ok(level) = turret_level.get(entity) else {
                warn!("Missing level for turret entity: {}!", turret);
                return;
            };

            let range = get_turret_range(turret, level.0);
            let range_tween = create_scale_tween(range);

            let posistion = cell.world_pos().on(GameLayer::RANGE);

            spawn_range_circle(&mut commands, &handles.range_circle, &handles.range_color, posistion, range_tween);
        }
        _ => ()
    }
}

fn turret_upgraded_trigger(
    trigger: Trigger<TurretUpgradedEvent>,
    mut holograms : Query<&mut Transform, With<HologramRange>>,
) {
    let Ok(mut transform) = holograms.get_single_mut() else { return };

    let TurretUpgradedEvent { turret, level } = trigger.event();
    let ragne = get_turret_range(*turret, *level);

    transform.scale = get_range_transform_scale(ragne);
}

pub struct HologramPLugin;

impl Plugin for HologramPLugin {
    fn build(&self, app: &mut App) {
        app

            .add_systems(Startup, init_handles)

            .observe(turret_upgraded_trigger)
            .observe(display_build_hologram_trigger)
            .observe(display_manage_hologram_trigger)

            .observe(|_trgger: Trigger<HologramEvent>, holograms: Query<Entity, With<BuildHologram>>, mut commands: Commands| {
                for entity in &holograms {
                    commands.entity(entity).despawn_recursive();
                }
            })

            .observe(|trigger: Trigger<CellSelectionChangedEvent>, mut commands: Commands, bottom_row: Res<BottomRowContent>| {
                commands.trigger(HologramEvent {
                    selected_cell: trigger.event().0,
                    bottom_row: *bottom_row,
                });
            })

            .observe(|trigger: Trigger<BottomRowContentChangedEvent>, mut commands: Commands, selected_cell: Res<SelectedCell>| {
                commands.trigger(HologramEvent {
                    selected_cell: selected_cell.0,
                    bottom_row: trigger.event().0,
                });
            })

            .observe(|_trigger: Trigger<BuildEvent>, mut commands: Commands, selected_cell: Res<SelectedCell>, bottom_row: Res<BottomRowContent>| {
                commands.trigger(HologramEvent {
                    selected_cell: selected_cell.0,
                    bottom_row: *bottom_row,
                });
            })

            ;
    }
}
