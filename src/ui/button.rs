use crate::*;

#[derive(Component, Debug, Default, Clone, PartialEq)]
pub struct TurretPickerButton {
    pub text: String,
    pub image: Handle<Image>,
    pub hover_enlarge: bool,
    pub turret: TurretType
}

#[derive(Component, Debug, Default, Clone, PartialEq)]
struct ButtonUi;

fn build_component(
    mut commands: Commands,
    query: Query<(Entity, &TurretPickerButton), Added<TurretPickerButton>>,
    assets: Res<AssetServer>
) {
    for (entity, button_source) in &query {
        let text = commands.spawn((
            UiLink::<ButtonUi>::path("Control/Image/Text"),
            UiLayout::window().pos(Rl((50., 120.))).anchor(Anchor::Center).pack::<Base>(),
            UiText2dBundle {
                text: Text::from_section(&button_source.text,
                    TextStyle {
                        font: assets.load(AssetPath::FONT_MEDIUM),
                        font_size: 60.0,
                        ..default()
                    }),
                ..default()
            },
            UiTextSize::new().size(Rh(60.0)),
            Pickable::IGNORE,
            UiAnimator::<Hover>::new().receiver(true),
            UiColor::<Base>::new(Color::BEVYPUNK_RED),
            UiColor::<Hover>::new(Color::BEVYPUNK_YELLOW),
        )).id();

        commands.entity(entity).insert(
            UiTreeBundle::<ButtonUi>::from(UiTree::new2d("Button")),
        ).with_children(|ui| {
            let image = ui.spawn((
                UiLink::<ButtonUi>::path("Control/Image"),
                UiLayout::window_full().pack::<Base>(),
                UiImage2dBundle::from(button_source.image.clone()),
                // ImageScaleMode::Sliced(TextureSlicer { border: BorderRect::square(20.), ..default() }),
                Pickable::IGNORE,
                UiAnimator::<Hover>::new().receiver(true),
                UiColor::<Base>::new(Color::BEVYPUNK_RED),
                UiColor::<Hover>::new(Color::BEVYPUNK_YELLOW),
                UiLayout::boundary()
                    .pos1(Rl(if button_source.hover_enlarge { -5.0 } else { 0.0 }))
                    .pos2(Rl(if button_source.hover_enlarge { 105.0 } else { 100.0 }))
                    .pack::<Hover>(),
                UiLayoutController::default(),
            )).id();

            ui.spawn((
                UiLink::<ButtonUi>::path("Control"),
                UiLayout::window_full().pack::<Base>(),
                UiZoneBundle::default(),
                UiAnimator::<Hover>::new().forward_speed(6.0).backward_speed(3.0),
                UiAnimatorPipe::<Hover>::new(vec![text, image]),
                OnHoverSetCursor::new(CursorIcon::Pointer),
                UiClickEmitter::new(entity),
            ));
        }).insert(
            TextPipe { entity: vec![text]}
        ).push_children(&[text]);
    }
}


#[derive(Component)]
struct TextPipe {
    entity: Vec<Entity>
}
fn pipe_text(
    query: Query<(&TurretPickerButton, &TextPipe), Changed<TurretPickerButton>>,
    mut desti: Query<&mut Text>,
) {
    for (button, pipe) in &query {
        for e in &pipe.entity {
            if let Ok(mut text) = desti.get_mut(*e) {
                text.sections[0].value = button.text.clone();
            }
        }
    }
}

pub struct ButtonPlugin;
impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(UiGenericPlugin::<ButtonUi>::new())
            .add_systems(Update, pipe_text)
            .add_systems(Update, build_component.before(UiSystems::Compute));
    }
}