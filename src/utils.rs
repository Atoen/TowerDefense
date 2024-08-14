use crate::*;

#[derive(Component)]
pub struct DespawnAfterFrames{
    pub delay: u32,
    pub recursive: bool
}

impl DespawnAfterFrames {
    pub const ROUTE_NAVIGATION: DespawnAfterFrames = DespawnAfterFrames { delay: 3, recursive: true };
}

pub fn despawn_after_frames_system(
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

pub struct UtilPlugin;
impl Plugin for UtilPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, despawn_after_frames_system);
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
    } else {
        diff
    }
}
