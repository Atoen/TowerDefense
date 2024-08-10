use bevy::asset::transformer;
use button::Button;

use crate::*;

#[derive(Component, Debug, Default, Clone, PartialEq)]
pub struct GameRoute;

fn build_route(
    mut commands: Commands,
    query: Query<Entity, Added<GameRoute>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    for route_entity in &query {
        commands.entity(route_entity).insert(
            SpatialBundle::default()
        ).with_children(|route| {

            route.spawn((
                UiTreeBundle::<MainUi>::from(UiTree::new2d("Game")),
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
                        material: materials.add(Color::GRAY_800),
                        ..default()
                    },
                    Pickable::IGNORE
                ));
    
                ui.spawn((
                    root.add("Top Row"),
                    UiLayout::window().size((Rl(100.0), 50.0)).pack::<Base>(),
                    GameStatus
                ));
                    
                ui.spawn((
                    root.add("Bottom Row"),
                    UiLayout::window().y(Rl(100.0)).size((Rl(100.0), 100.)).anchor(Anchor::BottomLeft).pack::<Base>(),
                    WeaponSelector
                ));

                ui.spawn((
                    root.add("Game Arena"),
                    UiLayout::window().y(50.0).size((Rl(100.0), Rl(100.0) - Ab(150.0))).pack::<Base>(),
                    UiClickEmitter::SELF,
                    UiZoneBundle::default(),
                    GameArena
                ));
            });
        });
    }
}

#[derive(Component)]
struct GameArena;

#[derive(Component)]
struct GridCellHighligh;

fn game_arena_clicked_system(
    mut events: EventReader<UiClickEvent>,
    query: Query<&GameArena>,
    state: Res<State<SelectedBuildabeState>>,
    mut next_state: ResMut<NextState<SelectedBuildabeState>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut writer: EventWriter<CashChangedEvent>
) {
    for event in events.read() {
        if let Ok(game_arena) = query.get(event.target) {
            let Some(click_pos) = windows.single().cursor_position() else { return; };
            let SelectedBuildabeState::Some(selected_buildable) = state.get() else { 
                info!("Clicked at {}", click_pos);
                return; 
            };

            info!("Placed {:?} at {}", selected_buildable, click_pos);

            next_state.set(SelectedBuildabeState::None);
            writer.send(CashChangedEvent{ change: -100 });
        }
    }
}

fn game_arena_hover_system(
    windows: Query<&Window, With<PrimaryWindow>>,
    query: Query<&GameArena>,
    mut highlights: Query<(Entity, &mut Transform), With<GridCellHighligh>>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if query.get_single().is_err() { return; }

    let highligh_result = highlights.get_single_mut();

    let window = windows.single();
    let Some(cursor_pos) = window.cursor_position() else {
        if let Ok((highlight, _)) = highligh_result {
            commands.entity(highlight).despawn();
        }
        return;
    };

    let pos = window_to_world_coords(cursor_pos, window.size());

    let grid_size = 40.0;
    let aligned_x = (pos.x / grid_size).floor() * grid_size;
    let aligned_y = (pos.y / grid_size).floor() * grid_size;

    let start = vec2(aligned_x, aligned_y);

    if let Ok((_, mut transform)) = highligh_result {
        transform.translation = Vec3 { x: start.x + 20.0, y: start.y + 20.0, z: 5.0 };
    } else {
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Rectangle {half_size: Vec2::splat(20.0)})),
                material: materials.add(Color::WHITE),
                transform: Transform {
                    translation: Vec3 { x: start.x + 20.0, y: start.y + 20.0, z: 5.0 },
                    ..default()
                },
                ..default()
            },
            GridCellHighligh
        ));
    }



}

fn window_to_world_coords(cursor_pos: Vec2, window_size: Vec2) -> Vec3 {
    Vec3 { 
        x: cursor_pos.x - window_size.x / 2.0,
        y: window_size.y / 2.0 - cursor_pos.y,
        z: 0.0
    }
}


pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(PostUpdate, game_arena_clicked_system
                .distributive_run_if(on_event::<UiClickEvent>())
                .distributive_run_if(input_just_pressed(MouseButton::Left)))
            .add_systems(Update, game_arena_hover_system)
            .add_systems(PreUpdate, build_route.before(UiSystems::Compute));
    }
}

