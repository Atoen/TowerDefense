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

pub mod build_info;
pub use build_info::*;

pub mod info_popup;
pub use info_popup::*;

pub mod hologram;
pub use hologram::*;

pub mod info_text;
pub use info_text::*;

pub mod game_arena;
pub use game_arena::*;

pub mod manage_buildable;
pub use manage_buildable::*;

use bevy::prelude::*;

pub struct ComponentPlugin;
impl Plugin for ComponentPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(ButtonPlugin)
            .add_plugins(GameStatusPlugin)
            .add_plugins(WaponSelectorPlugin)
            .add_plugins(ManageBuidablePlugin)
            .add_plugins(WeaponPagePlugin)
            .add_plugins(BuildInfoPlugin)
            .add_plugins(GameArenaPlugin)
            .add_plugins(InfoTextPlugin)
            .add_plugins(HologramPLugin)
            .add_plugins(MenuButtonPlugin);
    }
}
