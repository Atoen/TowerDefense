use bevy::ecs::component::StorageType;

use crate::*;

#[derive(Debug, Clone)]
pub struct Button {
    pub text: Option<String>,
    pub image: Option<Handle<Image>>,
    pub hover_enlarge: bool,
}

impl Component for Button {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(_hooks: &mut bevy::ecs::component::ComponentHooks) {
        _hooks.on_add(|mut world, entity, _| {

            let Some(source) = world.entity(entity).get::<Button>().cloned() else { return };

            let mut commands = world.commands();

            commands.entity(entity).insert(
                UiTreeBundle::<ButtonUi>::from(UiTree::new2d("Button"))            
            ).with_children(|ui| {
                let backround = source.image.as_ref().map(|image| ui.spawn((
                            UiLink::<ButtonUi>::path("Control/Image"),
    
                            UiLayout::window_full().pack::<Base>(),
                            Pickable::IGNORE,
                            UiAnimator::<Hover>::new().receiver(true),
    
                            UiColor::<Base>::new(Color::GRAY_500),
                            UiColor::<Hover>::new(Color::WHITE),
    
                            UiImage2dBundle::from(image.clone()),
                        )).id());
    
                let text = source.text.as_ref().map(|text| ui.spawn((
                            UiLink::<ButtonUi>::path("Control/Image/Text"),
                            UiLayout::window().pos(Rl(50.0)).anchor(Anchor::Center).pack::<Base>(),
                            UiTextSize::new().size(Rh(60.0)),
                            Pickable::IGNORE,
                            UiAnimator::<Hover>::new().receiver(true),
                            UiColor::<Base>::new(Color::GRAY_500),
                            UiColor::<Hover>::new(Color::WHITE),
                            UiText2dBundle {
                                text: Text::from_section(text,
                                    TextStyle {
                                        font_size: 60.0,
                                        ..default()
                                    }),
                                ..default()
                            }
                        )).id());
    
                let ui_animator_pipe = match (backround, text) {
                    (Some(a), Some(b)) => Some(vec![a, b]),
                    (Some(a), None) =>  Some(vec![a]),
                    (None, Some(b)) =>  Some(vec![b]),
                    _ => None,
                };            
    
                let mut button = ui.spawn((
                    UiLink::<ButtonUi>::path("Control"),
    
                    UiLayout::window_full().pack::<Base>(),
    
                    UiZoneBundle::default(),
    
                    UiAnimator::<Hover>::new().forward_speed(5.0).backward_speed(1.0),
                    OnHoverSetCursor::new(CursorIcon::Pointer),
                    UiLayout::boundary()
                        .pos1(Rl(if source.hover_enlarge { -5.0 } else { 0.0 }))
                        .pos2(Rl(if source.hover_enlarge { 105.0 } else { 100.0 }))
                        .pack::<Hover>(),
                    UiLayoutController::default(),
                    UiClickEmitter::new(entity)
                ));
    
                if let Some(animator) = ui_animator_pipe {
                    button.insert(UiAnimatorPipe::<Hover>::new(animator));
                }
            });
        });
    }
}

#[derive(Component, Debug, Default, Clone, PartialEq)]
struct ButtonUi;

pub struct ButtonPlugin;
impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(UiGenericPlugin::<ButtonUi>::new());
    }
}
