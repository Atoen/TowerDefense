use bevy::ecs::component::StorageType;

use crate::*;

#[derive(Debug, Clone)]
pub struct ManageBuidable {
    pub buildable: Buildable,
    pub entity: Entity
}

impl Component for ManageBuidable {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(_hooks: &mut bevy::ecs::component::ComponentHooks) {
        _hooks.on_add(|mut world, entity, _| {
            let Some(ManageBuidable { buildable, .. } ) = world.entity(entity).get::<ManageBuidable>().cloned() else { return };

            let mut commands = world.commands();

            commands.entity(entity).insert(
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
                    background_color: Color::BLACK.with_alpha(0.7).into(),
                    z_index: ZIndex::Global(101),
                    ..default()
                }
            ).with_children(|manage_buildable| {
                for button in ManageButton::iter() {

                    if buildable.is_standalone() && button == ManageButton::Upgrade {
                        manage_buildable.spawn(TextBundle::from_section(
                            button.to_string(),
                            TextStyle {
                                color: Color::NONE,
                                ..text_style()
                            })
                        );

                        continue;
                    }

                    manage_buildable.spawn((
                        TextBundle::from_section(button.to_string(), text_style()),
                        button,
                        On::<Pointer<Click>>::run(button_clicked)
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
        color: Color::WHITE,
        ..default()
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