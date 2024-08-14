use crate::*;

#[derive(Component, Debug, Default, Clone, PartialEq)]
pub struct MainMenuRoute;

fn build_route(
    mut commands: Commands,
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
                        material: materials.add(Color::GRAY_900),
                        ..default()
                    }
                ));

                let board = root.add("Solid");
                ui.spawn((
                    board.clone(),
                    UiLayout::solid().size((1.0, 2.5)).align_x(1.0).pack::<Base>(),
                ));

                let list = board.add("List");
                ui.spawn((
                    list.clone(),
                    UiLayout::window().pos(Rl(15.0)).size(Rl((80.0, 34.0))).pack::<Base>()
                ));

                let gap = 3.0;
                let size = 14.0;
                let mut offset = 0.0;

                for button_type in MainMenuButton::iter() {
                    let button = ui.spawn((
                        list.add(button_type.str()),
                        button_type.clone(),
                        UiLayout::window().y(Rl(offset)).size(Rl((100.0, size))).pack::<Base>(),
                        MenuButton {
                            text: button_type.str()
                        },
                    )).id();

                    // writer.send(UiElementAdded::new(&mut commands));

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
    mut commands: Commands,
    main_menu_route: Query<Entity, With<MainMenuRoute>>,
    query: Query<&MainMenuButton, With<MenuButton>>,
    mut exit: EventWriter<AppExit>,
) {
    for event in events.read() {
        if let Ok(button) = query.get(event.target) {
            info!("Pressed: {}", button.str());

            match button {
                MainMenuButton::NewGame => { 
                    commands.entity(main_menu_route.single()).insert(DespawnAfterFrames::ROUTE_NAVIGATION);
                    commands.spawn(GameRoute);
                },
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
            .add_systems(PreUpdate, build_route
                .before(UiSystems::Compute))
            .add_systems(PostUpdate, main_menu_button_clicked_system
                .distributive_run_if(input_just_pressed(MouseButton::Left))
            );
    }
}
