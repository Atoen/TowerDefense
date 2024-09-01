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
                for button in ManageButton::iter() {

                    let is_standalone = buildable.is_standalone();
                    let is_max_level = level.is_some_and(|l| l.0 == MAX_TURRET_LEVEL);

                    if button == ManageButton::Upgrade && (is_standalone || is_max_level) {

                        let color = if is_standalone {
                            Color::NONE
                        } else if is_max_level {
                            DISABLED_BUTTON_COLOR
                        } else {
                            return
                        };

                        manage_buildable.spawn(TextBundle::from_section(
                            button.to_string(),
                            TextStyle {
                                color,
                                ..text_style()
                            })
                        );

                        continue;
                    }

                    manage_buildable.spawn((
                        TextBundle::from_section(button.to_string(), text_style()),
                        button,
                        On::<Pointer<Click>>::run(button_clicked),
                        InteractionColors::BUTTON_DEFAULT
                    ));
                }
            });
        });
    }
}

#[derive(Component, Display, EnumIter, PartialEq)]
enum ManageButton {
    Info,
    Upgrade,
    Sell
}

fn text_style() -> TextStyle {
    TextStyle {
        font_size: 18.0,
        color: Color::GRAY_500,
        ..default()
    }
}

fn turret_upgraded_trigger(
    trigger: Trigger<TurretUpgradedEvent>,
    bottom_row: Res<BottomRowContent>,
    level: Query<&TurretLevel>,
    mut buttons: Query<(Entity, &ManageButton, &mut Text)>,
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

    let Some((button_entity, _, mut text)) = buttons.iter_mut()
        .find(|query| matches!(query.1, ManageButton::Upgrade)) else {
        warn!("Unable to find upgrade button!");
        return
    };

    commands.entity(button_entity).remove::<(
        On<Pointer<Click>>,
        InteractionColors
    )>();

    for section in text.sections.iter_mut() {
        section.style.color = DISABLED_BUTTON_COLOR;
    }
}

fn button_clicked(
    listener: Listener<Pointer<Click>>,
    buttons: Query<&ManageButton>,
    mut commands: Commands,
    mut bottom_row: ResMut<BottomRowContent>,
    mut grid: ResMut<GameGrid>,
    turret_query: Query<&TurretLevel>
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

            *bottom_row = BottomRowContent::Selector;
            commands.trigger(BottomRowContentChangedEvent(*bottom_row));
        }

        ManageButton::Info => {
            info!("Displaying info about {}", buildable);
        }
    }
}

pub struct ManageBuildablePlugin;

impl Plugin for ManageBuildablePlugin {
    fn build(&self, app: &mut App) {
        app
            .observe(turret_upgraded_trigger)
            ;
    }
}
