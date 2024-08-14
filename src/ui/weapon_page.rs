use button::Button;

use crate::*;

#[derive(Component, Debug, Default, Clone, PartialEq)]
pub struct WeaponPage;

#[derive(Component, Debug, Default, Clone, PartialEq)]
struct WeaponPageUi;

fn build_component(
    mut commands: Commands,
    query: Query<Entity, Added<WeaponPage>>,
    selected_page: Res<State<PageState>>,
) {
    for entity in &query {
        commands.entity(entity).insert(
            UiTreeBundle::<WeaponPageUi>::from(UiTree::new2d("Weapon Page"))
        ).with_children(|ui| {
            let elements_to_display = match selected_page.get() {
                PageState::Standard => vec![
                    Buildable::PulseBlaster,  
                    Buildable::IonCannon,     
                    Buildable::SwarmTurret,   
                    Buildable::PlasmaRay,          
                    Buildable::CryoGenerator, 
                    Buildable::Tesla,         
                    Buildable::SeekerLauncher,
                    Buildable::AcidSprayer,    
                    Buildable::FireThrower,       
                ],
        
                PageState::Advanced => vec![
                    Buildable::Sentinel,
                    Buildable::CyberOro,
                    Buildable::RailGun,
                ],
        
                PageState::Building => vec![
                    Buildable::Module,
                    Buildable::Mine,
                    Buildable::Blades,
                ],
            };

            let width = Ab(60.0);
            let height = Ab(30.0);
        
            let spacing = Ab(20.0);
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
    }
}

#[derive(Event)]
pub struct BuildableSelecedEvent(Buildable);

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash, Display)]

pub enum SelectedBuildabeState {
    #[default]
    None,
    Some(Buildable)
}

#[derive(Component)]
struct BuildableButton(Buildable);

#[derive(Display, EnumIter, EnumCount, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Buildable {
    // Standard Turrets
    PulseBlaster,  
    IonCannon,     
    SwarmTurret,   
    PlasmaRay,          
    CryoGenerator, 
    Tesla,         
    SeekerLauncher,
    AcidSprayer,    
    FireThrower,       

    // Advanced Turrets
    Sentinel,
    CyberOro,
    RailGun,

    // Buildings
    Module,
    Mine,
    Blades,
}

fn buildable_clicked_system(
    mut events: EventReader<UiClickEvent>,
    mut writer: EventWriter<BuildableSelecedEvent>,
    query: Query<&BuildableButton>,
    mut next_state: ResMut<NextState<SelectedBuildabeState>>
) {
    for event in events.read() {
        if let Ok(buildable_button) = query.get(event.target) {
            info!("Clicked: {}", buildable_button.0);

            writer.send(BuildableSelecedEvent(buildable_button.0));
            next_state.set(SelectedBuildabeState::Some(buildable_button.0));
        }
    }
}

pub struct WeaponPagePlugin;
impl Plugin for WeaponPagePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<BuildableSelecedEvent>()
            .add_systems(Update, build_component.before(UiSystems::Compute))
            .add_plugins(UiGenericPlugin::<WeaponPageUi>::new())
            .add_systems(PostUpdate, buildable_clicked_system
                .distributive_run_if(on_event::<UiClickEvent>())
                .distributive_run_if(input_just_pressed(MouseButton::Left)))
            .init_state::<SelectedBuildabeState>();
    }
}
