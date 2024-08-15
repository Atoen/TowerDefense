use crate::*;

#[derive(Event)]
pub struct GameCellClickedEvent(pub Option<UVec2>);

fn game_click_system(
    game_camera: Query<(&GlobalTransform, &OrthographicProjection), With<GameCamera>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    game_arena: Query<&Dimension, With<GameArena>>,
    mut reader: EventReader<GameArenaClickedEvent>,
    mut writer: EventWriter<GameCellClickedEvent>,
    mut selected_cell: ResMut<SelectedCell>
) {
    let window = windows.single();
    let Ok((camera_global_transform, projection)) = game_camera.get_single() else { return };

    for event in reader.read() {
        let world_cursor_pos = cursor_to_world_pos(event.0, window.size());
    
        fn calculate_scale_factor(dimension: &Dimension) -> f32 {
            // First pass render target height
            1080.0 / dimension.size.y
        }    
    
        let scale_factor = game_arena.get_single().map_or(1.5, calculate_scale_factor);
        let click_pos = world_cursor_pos * scale_factor * projection.scale + camera_global_transform.translation();

        let cell = get_grid_cell_coords(&click_pos.truncate());

        selected_cell.0 = cell;
        writer.send(GameCellClickedEvent(cell));
    }
}

pub fn get_grid_cell_coords(click_pos: &Vec2) -> Option<UVec2> {
    let half_grid_size = Vec2::new(
        (GRID_WIDTH * GRID_CELL_SIZE) as f32 / 2.0,
        (GRID_HEIGHT * GRID_CELL_SIZE) as f32 / 2.0
    );

    let local_pos = *click_pos + half_grid_size;
    
    if local_pos.x < 0.0 || local_pos.y < 0.0 {
        return None;
    }

    let cell_x = (local_pos.x / GRID_CELL_SIZE as f32).floor() as u32;
    let cell_y = (local_pos.y / GRID_CELL_SIZE as f32).floor() as u32;

    if cell_x < GRID_WIDTH && cell_y < GRID_HEIGHT {
        Some(UVec2::new(cell_x, cell_y))
    } else {
        None
    }
}

pub fn cell_to_world_pos(cell: &UVec2) -> Vec3 {
    let half_grid_size = Vec2::new(
        (GRID_WIDTH * GRID_CELL_SIZE) as f32 / 2.0,
        (GRID_HEIGHT * GRID_CELL_SIZE) as f32 / 2.0
    );

    let world_x = (cell.x * GRID_CELL_SIZE + GRID_CELL_SIZE / 2) as f32 - half_grid_size.x;
    let world_y = (cell.y * GRID_CELL_SIZE + GRID_CELL_SIZE / 2) as f32 - half_grid_size.y;

    Vec3::new(world_x, world_y, 0.0)
}

pub struct CursorTranslationPlugin;
impl Plugin for CursorTranslationPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<GameCellClickedEvent>()

            .add_systems(Update, game_click_system
                .run_if(on_event::<GameArenaClickedEvent>()));
    }
}
