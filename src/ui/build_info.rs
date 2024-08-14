use button::Button;

use crate::*;

#[derive(Component)]
pub struct BuildInfo(pub Buildable);

#[derive(Component, Clone)]
struct BuildInfoUi;

fn build_component(
    mut commands: Commands,
    query: Query<(Entity, &BuildInfo), Added<BuildInfo>>,
) {
    for (entity, build_info) in &query {
        commands.entity(entity).insert(
            UiTreeBundle::<BuildInfoUi>::from(UiTree::new2d("Build Info"))
        ).with_children(|ui| {

            let row = UiLink::<BuildInfoUi>::path("Row");
            ui.spawn((
                row.clone(),
                UiLayout::window_full().pack::<Base>()
            ));

            let width = Ab(60.0);
            let height = Ab(30.0);
            let spacing = Ab(20.0);

            let is_module = build_info.0 == Buildable::Module;
            let y = Rh(50.0) - height * 0.5;

            if is_module {
                ui.spawn((
                    row.add("Cancel"),
                    UiLayout::window().pos((-width * 0.5 + Rw(50.0), y)).size((width, height)).pack::<Base>(),
                    Button {
                        hover_enlarge: false,
                        image: None,
                        text: Some("Cancel".into())
                    },
                    InfoButton::Cancel
                ));

                return;
            }

            let total_width = width * 3.0 + spacing * 2.0;
            let initial_x = -total_width * 0.5 + Rw(50.0);

            for (index, button) in InfoButton::iter().enumerate() {
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
    }
}

#[derive(Component, EnumIter, Display)]
enum InfoButton {
    Info,
    Build,
    Cancel
}

#[derive(Event)]
pub struct InfoClosedEvent;

fn info_button_clicked_system(
    mut events: EventReader<UiClickEvent>,
    mut writer: EventWriter<InfoClosedEvent>,
    query: Query<&InfoButton>,
) {
    for event in events.read() {
        if let Ok(info_button) = query.get(event.target) {
            info!("Clicked: {}", info_button);

            if let InfoButton::Cancel = info_button {
                writer.send(InfoClosedEvent);
            }
        }
    }
}

pub struct BuildInfoPlugin;
impl Plugin for BuildInfoPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<InfoClosedEvent>()
            .add_plugins(UiGenericPlugin::<BuildInfoUi>::new())
            .add_systems(Update, build_component.before(UiSystems::Compute))
            .add_systems(PostUpdate, info_button_clicked_system
                .distributive_run_if(on_event::<UiClickEvent>())
                .distributive_run_if(input_just_pressed(MouseButton::Left)));
    }
}
