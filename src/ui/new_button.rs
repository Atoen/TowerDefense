use bevy::ecs::component::StorageType;

use crate::*;

#[derive(Debug, Clone)]
pub struct NewButton {
    pub text: Option<String>,
    pub image: Option<Handle<Image>>
}

impl Component for NewButton {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(_hooks: &mut bevy::ecs::component::ComponentHooks) {
        _hooks.on_add(|mut world, entity, _| {

            let Some(source) = world.entity(entity).get::<NewButton>().cloned() else { return };

            let mut commands = world.commands();

            commands.entity(entity).insert(ButtonBundle {
                style: Style {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            });

            if let Some(text) = source.text {
                commands.entity(entity).with_children(|parent| {

                    parent.spawn((
                        TextBundle::from_section(
                            text,
                            TextStyle {
                                font_size: 20.0,
                                color: Color::WHITE,
                                ..default()
                            }
                        ),
                        Pickable::IGNORE
                    ));
                });
            }

            if let Some(image_handle) = source.image {
                commands.entity(entity).with_children(|parent| {

                    parent.spawn((
                        ImageBundle {
                            image: image_handle.into(),
                            ..default()
                        },
                        Pickable::IGNORE
                    ));
                });
            }
        });
    }
}