use bevy::ecs::component::StorageType;

use crate::*;

#[derive(Debug)]
pub struct MainMenuRoute;

impl Component for MainMenuRoute {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(_hooks: &mut bevy::ecs::component::ComponentHooks) {
        _hooks.on_add(|mut world, entity, _| {

            let material = world
                .resource_mut::<Assets<ColorMaterial>>()
                .add(Color::GRAY_900);
            
            let mut commands = world.commands();

            commands.entity(entity).insert(
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
                            material,
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
                        ui.spawn((
                            list.add(button_type.str()),
                            button_type.clone(),
                            UiLayout::window().y(Rl(offset)).size(Rl((100.0, size))).pack::<Base>(),
                            MenuButton {
                                text: button_type.str()
                            },
                        ));
    
                        offset += gap + size;
                    }
                });
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
    mut next_state: ResMut<NextState<AppState>>,
    mut exit: EventWriter<AppExit>,
) {
    for event in events.read() {
        if let Ok(button) = query.get(event.target) {

            let Ok(main_menu_route) = main_menu_route.get_single() else {
                error!("Unable to find MainMenuRoute component!");
                return
            };

            info!("Pressed: {}", button.str());

            match button {
                MainMenuButton::NewGame => { 
                    commands.entity(main_menu_route).insert(DespawnAfterFrames::ONE);
                    commands.spawn(GameRoute);

                    next_state.set(AppState::InGame);
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
            .add_systems(PostUpdate, main_menu_button_clicked_system
                .run_if(input_just_pressed(MouseButton::Left))
            );
    }
}
