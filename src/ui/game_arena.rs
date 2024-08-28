use bevy::input::mouse::{MouseButtonInput, MouseMotion, MouseScrollUnit, MouseWheel};

use crate::*;

#[derive(Component)]
pub struct GameArena;

#[derive(Event)]
pub struct GameArenaClickedEvent(pub Vec2);

#[derive(Resource)]
struct InputData {
    first_input: bool,
    drag_distance: Vec2,
    input_held: bool,
    is_dragging: bool
}

impl Default for InputData {
    fn default() -> Self {
        Self {
            first_input: true,
            drag_distance: Default::default(),
            input_held: Default::default(),
            is_dragging: Default::default()
        }
    }
}

const DRAG_DISTANCE_THRESHOLD: f32 = 10.0;
const DRAG_DISTANCE_THRESHOLD_2: f32 = DRAG_DISTANCE_THRESHOLD * DRAG_DISTANCE_THRESHOLD;

fn game_arena_clicked_system(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut commands: Commands,
    mut button_events: EventReader<MouseButtonInput>,
    mut motion_events: EventReader<MouseMotion>,
    mut input_data: ResMut<InputData>,
    mut game_camera: Query<(&mut Transform, &OrthographicProjection), With<GameCamera>>,
    game_arena: Query<Option<&PickingInteraction>, With<GameArena>>
) {
    if let Ok(Some(PickingInteraction::Hovered | PickingInteraction::Pressed)) = game_arena.get_single() {
    } else if !input_data.is_dragging {
        return
    }

    let window = windows.single();
    let Some(cursor_pos) = window.cursor_position() else { 
        input_data.input_held = false;
        input_data.is_dragging = false;
        return;
    };

    let Ok((mut transform, projection)) = game_camera.get_single_mut() else { return };

    for button_event in button_events.read() {
        
        if button_event.button == MouseButton::Left {
            if input_data.first_input {
                input_data.first_input = false;
                return;
            }

            input_data.input_held = button_event.state.is_pressed();

            if input_data.input_held {
                input_data.is_dragging = false;
                input_data.drag_distance = Vec2::ZERO;
            } else if input_data.is_dragging {
                debug!("Drag ended");
            } else {
                debug!("Mouse click");

                commands.trigger(GameArenaClickedEvent(cursor_pos));
            }
        }
    }

    if input_data.input_held {
        for mouse_motion in motion_events.read() {
            input_data.drag_distance += mouse_motion.delta;
        }

        if !input_data.is_dragging && input_data.drag_distance.length_squared() >= DRAG_DISTANCE_THRESHOLD_2 {
            input_data.is_dragging = true;
            input_data.drag_distance = Vec2::ZERO;
            debug!("Drag started");
        }

        if input_data.is_dragging {
            let normalized_displacement = input_data.drag_distance * projection.scale;
            move_camera(&mut transform, &normalized_displacement);

            input_data.drag_distance = Vec2::ZERO;
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

        debug!("Zoom scale: {}", projection.scale);

        if zoom_amount < 0.0 || (previous_scale - projection.scale).abs() < 0.001 {
            continue;
        }

        let Some(cursor) = cursor else { continue };

        let wolrd_pos = cursor_to_world_pos(cursor, window.size());
        let translation = Vec2::new(-wolrd_pos.x, wolrd_pos.y) * zoom_amount * projection.scale;
        
        move_camera(&mut transform, &translation);
    }
}

pub struct GameArenaPlugin;
impl Plugin for GameArenaPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(InputData::default())

            .add_event::<GameArenaClickedEvent>()

            .add_systems(Update, handle_scroll)

            .add_systems(Update, game_arena_clicked_system
                .run_if(in_state(AppState::InGame).and_then(|query: Query<(), With<GameArena>>| !query.is_empty())));
    }
}
