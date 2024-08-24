use crate::*;

#[derive(Component, Debug, Default, Clone, PartialEq)]
pub struct MenuButton {
    pub text: String,
}

#[derive(Component, Debug, Default, Clone, PartialEq)]
struct MenuButtonUi;

fn build_component(
    mut commands: Commands,
    query: Query<(Entity, &MenuButton), Added<MenuButton>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    for (entity, button_source) in &query {

        commands.entity(entity).insert(
            UiTreeBundle::<MenuButtonUi>::from(UiTree::new2d("Menu Button"))
        ).with_children(|ui| {
            
            let background = ui.spawn((
                UiLink::<MenuButtonUi>::path("Control/Image"),

                UiLayout::window_full().pack::<Base>(),
                UiMaterial2dBundle {
                    material: materials.add(Color::BLACK.with_alpha(0.5)),
                    ..default()
                },

                Pickable::IGNORE,
                UiAnimator::<Hover>::new().receiver(true),

                UiColor::<Base>::new(Color::GRAY_400),
                UiColor::<Hover>::new(Color::WHITE),

                UiLayout::window_full().x(Rl(-10.0)).pack::<Hover>(),
                UiLayoutController::default(),
            )).id();

            let text = ui.spawn((
                UiLink::<MenuButtonUi>::path("Control/Image/Text"),
                UiLayout::window().pos((Rh(40.0), Rl(50.0))).anchor(Anchor::CenterLeft).pack::<Base>(),

                UiText2dBundle {
                    text: Text::from_section(&button_source.text,
                        TextStyle {
                            font_size: 60.0,
                            ..default()
                        }),
                    ..default()
                },

                Pickable::IGNORE,
                UiAnimator::<Hover>::new().receiver(true),

                UiColor::<Base>::new(Color::GRAY_500),
                UiColor::<Hover>::new(Color::WHITE),
            )).id();

            ui.spawn((
                UiLink::<MenuButtonUi>::path("Control"),

                UiLayout::window_full().pack::<Base>(),

                UiZoneBundle::default(),

                UiAnimator::<Hover>::new().forward_speed(5.0).backward_speed(1.0),
                UiAnimatorPipe::<Hover>::new(vec![text, background]),

                OnHoverSetCursor::new(CursorIcon::Pointer),
                UiClickEmitter::new(entity)
            ));
        });
    }
}

pub struct MenuButtonPlugin;
impl Plugin for MenuButtonPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(UiGenericPlugin::<MenuButtonUi>::new())
            .add_systems(Update, build_component
                .before(UiSystems::Compute));
    }
}
