use bevy::ecs::component::StorageType;

use crate::*;

#[derive(Debug)]
pub struct GameStatus;

impl Component for GameStatus {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(_hooks: &mut bevy::ecs::component::ComponentHooks) {
        _hooks.on_add(|mut world, entity, _| {

            let score = world.resource::<Score>().0;
            let cash = world.resource::<Cash>().0;
            let available_modules = world.resource::<AvailableModules>().0;
            let core_health = world.resource::<CoreHealth>().0;
            let wave_info = *world.resource::<WaveInfo>();

            let gradient = world.resource::<UiTextures>().gradient.clone();
            let pause_image = world.resource::<UiTextures>().pause.clone();
            let star = world.resource::<UiTextures>().star.clone();
            let dollar = world.resource::<UiTextures>().dollar.clone();
            let module = world.resource::<UiTextures>().module.clone();
            let core = world.resource::<UiTextures>().core.clone();
            let alien = world.resource::<GameTextures>().path_alien.clone();

            let mut commands = world.commands();

            commands.entity(entity).insert((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Px(50.0),
                        display: Display::Flex,
                        align_items: AlignItems::Start,
                        ..default()
                    },
                    z_index: ZIndex::Global(100),
                    ..default()
                },
                UiImage {
                    texture: gradient,
                    ..default()
                },
                Pickable::IGNORE
            )).with_children(|status| {
            
                status.spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(50.0),
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                }).with_children(|left| {

                    left.spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Px(30.0),
                                height: Val::Px(30.0),
                                ..default()
                            },
                            ..default()
                        },
                        On::<Pointer<Click>>::run(pause_button_clicked)
                    )).with_children(|pause_button| {
                        
                        pause_button.spawn((
                            ImageBundle {
                                image: pause_image.into(),
                                style: image_style(),
                                ..default()
                            },
                            Pickable::IGNORE
                        ));
                    });
                });
            
                status.spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(0.0),
                        flex_grow: 1.0,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                }).with_children(|center| {
                    center.spawn(ImageBundle {
                            image: star.into(),
                            style: image_style(),
                            ..default()
                        });
                    center.spawn((
                        TextBundle::from_section(score.to_string(), text_style()),
                        ScoreText
                    ));
                });
            
                status.spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(50.0),
                        justify_content: JustifyContent::FlexEnd,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(30.0),
                        margin: UiRect::right(Val::Px(10.0)),
                        ..default()
                    },
                    ..default()
                }).with_children(|right_group| {
            
                    right_group.spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ..default()
                    }).with_children(|cash_group| {
                        cash_group.spawn(ImageBundle {
                                image: dollar.into(),
                                style: image_style(),
                                ..default()
                            });
                        cash_group.spawn((
                            TextBundle::from_section(cash.to_string(), text_style()),
                            CashText
                        ));
                    });
            
                    right_group.spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ..default()
                    }).with_children(|modules_group| {
                        modules_group.spawn(ImageBundle {
                                image: module.into(),
                                style: image_style(),
                                ..default()
                            });
                        modules_group.spawn((
                            TextBundle::from_section(available_modules.to_string(), text_style()),
                            AvailableModulesText
                        ));
                    });
            
                    right_group.spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ..default()
                    }).with_children(|health_group| {
                        health_group.spawn(ImageBundle {
                                image: core.into(),
                                style: image_style(),
                                ..default()
                            });
                        health_group.spawn((
                            TextBundle::from_section(core_health.to_string(), text_style()),
                            CoreHealthText
                        ));
                    });
            
                    right_group.spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ..default()
                    }).with_children(|wave_group| {
                        wave_group.spawn(ImageBundle {
                                image: alien.into(),
                                style: image_style(),
                                ..default()
                            });
                        wave_group.spawn((
                            TextBundle::from_sections([
                                TextSection {
                                    value: wave_info.current.to_string(),
                                    style: text_style()
                                },
                                TextSection {
                                    value: '/'.into(),
                                    style: text_style()
                                },
                                TextSection {
                                    value: wave_info.total.to_string(),
                                    style: text_style()
                                }
                            ]),
                            WaveInfoText
                        ));
                    });
                });
            });
        });
    }
}

fn text_style() -> TextStyle {
    TextStyle {
        font_size: 25.0,
        color: Color::WHITE,
        ..default()
    }
}

fn image_style() -> Style {
    Style {
        height: Val::Px(25.0),
        width: Val::Auto,
        margin: UiRect::all(Val::Px(5.0)),
        ..default()
    }
}

#[derive(Resource)]
pub struct Cash(pub u32);

#[derive(Resource, Default)]
pub struct Score(pub u32);

#[derive(Resource, Default)]
pub struct AvailableModules(pub u32);

#[derive(Resource, Default)]
pub struct CoreHealth(pub u32);

#[derive(Resource, Default, Clone, Copy)]
pub struct WaveInfo {
    pub current: u32,
    pub total: u32
}


#[derive(Component)]
enum StatusButton {
    Pause,
    StartRound
}

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct CashText;

#[derive(Component)]
struct AvailableModulesText;

#[derive(Component)]
struct CoreHealthText;

#[derive(Component)]
struct WaveInfoText;

#[derive(Event)]
pub struct CashChangedEvent {
    pub change: i32
}

fn pause_button_clicked(
    pause_state: Res<State<PausedState>>,
    mut next_pause_state: ResMut<NextState<PausedState>>,
) {
    let next_pause = match pause_state.get() {
        PausedState::Running => PausedState::Paused,
        PausedState::Paused => PausedState::Running
    };

    info!("Current pause state: {}", next_pause);
                
    next_pause_state.set(next_pause);
}

fn game_cash_updated_trigger(
    trgger: Trigger<CashChangedEvent>,
    mut query: Query<&mut Text, With<CashText>>,
    mut game_cash: Option<ResMut<Cash>>
) {
    let Some(cash) = game_cash.as_mut() else {
        return;
    };

    cash.0 = cash.0.wrapping_add_signed(trgger.event().change);

    for mut text in &mut query {
        text.sections[0].value = cash.0.to_string();
    }
}

pub struct GameStatusPlugin;
impl Plugin for GameStatusPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<CashChangedEvent>()

            .init_resource::<Score>()
            .init_resource::<AvailableModules>()
            .init_resource::<CoreHealth>()
            .init_resource::<WaveInfo>()

            .observe( game_cash_updated_trigger)

            ;
    }
}
