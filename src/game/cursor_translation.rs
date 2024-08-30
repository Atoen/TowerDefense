use crate::*;

#[derive(Event)]
pub struct GameCellClickedEvent(pub Option<UVec2>);

fn game_click_trigger(
    trigger: Trigger<GameArenaClickedEvent>,
    mut commanads: Commands,
    game_camera: Query<(&GlobalTransform, &OrthographicProjection), With<GameCamera>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    game_arena: Query<&FillContainer, With<GameArena>>,
) {
    let window = windows.single();

    let Ok((camera_global_transform, projection)) = game_camera.get_single() else { return };

    let world_cursor_pos = cursor_to_world_pos(trigger.event().0, window.size());

    fn calculate_scale_factor(fill_container: &FillContainer) -> f32 {
        // First pass render target height
        1080.0 / fill_container.0.y
    }    

    let scale_factor = game_arena.get_single().map_or(1.5, calculate_scale_factor);
    let click_pos = world_cursor_pos * scale_factor * projection.scale + camera_global_transform.translation();

    let cell = wolrd_to_cell_pos(&click_pos.truncate());

    debug!("Cell {:?} clicked", cell);
    commanads.trigger(GameCellClickedEvent(cell));
}

pub fn wolrd_to_cell_pos(world_pos: &Vec2) -> Option<UVec2> {
    let half_grid_size = Vec2::new(
        (GRID_WIDTH * GRID_CELL_SIZE) as f32 / 2.0,
        (GRID_HEIGHT * GRID_CELL_SIZE) as f32 / 2.0
    );

    let local_pos = *world_pos + half_grid_size;
    
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

pub fn cell_to_world_pos(cell_pos: &UVec2) -> Vec3 {
    let half_grid_size = Vec2::new(
        (GRID_WIDTH * GRID_CELL_SIZE) as f32 / 2.0,
        (GRID_HEIGHT * GRID_CELL_SIZE) as f32 / 2.0
    );

    let world_x = (cell_pos.x * GRID_CELL_SIZE + GRID_CELL_SIZE / 2) as f32 - half_grid_size.x;
    let world_y = (cell_pos.y * GRID_CELL_SIZE + GRID_CELL_SIZE / 2) as f32 - half_grid_size.y;

    Vec3::new(world_x, world_y, 0.0)
}

pub struct CursorTranslationPlugin;
impl Plugin for CursorTranslationPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<GameCellClickedEvent>()

            .observe(game_click_trigger)

            ;
    }
}
