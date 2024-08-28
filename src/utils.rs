use bevy::diagnostic::DiagnosticsStore;

use crate::*;

#[derive(Component)]
pub struct FpsRoot;

#[derive(Component)]
struct FpsText;

#[derive(Component)]
pub struct DespawnAfterFrames {
    pub delay: u32,
    pub recursive: bool
}

#[derive(Component)]
pub struct DespawnAfter(pub Timer);

impl DespawnAfterFrames {
    pub const ONE: DespawnAfterFrames = DespawnAfterFrames { delay: 1, recursive: true };
    pub const TWO: DespawnAfterFrames = DespawnAfterFrames { delay: 2, recursive: true };
    pub const THREE: DespawnAfterFrames = DespawnAfterFrames { delay: 2, recursive: true };
}

fn despawn_after_frames_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DespawnAfterFrames)>
) {
    for (entity, mut despawn) in &mut query {
        despawn.delay -= 1;

        if despawn.delay > 0 {
            continue;
        }

        if despawn.recursive {
            commands.entity(entity).despawn_recursive();
        } else {
            commands.entity(entity).despawn();
        }
    }
}

fn despawn_after_timer_system( 
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut DespawnAfter)>
) {
    for (entity, mut timer) in &mut query {
        if timer.0.tick(time.delta()).finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn setup_fps_counter(
    mut commands: Commands,
) {
    let root = commands.spawn((
        FpsRoot,
        NodeBundle {
            background_color: BackgroundColor(Color::BLACK.with_alpha(0.5)),
            z_index: ZIndex::Global(i32::MAX),
            style: Style {
                position_type: PositionType::Absolute,
                right: Val::Auto,
                top: Val::Percent(1.0),
                bottom: Val::Auto,
                left: Val::Percent(5.0),
                padding: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            visibility: Visibility::Hidden,
            ..default()
        },
    )).id();

    let text_fps = commands.spawn((
        FpsText,
        TextBundle {
            text: Text::from_sections([
                TextSection {
                    value: "FPS: ".into(),
                    style: TextStyle {
                        font_size: 16.0,
                        color: Color::WHITE,
                        ..default()
                    }
                },
                TextSection {
                    value: " N/A".into(),
                    style: TextStyle {
                        font_size: 16.0,
                        color: Color::WHITE,
                        ..default()
                    }
                },
            ]),
            ..default()
        },
    )).id();
    commands.entity(root).add_child(text_fps);
}

fn fps_text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    for mut text in &mut query {
        if let Some(value) = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
        {
            text.sections[1].value = format!("{value:>4.0}");
            text.sections[1].style.color = if value >= 120.0 {
                Color::srgb(0.0, 1.0, 0.0)
            } else if value >= 60.0 {
                Color::srgb(
                    (1.0 - (value - 60.0) / (120.0 - 60.0)) as f32,
                    1.0,
                    0.0,
                )
            } else if value >= 30.0 {
                Color::srgb(
                    1.0,
                    ((value - 30.0) / (60.0 - 30.0)) as f32,
                    0.0,
                )
            } else {
                Color::srgb(1.0, 0.0, 0.0)
            }
        } else {
            text.sections[1].value = " N/A".into();
            text.sections[1].style.color = Color::WHITE;
        }
    }
}

#[derive(Event)]
pub struct FpsCounterVisibilityChanged;

fn fps_counter_showhide(
    mut commands: Commands,
    mut q: Query<&mut Visibility, With<FpsRoot>>,
    mut app_settings: ResMut<AppSettings>,
    kbd: Res<ButtonInput<KeyCode>>,
) {
    if kbd.just_pressed(KeyCode::F3) {
        let mut vis = q.single_mut();
        *vis = match *vis {
            Visibility::Hidden => Visibility::Visible,
            _ => Visibility::Hidden,
        };

        app_settings.display_fps = matches!(*vis, Visibility::Inherited | Visibility::Visible);
        commands.trigger(FpsCounterVisibilityChanged);
    }
}

pub struct UtilPlugin;
impl Plugin for UtilPlugin {
    fn build(&self, app: &mut App) {
        app

            .add_systems(Startup, setup_fps_counter)

            .add_systems(Update, (fps_text_update_system, fps_counter_showhide))

            .add_systems(Update, despawn_after_timer_system)

            .add_systems(Update, despawn_after_frames_system)

            ;
    }
}

pub fn cursor_to_world_pos(cursor_pos: Vec2, window_size: Vec2) -> Vec3 {
    Vec3 { 
        x: cursor_pos.x - window_size.x / 2.0,
        y: window_size.y / 2.0 - cursor_pos.y,
        z: 0.0
    }
}

pub fn smaller_magnitude(a: f32, b: f32) -> f32 {
    if a.abs() < b.abs() {
        a
    } else {
        b
    }
}

pub fn shortest_angle_diff(from: f32, to: f32) -> f32 {
    let diff = (to - from).rem_euclid(2.0 * std::f32::consts::PI);
    if diff > std::f32::consts::PI {
        diff - 2.0 * std::f32::consts::PI
    } else if diff < -std::f32::consts::PI {
        diff + 2.0 * std::f32::consts::PI
    } else {
        diff
    }
}

pub fn map_u32_to_range(value: u32, min: f32, max: f32) -> f32 {
    let normalized = value as f32 / u32::MAX as f32;
    min + normalized * (max - min)
}
