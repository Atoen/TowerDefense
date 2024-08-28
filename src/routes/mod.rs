pub mod main_menu;
pub use main_menu::*;

pub mod game;
pub use game::*;

pub mod settings;
pub use settings::*;

use bevy::prelude::*;

pub struct RoutePlugin;
impl Plugin for RoutePlugin {
    fn build(&self, app: &mut App) {

        app.add_plugins(SettingsPlugin);
        app.add_plugins(GameLayoutPlugin);
    }
}
