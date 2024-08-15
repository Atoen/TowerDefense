use bevy::input::mouse::{MouseButtonInput, MouseMotion, MouseScrollUnit, MouseWheel};

use crate::*;

#[derive(Component)]
pub struct GameArena;

#[derive(Event)]
pub struct GameArenaClickedEvent(pub Vec2);

#[derive(Component)]
struct Pointer;

#[derive(Resource, Default)]
struct InputDragData {
    drag_distance: Vec2,
    input_held: bool,
    is_dragging: bool
}

fn init(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn((
        Pointer,
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Circle {radius: 20.0})),
            material: materials.add(Color::WHITE),
            ..default()
        }
    ));
}

const DRAG_DISTANCE_THRESHOLD: f32 = 10.0;
const DRAG_DISTANCE_THRESHOLD_2: f32 = DRAG_DISTANCE_THRESHOLD * DRAG_DISTANCE_THRESHOLD;

fn game_arena_clicked_system(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut button_events: EventReader<MouseButtonInput>,
    mut motion_events: EventReader<MouseMotion>,
    mut drag_data: ResMut<InputDragData>,
    mut game_camera: Query<(&mut Transform, &OrthographicProjection), With<GameCamera>>,
    mut writer: EventWriter<GameArenaClickedEvent>
) {
    let window = windows.single();
    let Some(cursor_pos) = window.cursor_position() else { 
        drag_data.input_held = false;
        drag_data.is_dragging = false;
        return;
    };

    if window.size().y - cursor_pos.y < 100.0 {
        return;
    }

    let Ok((mut transform, projection)) = game_camera.get_single_mut() else { return };

    for button_event in button_events.read() {
        if button_event.button == MouseButton::Left {
            drag_data.input_held = button_event.state.is_pressed();

            if drag_data.input_held {
                drag_data.is_dragging = false;
                drag_data.drag_distance = Vec2::ZERO;
            } else if drag_data.is_dragging {
                debug!("Drag ended");
            } else {
                debug!("Mouse click");
                writer.send(GameArenaClickedEvent(cursor_pos));
            }
        }
    }

    if drag_data.input_held {
        for mouse_motion in motion_events.read() {
            drag_data.drag_distance += mouse_motion.delta;
        }

        if !drag_data.is_dragging && drag_data.drag_distance.length_squared() >= DRAG_DISTANCE_THRESHOLD_2 {
            drag_data.is_dragging = true;
            drag_data.drag_distance = Vec2::ZERO;
            debug!("Drag started");
        }

        if drag_data.is_dragging {
            let normalized_displacement = drag_data.drag_distance * projection.scale;
            move_camera(&mut transform, &normalized_displacement);

            drag_data.drag_distance = Vec2::ZERO;
        }
    }
}

fn move_camera(camera_transform: &mut Transform, displacement: &Vec2) {
    camera_transform.translation.x -= displacement.x;
    camera_transform.translation.y += displacement.y;

    const X_MIN: f32 = -500.0;
    const X_MAX: f32 = 500.0;
    const Y_MIN: f32 = -300.0;
    const Y_MAX: f32 = 300.0;

    camera_transform.translation.x = camera_transform.translation.x.clamp(X_MIN, X_MAX);
    camera_transform.translation.y = camera_transform.translation.y.clamp(Y_MIN, Y_MAX);
}

fn handle_scroll(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut scroll: EventReader<MouseWheel>,
    game_arena: Query<Option<&PickingInteraction>, With<GameArena>>,
    mut game_camera: Query<(&mut Transform, &mut OrthographicProjection), With<GameCamera>>
) {
    let Ok(Some(PickingInteraction::Hovered)) = game_arena.get_single() else { return };
    let Ok((mut transform, mut projection)) = game_camera.get_single_mut() else { return };

    let window = windows.single();
    let cursor = window.cursor_position();

    for event in scroll.read() {
        let zoom_amount = match event.unit {
            MouseScrollUnit::Line => event.y * 0.1,
            MouseScrollUnit::Pixel => event.y * 0.001
        };

        let previous_scale = projection.scale;

        projection.scale *= 1.0 - zoom_amount;
        projection.scale = projection.scale.clamp(0.2, 1.5);

        if zoom_amount < 0.0 || (previous_scale - projection.scale).abs() < 0.001 {
            continue;
        }

        let Some(cursor) = cursor else { continue };

        let wolrd_pos = cursor_to_world_pos(cursor, window.size());
        let translation = Vec2::new(-wolrd_pos.x, wolrd_pos.y) * zoom_amount * projection.scale;
        
        move_camera(&mut transform, &translation);
    }
}

fn game_arena_exists(query: Query<Entity, With<GameArena>>) -> bool {
    !query.is_empty()
}

pub struct GameArenaPlugin;
impl Plugin for GameArenaPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(InputDragData::default())

            .add_event::<GameArenaClickedEvent>()

            .add_systems(Startup, init)

            .add_systems(Update, handle_scroll)

            .add_systems(Update, game_arena_clicked_system
                .run_if(game_arena_exists));
    }
}
