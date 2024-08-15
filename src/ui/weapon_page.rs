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
            let elements_to_display: Vec<Buildable> = match selected_page.get() {
                PageState::Standard => vec![
                    Turret::PulseBlaster,
                    Turret::IonCannon,
                    Turret::SwarmTurret,
                    Turret::PlasmaRay,
                    Turret::CryoGenerator,
                    Turret::Tesla,
                    Turret::SeekerLauncher,
                    Turret::AcidSprayer,
                    Turret::FireThrower
                ].into_iter().map(Buildable::Turret).collect(),

                PageState::Advanced => vec![
                    Turret::Sentinel,
                    Turret::CyberOro,
                    Turret::RailGun
                ].into_iter().map(Buildable::Turret).collect(),

                PageState::Building => vec![
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
    }
}

#[derive(Event)]
pub struct BuildableSelecedEvent;

#[derive(Component)]
struct BuildableButton(Buildable);

fn buildable_clicked_system(
    mut events: EventReader<UiClickEvent>,
    mut writer: EventWriter<BuildableSelecedEvent>,
    query: Query<&BuildableButton>,
    mut selected_buildable: ResMut<SelectedBuildable>
) {
    for event in events.read() {
        if let Ok(buildable_button) = query.get(event.target) {
            info!("Clicked: {}", buildable_button.0);

            writer.send(BuildableSelecedEvent);
            selected_buildable.0 = Some(buildable_button.0);
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

            .init_resource::<SelectedBuildable>();
    }
}
