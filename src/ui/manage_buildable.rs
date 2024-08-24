use bevy::ecs::component::StorageType;
use button::Button;

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
                UiTreeBundle::<ManageBuidableUi>::from(UiTree::new2d("Manage"))
            ).with_children(|ui| {
    
                let row = UiLink::<ManageBuidableUi>::path("Row");
                ui.spawn((
                    row.clone(),
                    UiLayout::window_full().pack::<Base>()
                ));
    
                let width = Ab(60.0);
                let height = Ab(30.0);
                let spacing = Ab(20.0);
    
                let y = Rh(50.0) - height * 0.5;

                let total_width = width * 3.0 + spacing * 2.0;
                let initial_x = -total_width * 0.5 + Rw(50.0);
    
                for (index, button) in ManageButton::iter().enumerate() {

                    if buildable.is_standalone() && button == ManageButton::Upgrade{ continue }

                    let x = initial_x + (width + spacing) * index as f32;
                    ui.spawn((
                        row.add(button.to_string()),
                        UiLayout::window().pos((x, y)).size((width, height)).pack::<Base>(),
                        Button {
                            hover_enlarge: false,
                            image: None,
                            text: Some(button.to_string())
                        },
                        button
                    ));
                }
            });
        });
    }
}

#[derive(Component, Clone)]
struct ManageBuidableUi;


#[derive(Component, Display, EnumIter, PartialEq)]
enum ManageButton {
    Info,
    Upgrade,
    Sell
}

fn manage_button_clicked_system(
    mut events: EventReader<UiClickEvent>,
    mut commands: Commands,
    mut bottom_row: ResMut<BottomRowContent>,
    query: Query<&ManageButton>,
    mut grid: ResMut<GameGrid>,
    turret_query: Query<&TurretLevel>
) {
    for event in events.read() {
        if let Ok(button) = query.get(event.target) { 
            debug!("Clicked: {}", button);

            let BottomRowContent::Manage { entity, buildable } = *bottom_row else {
                error!("Bottom row state mismatch! Current mode {:?}, expected: BottomRowContent::Manage", bottom_row);
                return
            };

            match button {
                ManageButton::Upgrade => { 
                    let Ok(level) = turret_query.get(entity) else { continue };
                    if level.0 >= 4 { continue }

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
    }
}

pub struct ManageBuidablePlugin;
impl Plugin for ManageBuidablePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(UiGenericPlugin::<ManageBuidableUi>::new())

            .add_systems(PostUpdate, manage_button_clicked_system
                .distributive_run_if(on_event::<UiClickEvent>())
                .distributive_run_if(input_just_pressed(MouseButton::Left)))
            ;
    }
}