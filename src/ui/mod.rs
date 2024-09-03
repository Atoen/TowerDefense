pub mod game_status;
use bevy_mod_picking::focus::PickingInteraction;
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

use crate::ColorPalette;

const DISABLED_BUTTON_COLOR: Color = Color::GRAY_700;

#[derive(Event)]
pub struct BuildableRemovedEvent(pub Entity);

#[derive(Component, Default)]
pub struct InteractionColors {
    default: Color,
    hover: Option<Color>,
    pressed: Option<Color>,
}

impl InteractionColors {
    pub const BUTTON_DEFAULT: InteractionColors = InteractionColors {
        default: Color::GRAY_500,
        hover: Some(Color::WHITE),
        pressed: Some(Color::GRAY_600),
    };
}

fn update_button_colors(
    mut buttons: Query<(Option<&PickingInteraction>, &mut BackgroundColor), With<Button>>,
) {
    use bevy_mod_picking::prelude::PickingInteraction;

    buttons.iter_mut().for_each(|(interaction, mut button_color)| {
        *button_color = match interaction {
            Some(PickingInteraction::Pressed) => Color::srgb(0.35, 0.75, 0.35),
            Some(PickingInteraction::Hovered) => Color::srgb(0.25, 0.25, 0.25),
            Some(PickingInteraction::None) | None => Color::srgb(0.15, 0.15, 0.15),
        }
        .into();
    });
}

fn interaction_container_foreground_system(
    mut query: Query<(&PickingInteraction, &Children, &InteractionColors)>,
    mut text_query: Query<&mut Text>,
) {
    query.iter_mut().for_each(|(interaction, children, colors)| {
        let color = match *interaction {
            PickingInteraction::Pressed => colors.pressed.unwrap_or(colors.default),
            PickingInteraction::Hovered => colors.hover.unwrap_or(colors.default),
            PickingInteraction::None => colors.default,
        };

        children.iter().for_each(|&child| {
            if let Ok(mut text) = text_query.get_mut(child) {
                for section in text.sections.iter_mut() {
                    section.style.color = color;
                }
            }
        });
    });
}

fn interaction_foreground_system(
    mut query: Query<(&PickingInteraction, &mut Text, &InteractionColors)>
) {
    query.iter_mut().for_each(|(interaction, mut text, colors)| {
        let color = match *interaction {
            PickingInteraction::Pressed => colors.pressed.unwrap_or(colors.default),
            PickingInteraction::Hovered => colors.hover.unwrap_or(colors.default),
            PickingInteraction::None => colors.default
        };

        text.sections.iter_mut().for_each(|section| {
            section.style.color = color;
        });
    });
}

fn image_interaction_system(
    mut query: Query<(&PickingInteraction, &mut UiImage, &InteractionColors)>,
) {
    query.iter_mut().for_each(|(interaction, mut ui_image, colors)| {
        let color = match *interaction {
            PickingInteraction::Pressed => colors.pressed.unwrap_or(colors.default),
            PickingInteraction::Hovered => colors.hover.unwrap_or(colors.default),
            PickingInteraction::None => colors.default,
        };

        ui_image.color = color;
    });
}

pub struct ComponentPlugin;
impl Plugin for ComponentPlugin {
    fn build(&self, app: &mut App) {
        app

            .add_systems(Update, (
                update_button_colors,
                interaction_foreground_system,
                interaction_container_foreground_system,
                image_interaction_system
            ))

            .add_plugins((
                GameStatusPlugin,
                WaponSelectorPlugin,
                WeaponPagePlugin,
                BuildInfoPlugin,
                GameArenaPlugin,
                InfoTextPlugin,
                ManageBuildablePlugin,
                HologramPLugin    
            ));
    }
}
