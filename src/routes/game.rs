use bevy::ecs::component::StorageType;

use crate::*;

#[derive(Debug)]
pub struct GameRoute;

impl Component for GameRoute {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(_hooks: &mut bevy::ecs::component::ComponentHooks) {
        _hooks.on_add(|mut world, entity, _| {

            let material = world
                .resource_mut::<Assets<ColorMaterial>>()
                .add(Color::BLACK.with_alpha(0.7));
        
            let nebula = world.resource::<UiTextures>().nebula.clone();
            let render_target = world.resource::<RenderTarget>().0.clone();
            
            let mut commands = world.commands();
        
            commands.entity(entity).insert(
                SpatialBundle::default()
            ).with_children(|route| {
                route.spawn((
                    UiTreeBundle::<MainUi>::from(UiTree::new2d("Game")),
                    MovableByCamera,
                    GameLayout,
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
                            texture: nebula,
                            ..default()
                        }
                    ));
    
                    ui.spawn((
                        root.add("Bottom Row/Background"),
                        UiLayout::window().y(Rl(100.0)).size((Rl(100.0), 100.)).anchor(Anchor::BottomLeft).pack::<Base>(),
                        UiMaterial2dBundle {
                            material,
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
                        WeaponSelector,
                        BottomRow
                    ));            
    
                    ui.spawn((
                        root.add("Game Arena"),
                        UiLayout::solid().size((1920.0, 1080.0)).scaling(Scaling::Fill).pack::<Base>(),
                        UiImage2dBundle::from(render_target),
                        GameArena,
                        UiClickEmitter::SELF
                    ));
                });
            });
        });
    }
}

#[derive(Component)]
struct GameLayout;

#[derive(Event)]
pub struct BottomRowContentChangedEvent(pub BottomRowContent);

#[derive(Component)]
struct BottomRow;

#[derive(Resource, Default, Debug, Display, PartialEq, Eq, Clone, Copy)]
pub enum BottomRowContent {
    #[default]
    Selector,
    Build(Buildable),
    Manage { entity: Entity, buildable: Buildable }
}

fn update_bottom_row_trigger(
    trigger: Trigger<BottomRowContentChangedEvent>,
    mut commands: Commands,
    game_layout: Query<Entity, With<GameLayout>>,
    current_content: Query<Entity, With<BottomRow>>
) {
    let Ok(game_route) = game_layout.get_single() else { return };
    let Ok(current) = current_content.get_single() else { return };

    let content = commands.spawn((
        UiLink::<MainUi>::path("Root/Bottom Row"),
        UiLayout::window().y(Rl(100.0)).size((Rl(100.0), 100.)).anchor(Anchor::BottomLeft).pack::<Base>(),
        BottomRow
    )).id();

    match trigger.event().0 {
        BottomRowContent::Selector => commands.entity(content).insert(WeaponSelector),
        BottomRowContent::Build(buildable) => commands.entity(content).insert(BuildInfo(buildable)),
        BottomRowContent::Manage { entity, buildable } => {
            commands.entity(content).insert(ManageBuidable {
                buildable,
                entity
            })
        }
    };

    commands.entity(game_route).add_child(content);
    commands.entity(current).insert(DespawnAfterFrames::TWO);
}

pub struct GameLayoutPlugin;
impl Plugin for GameLayoutPlugin {
    fn build(&self, app: &mut App) {
        app

            .init_resource::<BottomRowContent>()
            .add_event::<BottomRowContentChangedEvent>()

            .add_systems(OnEnter(AppState::InGame), |mut commands: Commands| {
                commands.spawn(InfoText);
            })

            .observe(update_bottom_row_trigger)

            // .add_systems(Update, update_bottom_row_trigger
            //     .run_if(on_event::<BottomRowContentChangedEvent>()))

            ;
    }
}

