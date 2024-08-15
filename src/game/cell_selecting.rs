use std::time::Duration;

use bevy_tweening::{lens::TransformScaleLens, Animator, EaseFunction, RepeatCount, RepeatStrategy, Tween};

use crate::*;

#[derive(Component)]
struct SelectedCellMarker;

#[derive(Resource, Default)]
pub struct SelectedCell(pub Option<UVec2>);

fn highlight_clicked_cell_system(
    mut commands: Commands,
    mut reader: EventReader<GameCellClickedEvent>,
    mut query: Query<(Entity, &mut Transform), With<SelectedCellMarker>>,
    game_textures: Res<GameTextures>,
    selected_buildable: Res<SelectedBuildable>
) {
    for event in reader.read() {

        if let Some(cell_pos) = event.0 {
            
            if let Ok((entity, mut transform)) = query.get_single_mut() {

                let pos = cell_to_world_pos(&cell_pos).with_z(10.0);
                if transform.translation == pos && selected_buildable.0.is_none() {
                    commands.entity(entity).despawn();
                } else {
                    transform.translation = pos;
                }
            } else {

                let tween = Tween::new(
                    EaseFunction::QuadraticInOut,
                    Duration::from_millis(800),
                    TransformScaleLens {
                        start: Vec3::splat(0.9),
                        end: Vec3::splat(1.1)
                    }
                ).with_repeat_count(RepeatCount::Infinite)
                    .with_repeat_strategy(RepeatStrategy::MirroredRepeat);

                commands.spawn((
                    SpriteBundle {
                        texture: game_textures.selected_cell.clone(),
                        transform: Transform {
                            translation: cell_to_world_pos(&cell_pos).with_z(10.0),
                            ..default()
                        },
                        ..default()
                    },
                    Animator::new(tween),
                    SelectedCellMarker,
                    RenderLayers::layer(1)
                ));
            }
        } else if let Ok((entity, _)) = query.get_single_mut() {
            commands.entity(entity).despawn();
        }
    }
}

fn build_system(
    mut commands: Commands,
    mut reader: EventReader<GameCellClickedEvent>,
    mut grid_data: ResMut<GameGrid>,
    selected_buildable: Res<SelectedBuildable>,
    game_textures: Res<GameTextures>
) {
    let Some(buildable) = &selected_buildable.0 else { return };

    for event in reader.read() {
        let Some(cell_pos) = event.0 else { continue };
        let Some(cell) = grid_data.get_cell_mut(&cell_pos) else { continue };

        if buildable.is_module() {
            if cell.is_empty() {
                let module = commands.spawn((
                    SpriteBundle {
                        texture: game_textures.module.clone(),
                        transform: Transform {
                            translation: cell.world_pos().with_z(9.0),
                            scale: Vec3::splat(0.5),
                            ..default()
                        },
                        ..default()
                    },
                    RenderLayers::layer(1)
                )).id();
        
                cell.set_standalone(StandaloneBuildable::Module, module);
        
                info!("Placed Module at {}", cell_pos);
        
            } else if let Some((entity, StandaloneBuildable::Module)) = cell.standalone_entity {
                commands.entity(entity).despawn();
                cell.remove_standalone();
        
                info!("Removed Module at {}", cell_pos);
            }
        }
    }
}

pub struct CellSelectionPlugin;
impl Plugin for CellSelectionPlugin {
    fn build(&self, app: &mut App) {
        app

            .init_resource::<GameGrid>()
            .init_resource::<SelectedCell>()

            .add_systems(Update, build_system
                .run_if(on_event::<GameCellClickedEvent>()))

            .add_systems(PreUpdate, highlight_clicked_cell_system
                .run_if(on_event::<GameCellClickedEvent>()));
    }
}