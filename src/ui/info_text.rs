
use bevy::ecs::component::StorageType;

use crate::*;

#[derive(Debug)]
pub struct InfoText;

impl Component for InfoText {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(_hooks: &mut bevy::ecs::component::ComponentHooks) {
        _hooks.on_add(|mut world, entity, _| {

            let mut commands = world.commands();

            commands.entity(entity).insert(
                NodeBundle {
                    z_index: ZIndex::Global(1200),
                    style: Style {
                        position_type: PositionType::Absolute,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::FlexEnd,
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(10.0),
                        width: Val::Percent(100.0),
                        height: Val::Px(200.0),
                        bottom: Val::Px(100.0),
                        padding: UiRect::bottom(Val::Px(5.0)),
                        ..default()
                    },
                    ..default()
                },
            ).with_children(|container| {
            
                container.spawn((
                    MessageContainer::default(),
                    NodeBundle {
                        z_index: ZIndex::Global(1202),
                        style: Style {
                            justify_content: JustifyContent::FlexEnd,
                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(10.0),
                            height: Val::Px(120.0),
                            overflow: Overflow {
                                y: OverflowAxis::Hidden,
                                ..default()
                            },
                            ..default()
                        },
                        ..default()
                    }
                ));
            
                container.spawn((
                    BuildableNameDisplay,
                    TextBundle::from_section(
                        "",
                        TextStyle { 
                            font_size: 18.0,
                            color: Color::WHITE,
                            ..default()
                        }
                    )
                ));
            });
        });
    }
}

#[derive(Event)]
pub struct InfoMessageAddedEvent(pub String);

#[derive(Component, Default)]
struct MessageContainer {
    displayed_messages: u32
}

#[derive(Component)]
struct BuildableNameDisplay;

#[derive(Component)]
struct InfoMessage {
    decay_timer: Timer
}

fn add_message_trigger(
    trigger: Trigger<InfoMessageAddedEvent>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut MessageContainer)>
) {

    let Ok((entity, mut container)) = query.get_single_mut() else {
        warn!("Unable to find info message box!");
        return
    };

    let message_node = commands.spawn((
        InfoMessage { decay_timer: Timer::from_seconds(5.0, TimerMode::Once) },
        TextBundle {
            text: Text::from_section(
                trigger.event().0.clone(),
                TextStyle {
                    font_size: 20.0,
                    color: Color::WHITE,
                    ..default()
                }
            ),
            ..default()
        }.with_text_justify(JustifyText::Center)
        
    )).id();

    commands.entity(entity).add_child(message_node);

    container.displayed_messages += 1;
}

fn decay_message_system(
    time: Res<Time>,
    mut commands: Commands,
    mut messages: Query<(Entity, &mut InfoMessage, &mut Text)>,
    mut container: Query<(Entity, &mut MessageContainer)>
) {
    
    let Ok((container_entity, mut container)) = container.get_single_mut() else {
        warn!("Unable to find info message box!");
        return
    };

    let delta = time.delta();
    for (message_entity, mut message, mut text) in &mut messages{
        message.decay_timer.tick(delta);

        text.sections[0].style.color.set_alpha(message.decay_timer.fraction_remaining());

        if message.decay_timer.finished() {

            commands.entity(container_entity).remove_children(&[message_entity]);

            commands.entity(message_entity).despawn();
            container.displayed_messages -= 1;
        }
    }
}

fn display_buildable_name_trigger(
    _trigger: Trigger<BottomRowContentChangedEvent>,
    mut display: Query<&mut Text, With<BuildableNameDisplay>>,
    bottom_row: Res<BottomRowContent>
) {
    let Ok(mut text) = display.get_single_mut() else {
        warn!("Unable to find BuildableNameDisplay!");
        return
    };

    match &*bottom_row {
        BottomRowContent::Selector => { 
            text.sections[0].value.clear();
        }
        BottomRowContent::Build(buildable) | BottomRowContent::Manage { buildable, .. } => {
            text.sections[0].value = buildable.to_string();
        }
    }
}

pub struct InfoTextPlugin;
impl Plugin for InfoTextPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<InfoMessageAddedEvent>()

            .add_systems(Update, decay_message_system
                .run_if(in_state(AppState::InGame)))

            .observe(display_buildable_name_trigger)
            .observe(add_message_trigger)

            ;
    }
}
