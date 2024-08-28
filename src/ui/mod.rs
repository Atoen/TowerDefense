pub mod button;
pub use button::*;

pub mod new_button;
pub use new_button::*;

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

fn update_button_colors(
    mut buttons: Query<(Option<&bevy_mod_picking::prelude::PickingInteraction>, &mut BackgroundColor), With<bevy::ui::widget::Button>>,
) {
    use bevy_mod_picking::prelude::PickingInteraction;

    for (interaction, mut button_color) in &mut buttons {
        *button_color = match interaction {
            Some(PickingInteraction::Pressed) => Color::srgb(0.35, 0.75, 0.35),
            Some(PickingInteraction::Hovered) => Color::srgb(0.25, 0.25, 0.25),
            Some(PickingInteraction::None) | None => Color::srgb(0.15, 0.15, 0.15),
        }
        .into();
    }
}


pub struct ComponentPlugin;
impl Plugin for ComponentPlugin {
    fn build(&self, app: &mut App) {
        app

            .add_systems(Update, update_button_colors)

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
