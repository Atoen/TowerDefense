use bevy::ecs::component::StorageType;

use crate::*;

#[derive(Debug, Clone)]
pub struct BuildInfo(pub Buildable);

impl Component for BuildInfo {
    const STORAGE_TYPE: StorageType = StorageType::Table;
    
    fn register_component_hooks(_hooks: &mut bevy::ecs::component::ComponentHooks) {
        _hooks.on_add(|mut world, entity, _| {
            let Some(BuildInfo(buildable)) = world.entity(entity).get::<BuildInfo>().cloned() else { return };

            let mut commands = world.commands();

            commands.entity(entity).insert((
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
                },
            )).with_children(|build_info| {
                if buildable.is_module() {

                    build_info.spawn((
                        NodeBundle {
                            style: Style {
                                padding: UiRect::axes(Val::Px(10.0), Val::Px(15.0)),
                                ..default()
                            },
                            ..default()
                        },
                        InfoButton::Cancel,
                        On::<Pointer<Click>>::run(button_clicked),
                        InteractionColors::BUTTON_DEFAULT
                    )).with_children(|button_container| {
                        button_container.spawn((
                            TextBundle::from_section("Done", text_style()),
                            Pickable::IGNORE
                        ));
                    });

                    return;
                }

                for button in InfoButton::iter() {
                    build_info.spawn((
                        NodeBundle {
                            style: Style {
                                padding: UiRect::axes(Val::Px(10.0), Val::Px(15.0)),
                                ..default()
                            },
                            ..default()
                        },
                        button,
                        On::<Pointer<Click>>::run(button_clicked),
                        InteractionColors::BUTTON_DEFAULT
                    )).with_children(|button_container| {
                        button_container.spawn((
                            TextBundle::from_section(button.to_string(), text_style()),
                            Pickable::IGNORE
                        ));
                    });
                }
            });
        });
    }
}


#[derive(Component, EnumIter, Display, Clone, Copy)]
enum InfoButton {
    Info,
    Build,
    Cancel
}

fn text_style() -> TextStyle {
    TextStyle {
        font_size: 18.0,
        color: Color::GRAY_500,
        ..default()
    }
}

fn button_clicked(
    listener: Listener<Pointer<Click>>,
    buttons: Query<&InfoButton>,
    mut commands: Commands,
    mut bottom_row: ResMut<BottomRowContent>,
    selected_cell: Res<SelectedCell>,
) {
    let target = listener.target;
    let Ok(button) = buttons.get(target) else { return };

    debug!("Clicked: {}", button);

    let BottomRowContent::Build(buildable) = *bottom_row else {
        error!("Bottom row state mismatch! Current mode {:?}, expected: BottomRowContent::Build", bottom_row);
        return
    };

    match button {
        InfoButton::Cancel => {
            *bottom_row = BottomRowContent::Selector;
            commands.trigger(BottomRowContentChangedEvent(*bottom_row));
        }

        InfoButton::Build => {
            let Some(cell_pos) = selected_cell.0 else { return };

            if !buildable.is_module() {
                commands.trigger(BuildEvent {
                    buildable,
                    cell_pos
                });
            }
        }
        
        InfoButton::Info => {
            info!("Displaying info about {}", buildable);
        }
    }
}

pub struct BuildInfoPlugin;
impl Plugin for BuildInfoPlugin {
    fn build(&self, app: &mut App) {
        app

            .add_event::<BuildEvent>();
    }
}
