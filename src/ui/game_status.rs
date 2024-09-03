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

                // Wave start button
                status.spawn((
                    NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            right: Val::Px(10.0),
                            top: Val::Px(50.0),
                            padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
                            ..default()
                        },
                        ..default()
                    },
                    InteractionColors {
                        default: Color::GRAY_300,
                        hover: Some(Color::srgb(202./255., 1., 186./255.)), 
                        pressed: Some(Color::GRAY_600)
                    },
                    On::<Pointer<Click>>::run(start_wave_clicked)

                )).with_children(|float| {
                    float.spawn((
                        TextBundle::from_section("Start Wave", TextStyle {
                            color: Color::GRAY_300,
                            font_size: 30.0,
                            ..default()
                        }),
                        Pickable::IGNORE,
                        StartWaveText
                    ));
                });
            
                // Left aligned pause button
                status.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(50.0),
                            justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ..default()
                    },
                )).with_children(|left| {
                    left.spawn((
                        ImageBundle {
                            image: Into::<UiImage>::into(pause_image).with_color(Color::GRAY_500),
                            style: image_style(),
                            ..default()
                        },
                        On::<Pointer<Click>>::run(pause_button_clicked),
                        InteractionColors::BUTTON_DEFAULT
                    ));
                });
            
                // Centered score display
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
            
                // Right aligned status group
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
            
                    // Cash image + text
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
                            TextBundle::from_section(format!("{cash:<4}"), text_style()),
                            CashText
                        ));
                    });
            
                    // Modules image + text
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
                            TextBundle::from_section(format!("{available_modules:<2}"), text_style()),
                            AvailableModulesText
                        ));
                    });

                    // Core image + text
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
                            TextBundle::from_section(format!("{core_health:<2}"), text_style()),
                            CoreHealthText
                        ));
                    });
                    
                    // Wave info image + text
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
                                    value: format!("{:>2}", wave_info.current),
                                    style: text_style()
                                },
                                TextSection {
                                    value: '/'.into(),
                                    style: text_style()
                                },
                                TextSection {
                                    value: format!("{:<2}", wave_info.total),
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

#[derive(Resource)]
pub struct AvailableModules(pub u32);

#[derive(Resource)]
pub struct CoreHealth(pub u32);

#[derive(Resource, Clone, Copy)]
pub struct WaveInfo {
    pub current: u32,
    pub total: u32
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

#[derive(Component)]
struct StartWaveText;

#[derive(Event)]
pub struct CashChangedEvent {
    pub change: i32
}

#[derive(Event)]
pub struct ScoreChangedEvent {
    pub change: i32
}

#[derive(Event)]
pub struct AvailableModulesChangedEvent {
    pub change: i32
}

#[derive(Event)]
pub struct CoreHealthChangedEvent {
    pub change: i32
}

#[derive(Event)]
pub struct WaveChangedEvent {
    pub current: i32,
    pub total: i32
}

fn pause_button_clicked(
    pause_state: Res<State<PauseState>>,
    mut next_pause_state: ResMut<NextState<PauseState>>,
) {
    let next_pause = match pause_state.get() {
        PauseState::Running => PauseState::Paused,
        PauseState::Paused => PauseState::Running
    };

    info!("Current pause state: {}", next_pause);
                
    next_pause_state.set(next_pause);
}

fn start_wave_clicked(
    game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut text: Query<&mut Text, With<StartWaveText>>
) {
    if *game_state.get() == GameState::AttackWave {
        return
    }

    if let Ok(mut text) = text.get_single_mut() {
        text.sections[0].value = "Wave Incoming".into();
    } else {
        warn!("Missing StartWaveText!");
    }
    
    info!("Starting round");
    
    next_game_state.set(GameState::AttackWave);
}

fn score_updated_trigger(
    trgger: Trigger<ScoreChangedEvent>,
    mut query: Query<&mut Text, With<ScoreText>>,
    mut score: ResMut<Score>
) {
    score.0 = score.0.saturating_add_signed(trgger.event().change);

    for mut text in &mut query {
        text.sections[0].value = score.0.to_string();
    }
}

fn cash_updated_trigger(
    trgger: Trigger<CashChangedEvent>,
    mut query: Query<&mut Text, With<CashText>>,
    mut cash: ResMut<Cash>
) {
    cash.0 = cash.0.saturating_add_signed(trgger.event().change);

    for mut text in &mut query {
        text.sections[0].value = cash.0.to_string();
    }
}

fn available_modules_updated_trigger(
    trgger: Trigger<AvailableModulesChangedEvent>,
    mut query: Query<&mut Text, With<AvailableModulesText>>,
    mut modules: ResMut<AvailableModules>
) {
    modules.0 = modules.0.saturating_add_signed(trgger.event().change);

    for mut text in &mut query {
        text.sections[0].value = modules.0.to_string();
    }
}

fn core_health_updated_trigger(
    trgger: Trigger<CoreHealthChangedEvent>,
    mut query: Query<&mut Text, With<CoreHealthText>>,
    mut health: ResMut<CoreHealth>
) {
    health.0 = health.0.saturating_add_signed(trgger.event().change);

    for mut text in &mut query {
        text.sections[0].value = health.0.to_string();
    }
}

pub struct GameStatusPlugin;
impl Plugin for GameStatusPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<ScoreChangedEvent>()
            .add_event::<CashChangedEvent>()
            .add_event::<AvailableModulesChangedEvent>()
            .add_event::<CoreHealthChangedEvent>()
            .add_event::<WaveChangedEvent>()

            .init_resource::<Score>()
            .insert_resource(AvailableModules(20))
            .insert_resource(CoreHealth(10))
            .insert_resource(WaveInfo { current: 1, total: 5 })

            .observe(score_updated_trigger)
            .observe(cash_updated_trigger)
            .observe(available_modules_updated_trigger)
            .observe(core_health_updated_trigger)

            ;
    }
}
