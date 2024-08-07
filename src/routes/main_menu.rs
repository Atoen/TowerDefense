use ui::button::Button;

use crate::*;

#[derive(Component, Debug, Default, Clone, PartialEq)]
pub struct MainMenuRoute;


fn build_route(
    mut commands: Commands,
    assets: Res<AssetServer>,
    query: Query<Entity, Added<MainMenuRoute>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    for route_entity in &query {
        commands.entity(route_entity).insert(
            SpatialBundle::default()
        ).with_children(|route| {

            route.spawn((
                UiTreeBundle::<MainUi>::from(UiTree::new2d("Main Menu")),
                MovableByCamera
            )).with_children(|ui| {

                let root = UiLink::<MainUi>::path("Root");
                ui.spawn((
                    root.clone(),
                    UiLayout::window_full().pack::<Base>()
                ));

                ui.spawn((
                    root.add("Background"),
                    UiLayout::solid().size((1920.0, 1080.0)).scaling(Scaling::Fill).pack::<Base>(),
                    UiMaterial2dBundle {
                        material: materials.add(Color::srgba(0.5, 0.2, 0.2, 0.5)),
                        ..default()
                    }
                ));

                let board = root.add("Solid");
                ui.spawn((
                    board.clone(),
                    UiLayout::solid().size((881.0, 1600.0)).align_x(-0.74).pack::<Base>(), // Just different layout type that preserves aspect ratio
                ));

                let board = board.add("Board");
                ui.spawn((
                    board.clone(),
                    UiLayout::window().x(Rl(50.0)).anchor(Anchor::TopCenter).size(Rl(105.0)).pack::<Base>(),
                    UiMaterial2dBundle {
                        material: materials.add(Color::BEVYPUNK_RED_DIM),
                        ..default()
                    }
                ));

                let list = board.add("List");
                ui.spawn((
                    list.clone(),
                    UiLayout::window().pos(Rl((22.0, 33.0))).size(Rl((55.0, 34.0))).pack::<Base>()
                ));

                let gap = 3.0;
                let size = 14.0;
                let mut offset = 0.0;

                for button_type in MainMenuButton::iter() {
                    let mut button = ui.spawn((
                        list.add(button_type.str()),
                        button_type.clone(),
                        UiLayout::window().y(Rl(offset)).size(Rl((100.0, size))).pack::<Base>(),
                        MainButton {
                            text: button_type.str()
                        }
                    ));

                    if button_type == MainMenuButton::Continue {
                        button.insert(OnUiClickDespawn::new(route_entity));
                    }

                    offset += gap + size;
                }
            });
        });
    }
}

#[derive(Component, Clone, PartialEq, EnumIter)]
enum MainMenuButton {
    Continue,
    NewGame,
    Settings,
    QuitGame
}
impl MainMenuButton {
    fn str(&self) -> String {
        match self {
            MainMenuButton::Continue => "CONTINUE".into(),
            MainMenuButton::NewGame => "NEW GAME".into(),
            MainMenuButton::Settings => "SETTINGS".into(),
            MainMenuButton::QuitGame => "QUIT GAME".into(),
        }
    }
}

fn main_menu_button_clicked_system(
    mut events: EventReader<UiClickEvent>,
    query: Query<&MainMenuButton, With<Button>>,
    mut exit: EventWriter<AppExit>
) {
    for event in events.read() {
        if let Ok(button) = query.get(event.target) {
            info!("Pressed: {}", button.str());

            match button {
                MainMenuButton::QuitGame => {
                    exit.send(AppExit::Success);
                },
                _ => {}
            }
        }
    }
}

pub struct MainMenuRoutePlugin;
impl Plugin for MainMenuRoutePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(PreUpdate, build_route.before(UiSystems::Compute))
            .add_systems(Update, main_menu_button_clicked_system
                .distributive_run_if(on_event::<UiClickEvent>())
                .distributive_run_if(input_just_pressed(MouseButton::Left)));
    }
}
