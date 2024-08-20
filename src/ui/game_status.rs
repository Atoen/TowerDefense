use bevy::ecs::component::StorageType;
use button::Button;

use crate::*;

#[derive(Debug)]
pub struct GameStatus;

impl Component for GameStatus {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(_hooks: &mut bevy::ecs::component::ComponentHooks) {
        _hooks.on_add(|mut world, entity, _| {

            let cash = world.resource::<GameCash>().0;
            let pause = world.resource::<UiTextures>().pause.clone();

            let mut commands = world.commands();

            commands.entity(entity).insert(
                UiTreeBundle::<GameStatusUi>::from(UiTree::new2d("Game Status"))
            ).with_children(|ui| {
                let row = UiLink::<GameStatusUi>::path("Row");
                ui.spawn((
                    row.clone(),
                    UiLayout::window_full().pack::<Base>(),
                    Pickable::IGNORE,
                ));
    
                ui.spawn((
                    row.add("Pause Button"),
                    UiLayout::window().pos(5.).anchor(Anchor::TopLeft).size((20.0, 20.0)).pack::<Base>(),
                    Button {
                        hover_enlarge: false,
                        text: None,
                        image: Some(pause)
                    },
                    StatusButton::Pause
                ));
    
                ui.spawn((
                    row.add("Cash"),
                    UiLayout::window().x(Rl(50.0)).anchor(Anchor::TopCenter).size((100.0, 40.0)).pack::<Base>(),
                    UiText2dBundle {
                         text: Text::from_sections([
                             TextSection {
                                 value: cash.to_string(),
                                 style: TextStyle {
                                     font_size: 50.0,
                                     color: Color::GRAY_200,
                                     ..default()
                                 }
                             },
                             TextSection {
                                 value: "$".into(),
                                 style: TextStyle {
                                     font_size: 50.0,
                                     color: Color::GRAY_200,
                                     ..default()
                                 }
                             }
                         ]),
                         ..default()
                     },
                    GameCashDisplay
                ));
    
                ui.spawn((
                    row.add("Start Round"),
                    UiLayout::window().x(Rl(100.0)).anchor(Anchor::TopRight).size((150.0, 40.0)).pack::<Base>(),
                    Button {
                        hover_enlarge: true,
                        text: Some("Start Round".into()),
                        image: None
                    },
                    StatusButton::StartRound
                ));
            });
        });
    }
}

#[derive(Component, Debug, Default, Clone, PartialEq)]
struct GameStatusUi;

#[derive(Resource)]
pub struct GameCash(pub u32);

#[derive(Component)]
enum StatusButton {
    Pause,
    StartRound
}

#[derive(Component)]
struct GameCashDisplay;

#[derive(Event)]
pub struct CashChangedEvent {
    pub change: i32
}

fn toggle_pause_system(
    mut events: EventReader<UiClickEvent>,
    query: Query<&StatusButton>,
    pause_state: Res<State<PausedState>>,
    mut next_pause_state: ResMut<NextState<PausedState>>,
    mut next_game_state: ResMut<NextState<GameState>>
) {
    for event in events.read() {
        if let Ok(button) = query.get(event.target){    

            match button {
                StatusButton::Pause => {
                    let next_pause = match pause_state.get() {
                        PausedState::Running => PausedState::Paused,
                        PausedState::Paused => PausedState::Running
                    };
                
                    info!("Current pause state: {}", next_pause);
                
                    next_pause_state.set(next_pause);
                }
                StatusButton::StartRound => {
                    info!("Starting round");
                    next_game_state.set(GameState::AttackWave);
                }
            }
        }
    }
}

fn game_cash_updated_system(
    mut events: EventReader<CashChangedEvent>,
    mut query: Query<&mut Text, With<GameCashDisplay>>,
    mut game_cash: Option<ResMut<GameCash>>
) {
    for event in events.read() {
        let Some(cash) = game_cash.as_mut() else {
            return;
        };

        cash.0 = cash.0.wrapping_add_signed(event.change);

        for mut text in &mut query {
            text.sections[0].value = cash.0.to_string();
        }
    }
}

pub struct GameStatusPlugin;
impl Plugin for GameStatusPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<CashChangedEvent>()

            .add_plugins(UiGenericPlugin::<GameStatusUi>::new())

            .add_systems(PostUpdate, toggle_pause_system
                .distributive_run_if(on_event::<UiClickEvent>())
                .distributive_run_if(input_just_pressed(MouseButton::Left)))

            .add_systems(Update, game_cash_updated_system
                .run_if(on_event::<CashChangedEvent>())
            );
    }
}
