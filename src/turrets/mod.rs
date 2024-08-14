pub mod bundles;
pub use bundles::*;

pub mod systems;
pub use systems::*;

pub mod components;
pub use components::*;


use bevy::prelude::*;

pub struct TurretsPlugin;
impl Plugin for TurretsPlugin {
    fn build(&self, app: &mut App) {
        
    }
}