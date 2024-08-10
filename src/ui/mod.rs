pub mod button;
pub use button::*;

pub mod menu_button;
pub use menu_button::*;

pub mod game_status;
pub use game_status::*;

pub mod weapon_selector;
pub use weapon_selector::*;

pub mod weapon_page;
pub use weapon_page::*;

use bevy::prelude::*;

pub struct ComponentPlugin;
impl Plugin for ComponentPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(ButtonPlugin)
            .add_plugins(GameStatusPlugin)
            .add_plugins(WaponSelectorPlugin)
            .add_plugins(WeaponPagePlugin)
            .add_plugins(MenuButtonPlugin);
    }
}