use bevy::ecs::component::StorageType;
use ui::DISABLED_BUTTON_COLOR;

use crate::*;

#[derive(Debug, Clone, Copy)]
pub struct ManageBuidable {
    pub buildable: Buildable,
    pub entity: Entity
}

impl Component for ManageBuidable {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(_hooks: &mut bevy::ecs::component::ComponentHooks) {
        _hooks.on_add(|mut world, ui_entity, _| {
            let Some(ManageBuidable { buildable, entity: buildable_entity }) = world.entity(ui_entity).get::<ManageBuidable>().copied() else { return };

            let level = world.entity(buildable_entity).get::<TurretLevel>().copied();
            let targeting_turret_and_entity = world.entity(buildable_entity).get::<Children>()
                .and_then(|children| {
                    children.iter().find_map(|&child| {
                        world.entity(child).get::<TargetingTurret>()
                            .cloned()
                            .map(|turret| (turret, child))
                    })
                });

            let mut commands = world.commands();

            commands.entity(ui_entity).insert(
                NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(100.0),
                        height: Val::Px(100.0),
                        bottom: Val::Px(0.0),
            
                        display: Display::Flex,
                        column_gap: Val::Px(30.0),
            
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::BLACK.with_alpha(0.5).into(),
                    z_index: ZIndex::Global(101),
                    ..default()
                }
            ).with_children(|manage_buildable| {
                if let Some((targeting_turret, entity)) = targeting_turret_and_entity {
                    manage_buildable.spawn((
                        NodeBundle {
                            style: Style {
                                position_type: PositionType::Absolute,
                                left: Val::Px(30.0),
                                display: Display::Flex,
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(30.0),
                                padding: UiRect::axes(Val::Px(10.0), Val::Px(15.0)),
                                ..default()
                            },
                            ..default()
                        },
                        InteractionColors::BUTTON_DEFAULT,
                        On::<Pointer<Click>>::run(targeting_button_clicked)
                    )).with_children(|button_container| {    
                        button_container.spawn((
                            TextBundle::from_sections([
                                TextSection {
                                    value: "Targeting: ".into(),
                                    style: text_style(),
                                },
                                TextSection {
                                    value: targeting_turret.mode.to_string(),
                                    style: text_style(),
                                }
                            ]),
                            TargetingModeButton(entity),
                            Pickable::IGNORE
                        ));
                    });
                }
            
                let is_standalone = buildable.is_standalone();
                let is_max_level = level.is_some_and(|l| l.0 == MAX_TURRET_LEVEL);
        
                for button in ManageButton::iter() {

                    let mut entity_commands = manage_buildable.spawn((
                        NodeBundle {
                            style: Style {
                                padding: UiRect::axes(Val::Px(10.0), Val::Px(15.0)),
                                ..default()
                            },
                            ..default()
                        },
                        button
                    ));

                    let should_skip_interactivity = button == ManageButton::Upgrade && (is_standalone || is_max_level);
                    if !should_skip_interactivity {
                        entity_commands.insert((
                            InteractionColors::BUTTON_DEFAULT,
                            On::<Pointer<Click>>::run(button_clicked)
                        ));
                    }
                    
                    entity_commands.with_children(|button_container| {

                        if button == ManageButton::Upgrade {

                            let color = if is_standalone {
                                Color::NONE
                            } else if is_max_level {
                                DISABLED_BUTTON_COLOR
                            } else {
                                Color::GRAY_500
                            };

                            button_container.spawn((
                                TextBundle::from_section(button.to_string(), TextStyle {
                                    color,
                                    ..text_style()
                                }),
                                Pickable::IGNORE,
                                UpgradeButtonText
                            ));
                        } else {
                            button_container.spawn((
                                button,
                                TextBundle::from_section(button.to_string(), text_style()),
                                Pickable::IGNORE
                            ));
                        }
                    });
                }
            });
        });
    }
}

#[derive(Component, Display, EnumIter, PartialEq, Clone, Copy)]
enum ManageButton {
    Info,
    Upgrade,
    Sell
}

