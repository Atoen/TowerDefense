pub mod cursor_translation;
pub use cursor_translation::*;

pub mod cell_selecting;
pub use cell_selecting::*;

pub mod buildables;
pub use buildables::*;

pub mod grid;
pub use grid::*;

pub mod spawning_buildings;
pub use spawning_buildings::*;


use bevy::prelude::*;

#[derive(Component)]
pub struct ConstantRotation {
    pub speed: f32
}

fn constant_rotation_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &ConstantRotation)>
) {
    for (mut transform, rotation) in &mut query {
        transform.rotate_local_z(rotation.speed * time.delta_seconds());
    }
}

fn mine_system(
    time: Res<Time>,
    mut query: Query<(&mut ProximityMine, &mut TextureAtlas)>
) {
    for (mut mine, mut atlas) in &mut query {
        if mine.light_on {
            if mine.on_timer.tick(time.delta()).finished() {
                mine.light_on = false;
                mine.on_timer.reset();
                
                atlas.index = 1;
            }
        } else if mine.off_timer.tick(time.delta()).finished() {
            mine.light_on = true;
            mine.off_timer.reset();

            atlas.index = 0;
        }
    } 
}

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, constant_rotation_system)

            .add_systems(Update, mine_system)

            .add_plugins(CellSelectionPlugin)

            .add_systems(Update, build_clicked_system
                .run_if(on_event::<BuildEvent>()))

            .add_plugins(CursorTranslationPlugin);
    }
}