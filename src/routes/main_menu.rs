use bevy::ecs::component::StorageType;
use routes::AppSettingsPage;

use crate::*;

#[derive(Debug)]
pub struct MainMenuPage;

impl Component for MainMenuPage {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(_hooks: &mut bevy::ecs::component::ComponentHooks) {
        _hooks.on_add(|mut world, entity, _| {
            
            let mut commands = world.commands();

            commands.entity(entity).insert(
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),

                        justify_content: JustifyContent::End,

                        ..default()
                    },
                    ..default()
                }
            ).with_children(|background| {
                background.spawn(
                    NodeBundle {
                        style: Style {
                            height: Val::Percent(100.0),
                            width: Val::Percent(30.0),

                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(10.0),
                            padding: UiRect::all(Val::Px(10.0)),

                            ..default()
                        },
                        background_color: Color::RED.into(),
                        ..default()
                    }
                ).with_children(|sidebar| {
                    for button_type in MainMenuButton::iter() {
                        sidebar.spawn((
                            ButtonBundle {
                                style: main_button_style(),
                                ..default()
                            },
                            On::<Pointer<Click>>::run(main_menu_button_clicked),
                            button_type
                        )).with_children(|button| {
                            button.spawn((
                                TextBundle::from_section(button_type.to_string(), main_button_text_style()),
                                Pickable::IGNORE
                            ));
                        });
                    }
                });
            });
        });
    }
}

#[derive(Component, Clone, PartialEq, EnumIter, Copy)]
enum MainMenuButton {
    Continue,
    NewGame,
    Settings,
    QuitGame
}

impl std::fmt::Display for MainMenuButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            MainMenuButton::Continue => "CONTINUE",
            MainMenuButton::NewGame => "NEW GAME",
            MainMenuButton::Settings => "SETTINGS",
            MainMenuButton::QuitGame => "QUIT GAME",
        };

        write!(f, "{output}")
    }
}

fn main_button_style() -> Style {
    Style {
        ..default()
    }
}

fn main_button_text_style() -> TextStyle {
    TextStyle {
        color: Color::WHITE,
        font_size: 40.0,
        ..default()
    }
}

fn main_menu_button_clicked(
    listener: Listener<Pointer<Click>>,
    query: Query<Entity, With<MainMenuPage>>,
    buttons: Query<&MainMenuButton>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    mut exit: EventWriter<AppExit>,
) {
    let target = listener.target;
    let Ok(button) = buttons.get(target) else { return };
    let Ok(main_menu) = query.get_single() else { return };

    info!("Clicked: {}", button.to_string());

    match button {
        MainMenuButton::Continue => { }
        MainMenuButton::NewGame => {
            commands.entity(main_menu).despawn_recursive();
            commands.spawn(GameRoute);

            next_state.set(AppState::InGame);
        }

        MainMenuButton::Settings => {
            commands.entity(main_menu).despawn_recursive();
            commands.spawn(AppSettingsPage);
        }

        MainMenuButton::QuitGame => {
            exit.send(AppExit::Success);
        }
    };
}
