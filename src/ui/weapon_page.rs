use bevy::ecs::component::StorageType;
use button::Button;

use crate::*;

#[derive(Debug)]
pub struct WeaponPage;


impl Component for WeaponPage {
    const STORAGE_TYPE: StorageType = StorageType::Table;
    
    fn register_component_hooks(_hooks: &mut bevy::ecs::component::ComponentHooks) {
        _hooks.on_add(|mut world, entity, _| {
        
            let page = *world.resource::<WeaponSelectorPage>();

            let mut commands = world.commands();

            commands.entity(entity).insert(
                UiTreeBundle::<WeaponPageUi>::from(UiTree::new2d("Weapon Page"))
            ).with_children(|ui| {
                let elements_to_display: Vec<Buildable> = match page {
                    WeaponSelectorPage::Standard => vec![
                        Turret::PulseBlaster,
                        Turret::IonCannon,
                        Turret::SwarmTurret,
                        Turret::PlasmaRay,
                        Turret::CryoGenerator,
                        Turret::Tesla,
                        Turret::AcidSprayer,
                        Turret::FireThrower
                        ].into_iter().map(Buildable::Turret).collect(),
                        
                        WeaponSelectorPage::Advanced => vec![
                        Turret::SeekerLauncher,
                        Turret::Sentinel,
                        Turret::Recycler,
                        Turret::RailGun
                    ].into_iter().map(Buildable::Turret).collect(),
    
                    WeaponSelectorPage::Building => vec![
                        StandaloneBuildable::Module,
                        StandaloneBuildable::Consumable(Consumable::ProximityMine),
                        StandaloneBuildable::Consumable(Consumable::RotorBlades)
                    ].into_iter().map(Buildable::Standalone).collect()
                };
    
                let width = Ab(90.0);
                let height = Ab(30.0);
            
                let spacing = Ab(30.0);
                let fragments = elements_to_display.len();
                let total_width = width * fragments as f32 + spacing * (fragments - 1) as f32;
                
                let initial_x = -total_width * 0.5 + Rw(50.0);
                let y = Rh(50.0) - height * 0.5;
    
                let list = UiLink::<WeaponPageUi>::path("List");
                for (index, element) in elements_to_display.iter().enumerate() {
    
                    let x = initial_x + (width + spacing) * index as f32;
    
                    ui.spawn((
                        list.add(element.to_string()),
                        UiLayout::window().pos((x, y)).size((width, height)).pack::<Base>(),
                        Button {
                            hover_enlarge: true,
                            image: None,
                            text: Some(element.to_string())
                        },
                        BuildableButton(*element)
                    ));
                }
            });
        });
    }
}
#[derive(Component, Debug, Default, Clone, PartialEq)]
struct WeaponPageUi;

#[derive(Component)]
struct BuildableButton(Buildable);

fn buildable_clicked_system(
    mut events: EventReader<UiClickEvent>,
    mut commands: Commands,
    query: Query<&BuildableButton>,
    mut bottom_row: ResMut<BottomRowContent>
) {
    for event in events.read() {
        if let Ok(buildable_button) = query.get(event.target) {
            info!("Clicked: {}", buildable_button.0);

            *bottom_row = BottomRowContent::Build(buildable_button.0);
            commands.trigger(BottomRowContentChangedEvent(*bottom_row));
        }
    }
}

pub struct WeaponPagePlugin;
impl Plugin for WeaponPagePlugin {
    fn build(&self, app: &mut App) {
        app
        
            .add_plugins(UiGenericPlugin::<WeaponPageUi>::new())

            .add_systems(PostUpdate, buildable_clicked_system
                .distributive_run_if(on_event::<UiClickEvent>())
                .distributive_run_if(input_just_pressed(MouseButton::Left)))

            ;
    }
}
