use button::Button;

use crate::*;

#[derive(Component, Debug, Default, Clone, PartialEq)]
pub struct GameStatus;

#[derive(Component, Debug, Default, Clone, PartialEq)]
struct GameStatusUi;

fn build_component(
    mut commands: Commands,
    query: Query<Entity, Added<GameStatus>>,
    assets: Res<AssetServer>,
    game_cash: Res<GameCash>
) {
    for entity in &query {
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
                    image: Some(assets.load(AssetPath::PAUSE))
                },
                PasueButton
            ));

            ui.spawn((
                row.add("Cash"),
                UiLayout::window().x(Rl(50.0)).anchor(Anchor::TopCenter).size((100.0, 40.0)).pack::<Base>(),
                UiText2dBundle {
                     text: Text::from_sections([
                         TextSection {
                             value: game_cash.0.to_string(),
                             style: TextStyle {
                                 font_size: 50.0,
                                 color: Color::GRAY_200,
                                 font: assets.load(AssetPath::FONT_REGULAR)
                             }
                         },
                         TextSection {
                             value: "$".into(),
                             style: TextStyle {
                                 font_size: 50.0,
                                 color: Color::GRAY_200,
                                 font: assets.load(AssetPath::FONT_REGULAR)
                             }
                         }
                     ]),
                     ..default()
                 },
                GameCashDisplay
            ));

            ui.spawn((
                row.add("Start Round"),
                UiLayout::window().x(Rl(100.0)).anchor(Anchor::TopRight).size((100.0, 40.0)).pack::<Base>(),
                Button {
                    hover_enlarge: true,
                    text: Some("Start Round".into()),
                    image: None
                }
            ));
        });
    }
}

#[derive(Resource)]
pub struct GameCash(pub u32);

#[derive(Component)]
struct PasueButton;

#[derive(Component)]
struct GameCashDisplay;

#[derive(Event)]
pub struct CashChangedEvent {
    pub change: i32
}

fn toggle_pause_system(
    mut events: EventReader<UiClickEvent>,
    query: Query<&PasueButton>,
    state: Res<State<PausedState>>,
    mut next_state: ResMut<NextState<PausedState>>
) {
    for event in events.read() {
        if query.get(event.target).is_ok() {            
            let next_pause_state = match state.get() {
                PausedState::Running => PausedState::Paused,
                PausedState::Paused => PausedState::Running
            };
        
            info!("Current game state: {}", next_pause_state);
        
            next_state.set(next_pause_state);
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
            .add_systems(Update, build_component.before(UiSystems::Compute))
            .add_systems(PostUpdate, toggle_pause_system
                .distributive_run_if(on_event::<UiClickEvent>())
                .distributive_run_if(input_just_pressed(MouseButton::Left)))
            .add_systems(Update, game_cash_updated_system
                .run_if(on_event::<CashChangedEvent>())
            );
    }
}
