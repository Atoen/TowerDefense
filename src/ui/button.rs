use crate::*;

#[derive(Component, Debug, Default, Clone, PartialEq)]
pub struct Button {
    pub text: Option<String>,
    pub image: Option<Handle<Image>>,
    pub hover_enlarge: bool,
}

#[derive(Component, Debug, Default, Clone, PartialEq)]
struct ButtonUi;

fn build_component(
    mut commands: Commands,
    query: Query<(Entity, &Button), Added<Button>>,
    assets: Res<AssetServer>
) {
    for (entity, button_source) in &query {
        let mut text = commands.spawn((
            UiLink::<ButtonUi>::path("Control/Image/Text"),
            UiLayout::window().pos(Rl((50., 120.))).anchor(Anchor::Center).pack::<Base>(),
            UiTextSize::new().size(Rh(60.0)),
            Pickable::IGNORE,
            UiAnimator::<Hover>::new().receiver(true),
            UiColor::<Base>::new(Color::BEVYPUNK_RED),
            UiColor::<Hover>::new(Color::BEVYPUNK_YELLOW),
        ));

        if let Some(source_text) = &button_source.text {
            text.insert(
                UiText2dBundle {
                    text: Text::from_section(source_text,
                        TextStyle {
                            font: assets.load(AssetPath::FONT_MEDIUM),
                            font_size: 60.0,
                            ..default()
                        }),
                    ..default()
                }
            );
        };

        let text_entity = text.id();

        commands.entity(entity).insert(
            UiTreeBundle::<ButtonUi>::from(UiTree::new2d("Button")),
        ).with_children(|ui| {
            let mut image = ui.spawn((
                UiLink::<ButtonUi>::path("Control/Image"),
                UiLayout::window_full().pack::<Base>(),
                Pickable::IGNORE,
                UiAnimator::<Hover>::new().receiver(true),
                UiColor::<Base>::new(Color::BEVYPUNK_RED),
                UiColor::<Hover>::new(Color::BEVYPUNK_YELLOW),
                UiLayout::boundary()
                    .pos1(Rl(if button_source.hover_enlarge { -5.0 } else { 0.0 }))
                    .pos2(Rl(if button_source.hover_enlarge { 105.0 } else { 100.0 }))
                    .pack::<Hover>(),
                UiLayoutController::default(),
            ));

            if let Some(source_image) = &button_source.image {
                image.insert((
                    UiImage2dBundle::from(source_image.clone()),
                    // ImageScaleMode::Sliced(TextureSlicer { border: BorderRect::square(20.), ..default() }),
                ));
            }
            let image_entity = image.id();


            ui.spawn((
                UiLink::<ButtonUi>::path("Control"),
                UiLayout::window_full().pack::<Base>(),
                UiZoneBundle::default(),
                UiAnimator::<Hover>::new().forward_speed(6.0).backward_speed(3.0),
                UiAnimatorPipe::<Hover>::new(vec![text_entity, image_entity]),
                OnHoverSetCursor::new(CursorIcon::Pointer),
                UiClickEmitter::new(entity),
            ));
        }).push_children(&[text_entity]);
    }
}

pub struct ButtonPlugin;
impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(UiGenericPlugin::<ButtonUi>::new())
            .add_systems(Update, build_component.before(UiSystems::Compute));
    }
}
