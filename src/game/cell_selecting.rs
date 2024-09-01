use std::time::Duration;

use bevy_tweening::{lens::TransformScaleLens, Animator, EaseFunction, RepeatCount, RepeatStrategy, Tween};
use game::GameLayerOrder;

use crate::*;

#[derive(Component)]
struct SelectedCellMarker;

#[derive(Resource, Default)]
pub struct SelectedCell(pub Option<UVec2>);

#[derive(Event)]
pub struct CellSelectionChangedEvent(pub Option<UVec2>);

#[derive(Event)]
pub struct DeselectCell;

fn highlight_clicked_cell_trigger(
    trigger: Trigger<GameCellClickedEvent>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform), With<SelectedCellMarker>>,
    mut selected_cell: ResMut<SelectedCell>,
    game_textures: Res<GameTextures>,
    bottom_row: Res<BottomRowContent>,

) {
    if let Some(cell_pos) = trigger.event().0 {
        
        if let Ok((entity, mut transform)) = query.get_single_mut() {

            let row_is_building = matches!(*bottom_row, BottomRowContent::Build(_));
            let same_cell_pos = selected_cell.0.map_or(false, |selected_cell_pos| {
                selected_cell_pos == cell_pos
            });
            
            if same_cell_pos && !row_is_building {
                commands.entity(entity).despawn();
                selected_cell.0 = None;

                debug!("Unselected cell");

            } else {
                let pos = cell_to_world_pos(&cell_pos).on(GameLayer::SELECTED_CELL);
                transform.translation = pos;
                selected_cell.0 = Some(cell_pos);

                debug!("Selected cell at {:?}", cell_pos);
            }

            commands.trigger(CellSelectionChangedEvent(Some(cell_pos)));
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
                        translation: cell_to_world_pos(&cell_pos).on(GameLayer::SELECTED_CELL),
                        ..default()
                    },
                    ..default()
                },
                Animator::new(tween),
                SelectedCellMarker,
                RenderLayers::layer(1)
            ));

            selected_cell.0 = Some(cell_pos);
            debug!("Selected cell at {:?}", cell_pos);
            
            commands.trigger(CellSelectionChangedEvent(Some(cell_pos)));

        }
    } else if let Ok((entity, _)) = query.get_single_mut() {
        commands.entity(entity).despawn();
        selected_cell.0 = None;

        debug!("Removed cell selection");

        commands.trigger(CellSelectionChangedEvent(None));
    }
}

fn automatic_page_navigation_trigger(
    trigger: Trigger<GameCellClickedEvent>,
    mut commands: Commands,
    mut page: ResMut<WeaponSelectorPage>,
    grid: ResMut<GameGrid>
) {
    let Some(cell) = trigger.event().0.and_then(|cell_pos| grid.get_cell(&cell_pos)) else { 
        return
    };

    if cell.has_moudle() {
        if *page == WeaponSelectorPage::Building {
            *page = WeaponSelectorPage::Standard;
            commands.trigger(PageChangedEvent(*page));
        }
    } else if *page != WeaponSelectorPage::Building {
        *page = WeaponSelectorPage::Building;
        commands.trigger(PageChangedEvent(*page));
    }
}

fn build_module_trigger(
    trigger: Trigger<GameCellClickedEvent>,
    mut commands: Commands,
    mut grid: ResMut<GameGrid>,
    available_modules: Res<AvailableModules>,
    game_textures: Res<GameTextures>,
    bottom_row: Res<BottomRowContent>
) {
    let BottomRowContent::Build(buildable) = *bottom_row else { return };
    if !buildable.is_module() { return }

    let Some(cell_pos) = trigger.event().0 else { return };

    let path_state = grid.try_place_module(&cell_pos,  &available_modules, &mut commands, &game_textures);
    if let PathState::Updated = path_state {
        commands.trigger(PathChangedEvent);
    }
}

fn display_manage_trigger(
    trigger: Trigger<GameCellClickedEvent>,
    mut commands: Commands,
    mut bottom_row: ResMut<BottomRowContent>,
    grid: Res<GameGrid>
) {
    let Some((entity, buildable)) = trigger.event().0
        .and_then(|cell_pos| grid.get_cell(&cell_pos))
        .and_then(|cell| cell.managable_buildable()) else {

        if matches!(*bottom_row, BottomRowContent::Manage {..}) {
            *bottom_row = BottomRowContent::Selector;
            commands.trigger(BottomRowContentChangedEvent(*bottom_row));
        }

        return
    };

    *bottom_row = BottomRowContent::Manage { entity, buildable };
    commands.trigger(BottomRowContentChangedEvent(*bottom_row));
}

fn deselect_cell_trigger(
    _trigger: Trigger<DeselectCell>,
    query: Query<Entity, With<SelectedCellMarker>>,
    mut commands: Commands,
    mut selected_cell: ResMut<SelectedCell>,
) {
    let Ok(entity) = query.get_single() else { return };

    commands.entity(entity).despawn();
    selected_cell.0 = None;
}

#[derive(Event)]
pub struct PathChangedEvent;

pub struct CellSelectionPlugin;
impl Plugin for CellSelectionPlugin {
    fn build(&self, app: &mut App) {
        app

            .init_resource::<GameGrid>()
            .init_resource::<SelectedCell>()
            .add_event::<CellSelectionChangedEvent>()

            .observe(highlight_clicked_cell_trigger)
            .observe(automatic_page_navigation_trigger)
            .observe(build_module_trigger)
            .observe(display_manage_trigger)
            .observe(deselect_cell_trigger)

            ;
    }
}