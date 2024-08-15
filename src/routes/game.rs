use crate::*;

#[derive(Component, Debug, Default, Clone, PartialEq)]
pub struct GameRoute;

fn build_route(
    mut commands: Commands,
    query: Query<Entity, Added<GameRoute>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    ui_textures: Res<UiTextures>,
    first_pass_handle: Res<FirstPassImageHandle>
) {
    for route_entity in &query {        
        commands.entity(route_entity).insert(
            SpatialBundle::default()
        ).with_children(|route| {

            route.spawn((
                UiTreeBundle::<MainUi>::from(UiTree::new2d("Game")),
                MovableByCamera,
                GameLayout
            )).with_children(|ui| {
    
                let root = UiLink::<MainUi>::path("Root");
                ui.spawn((
                    root.clone(),
                    UiLayout::window_full().pack::<Base>()
                ));
    
                ui.spawn((
                    root.add("Background"),
                    UiLayout::solid().size((1920.0, 1080.0)).scaling(Scaling::Fill).pack::<Base>(),
                    Pickable::IGNORE,
                    UiImage2dBundle {
                        texture: ui_textures.nebula.clone(),
                        ..default()
                    }
                ));

                ui.spawn((
                    root.add("Bottom Row/Background"),
                    UiLayout::window().y(Rl(100.0)).size((Rl(100.0), 100.)).anchor(Anchor::BottomLeft).pack::<Base>(),
                    UiMaterial2dBundle {
                        material: materials.add(Color::BLACK.with_alpha(0.7)),
                        ..default()
                    }
                ));
    
                ui.spawn((
                    root.add("Top Row"),
                    UiLayout::window().size((Rl(100.0), 50.0)).pack::<Base>(),
                    GameStatus,
                    Pickable::IGNORE
                ));
                    
                ui.spawn((
                    root.add("Bottom Row"),
                    UiLayout::window().y(Rl(100.0)).size((Rl(100.0), 100.0)).anchor(Anchor::BottomLeft).pack::<Base>(),
                    WeaponSelector
                ));            

                ui.spawn((
                    root.add("Game Arena"),
                    UiLayout::solid().size((1920.0, 1080.0)).scaling(Scaling::Fill).pack::<Base>(),
                    UiImage2dBundle::from(first_pass_handle.0.clone()),
                    GameArena,
                    UiClickEmitter::SELF
                ));
            });
        });
    }
}

#[derive(Component)]
struct GameLayout;

fn hide_build_info_system(
    mut commands: Commands,
    game_layout: Query<Entity, With<GameLayout>>,
    info: Query<Entity, With<BuildInfo>>
) {
    let Ok(info) = info.get_single() else { return; };
    let Ok(game_route) = game_layout.get_single() else { return; };

    commands.entity(info).insert(DespawnAfterFrames { delay: 2, recursive: true });

    let selector = commands.spawn((
        UiLink::<MainUi>::path("Root/Bottom Row"),
        UiLayout::window().y(Rl(100.0)).size((Rl(100.0), 100.)).anchor(Anchor::BottomLeft).pack::<Base>(),
        WeaponSelector
    )).id();

    commands.entity(game_route).add_child(selector);
}

fn build_info_display_system(
    mut commands: Commands,
    weapon_selector: Query<Entity, With<WeaponSelector>>,
    game_layout: Query<Entity, With<GameLayout>>,
    selected_buildable: Res<SelectedBuildable>
) {
    let Some(buildable) = &selected_buildable.0 else { return; };

    let Ok(weapon_selector) = weapon_selector.get_single() else { return; };
    let Ok(game_route) = game_layout.get_single() else { return; };

    commands.entity(weapon_selector).insert(DespawnAfterFrames { delay: 2, recursive: true });
    let info = commands.spawn((
        UiLink::<MainUi>::path("Root/Bottom Row"),
        UiLayout::window().y(Rl(100.0)).size((Rl(100.0), 100.)).anchor(Anchor::BottomLeft).pack::<Base>(),
        BuildInfo(*buildable)
    )).id();

    commands.entity(game_route).add_child(info);
}

pub struct GameLayoutPlugin;
impl Plugin for GameLayoutPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, build_route
                .before(UiSystems::Compute))

            .add_systems(Update, build_info_display_system
                .run_if(on_event::<BuildableSelecedEvent>()))

            .add_systems(Update, hide_build_info_system
                .run_if(on_event::<InfoClosedEvent>()));
    }
}