#[derive(Component)]
struct TargetingModeButton(Entity);

#[derive(Component)]
struct UpgradeButtonText;

fn text_style() -> TextStyle {
    TextStyle {
        font_size: 18.0,
        color: Color::GRAY_500,
        ..default()
    }
}

fn button_clicked(
    listener: Listener<Pointer<Click>>,
    buttons: Query<&ManageButton>,
    bottom_row: Res<BottomRowContent>,
    turret_query: Query<&TurretLevel>,
    mut commands: Commands,
    mut grid: ResMut<GameGrid>
) {
    let target = listener.target;
    let Ok(button) = buttons.get(target) else { return };

    debug!("Clicked: {}", button);

    let BottomRowContent::Manage { entity, buildable } = *bottom_row else {
        error!("Bottom row state mismatch! Current mode {:?}, expected: BottomRowContent::Manage", bottom_row);
        return
    };

    match button {
        ManageButton::Upgrade => { 
            let Ok(level) = turret_query.get(entity) else { return };
            if level.0 >= 4 { return }

            commands.entity(entity).insert(RaiseTurretLevel);
        }

        ManageButton::Sell => { 
            if let Some(cell) = grid.find_cell_mut(entity) {
                cell.remove_buildable(buildable);
            } else {
                warn!("Unable to find cell with specified entity!");
            }

            commands.entity(entity).despawn_recursive();
            commands.trigger(BuildableRemovedEvent(entity));
        }

        ManageButton::Info => {
            info!("Displaying info about {}", buildable);
        }
    }
}

fn targeting_button_clicked(
    button: Query<&TargetingModeButton>,
    mut turrets: Query<&mut TargetingTurret>,
    mut query: Query<&mut Text, With<TargetingModeButton>>
) {
    let Ok(button) = button.get_single() else { return };
    let Ok(mut turret) = turrets.get_mut(button.0) else { return };
    let Ok(mut text) = query.get_single_mut() else { return };

    let targetting_mode = turret.mode;

    turret.mode = targetting_mode.next();
    text.sections[1].value = turret.mode.to_string();
}

fn turret_upgraded_trigger(
    trigger: Trigger<TurretUpgradedEvent>,
    bottom_row: Res<BottomRowContent>,
    level: Query<&TurretLevel>,
    mut buttons: Query<(Entity, &ManageButton)>,
    mut upgrde_text: Query<&mut Text, With<UpgradeButtonText>>,
    mut commands: Commands
) {
    let turret_entity = trigger.event().entity;
    let BottomRowContent::Manage { entity, .. } = *bottom_row else { return };

    if turret_entity != entity {
        warn!("Not matching!");
        return
    }

    let Ok(level) = level.get(turret_entity) else { return };
    if level.0 < MAX_TURRET_LEVEL {
        return
    }

    let Some((button_entity, _)) = buttons.iter_mut()
        .find(|query| matches!(query.1, ManageButton::Upgrade)) else {
        warn!("Unable to find upgrade button!");
        return
    };

    commands.entity(button_entity).remove::<(
        On<Pointer<Click>>,
        InteractionColors
    )>();

    let Ok(mut text) = upgrde_text.get_single_mut() else {
        warn!("Unable to find upgrade text!");
        return
    };

    for section in text.sections.iter_mut() {
        section.style.color = DISABLED_BUTTON_COLOR;
    }
}

fn buildable_removed_trigger(
    trigger: Trigger<BuildableRemovedEvent>,
    mut bottom_row: ResMut<BottomRowContent>,
    mut commands: Commands
) {
    let removed_entity = trigger.event().0;

    if let BottomRowContent::Manage { entity, .. } = *bottom_row {
        if entity == removed_entity {
            *bottom_row = BottomRowContent::Selector;
            commands.trigger(BottomRowContentChangedEvent(*bottom_row))
        }
    }
}

pub struct ManageBuildablePlugin;

impl Plugin for ManageBuildablePlugin {
    fn build(&self, app: &mut App) {
        app
            .observe(turret_upgraded_trigger)
            .observe(buildable_removed_trigger)
            ;
    }
}
