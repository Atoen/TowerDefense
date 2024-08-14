pub mod main_menu;
pub use main_menu::*;

pub mod game;
pub use game::*;

use bevy::prelude::*;

pub struct RoutePlugin;
impl Plugin for RoutePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MainMenuRoutePlugin);
        app.add_plugins(GameLayoutPlugin);
    }
}
