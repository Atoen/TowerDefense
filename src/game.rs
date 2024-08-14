use crate::*;

#[derive(Component)]
struct Marker;

#[derive(Component)]
struct SelectedCell;

#[derive(Event)]
pub struct GameCellClickedEvent(pub Option<UVec2>);

fn init(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Circle { radius: 20.0 })),
            material: materials.add(Color::ORANGE),
            ..default()
        },
        RenderLayers::layer(1),
        Marker
    ));
}

fn game_click_system(
    game_camera: Query<(&GlobalTransform, &OrthographicProjection), With<GameCamera>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    game_arena: Query<&Dimension, With<GameArena>>,
    mut reader: EventReader<GameArenaClickedEvent>,
    mut writer: EventWriter<GameCellClickedEvent>,
    mut marker: Query<&mut Transform, With<Marker>>
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
        writer.send(GameCellClickedEvent(cell));

        let mut marker_transform = marker.single_mut();
        marker_transform.translation = click_pos;
    }
}

fn get_grid_cell_coords(click_pos: &Vec2) -> Option<UVec2> {
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

fn cell_to_world_pos(cell: &UVec2) -> Vec3 {
    let half_grid_size = Vec2::new(
        (GRID_WIDTH * GRID_CELL_SIZE) as f32 / 2.0,
        (GRID_HEIGHT * GRID_CELL_SIZE) as f32 / 2.0
    );

    let world_x = (cell.x * GRID_CELL_SIZE + GRID_CELL_SIZE / 2) as f32 - half_grid_size.x;
    let world_y = (cell.y * GRID_CELL_SIZE + GRID_CELL_SIZE / 2) as f32 - half_grid_size.y;

    Vec3::new(world_x, world_y, 0.0)
}

fn highlight_clicked_cell_system(
    mut commands: Commands,
    mut reader: EventReader<GameCellClickedEvent>,
    mut query: Query<(Entity, &mut Transform), With<SelectedCell>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for event in reader.read() {
        if let Some(cell) = event.0 {
            match query.get_single_mut() {
                Ok((_, mut transform)) => {
                    transform.translation = cell_to_world_pos(&cell).with_z(10.0);
                }
                _ => {
                    commands.spawn((
                        MaterialMesh2dBundle {
                            mesh: Mesh2dHandle(meshes.add(Rectangle { half_size: Vec2::splat(24.0) })),
                            material: materials.add(ColorMaterial::from(Color::BLUE.with_alpha(0.6))),
                            transform: Transform {
                                translation: cell_to_world_pos(&cell).with_z(10.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        SelectedCell,
                        RenderLayers::layer(1)
                    ));
                }
            }
        } else if let Ok((entity, _)) = query.get_single_mut() {
            commands.entity(entity).despawn();
        }
    }
}

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, init)

            .add_event::<GameCellClickedEvent>()

            .add_systems(Update, highlight_clicked_cell_system
                .run_if(on_event::<GameCellClickedEvent>()))

            .add_systems(Update, game_click_system
                .run_if(on_event::<GameArenaClickedEvent>()));
    }
}
