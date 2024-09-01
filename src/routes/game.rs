use bevy::{ecs::component::StorageType, window::WindowResized};

use crate::*;

#[derive(Debug)]
pub struct GameRoute;

impl Component for GameRoute {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(_hooks: &mut bevy::ecs::component::ComponentHooks) {
        _hooks.on_add(|mut world, entity, _| {

            let nebula = world.resource::<UiTextures>().nebula.clone();
            let render_target = world.resource::<RenderTarget>().0.clone();
            
            let mut commands = world.commands();

            commands.trigger(RecalculateWindoSize);

            commands.entity(entity).insert((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),

                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    ..default()
                },
                GameLayout
            )).with_children(|container| {
                container.spawn((
                    ImageBundle {
                        image: nebula.into(),
                        style: Style {
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                        ..default()
                    },
                    FillContainer::default(),
                    Pickable::IGNORE
                ));

                container.spawn((
                    GameStatus,
                    Pickable::IGNORE
                ));

                container.spawn((
                    ImageBundle {
                        image: render_target.into(),
                        style: Style {
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                        z_index: ZIndex::Global(99),
                        ..default()
                    },
                    FillContainer::default(),
                    GameArena::default()
                ));

                container.spawn((WeaponSelector, BottomRow));
            });
        });
    }
}

#[derive(Event)]
pub struct RecalculateWindoSize;

#[derive(Component)]
pub struct GameLayout;

#[derive(Component, Default)]
pub struct FillContainer(pub Vec2);

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
    let Ok(game_layout) = game_layout.get_single() else { return };
    let Ok(current) = current_content.get_single() else { return };

    let content = commands.spawn(BottomRow).id();
    let new_content = trigger.event().0;

    info!("Changing bottom row content to {new_content}");
    
    match new_content {
        BottomRowContent::Selector => commands.entity(content).insert(WeaponSelector),
        BottomRowContent::Build(buildable) => commands.entity(content).insert(BuildInfo(buildable)),
        BottomRowContent::Manage { entity, buildable } => {
            commands.entity(content).insert(ManageBuidable {
                buildable,
                entity
            })
        }
    };

    commands.entity(current).despawn_recursive();
    commands.entity(game_layout).add_child(content);
}

fn window_resized(
    mut reader: EventReader<WindowResized>,
    mut query: Query<(&mut Style, &mut FillContainer)>
) {
    for event in reader.read() {
        for (mut style, mut container) in &mut query {
            let width = 1920.0;
            let height = 1080.0;
    
            let window_width = event.width;
            let window_height = event.height;
    
            let scale = f32::max(window_width / width, window_height / height);
    
            let scaled_width = width * scale;
            let scaled_height = height * scale;
    
            let left = (window_width - scaled_width) / 2.0;
            let top = (window_height - scaled_height) / 2.0;
    
            style.width = Val::Px(scaled_width);
            style.height = Val::Px(scaled_height);

            container.0.x = scaled_width;
            container.0.y = scaled_height;
    
            style.left = Val::Px(left);
            style.top = Val::Px(top);
        }
    }
}

fn trigger_window_resize(
    _trigger: Trigger<RecalculateWindoSize>,
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
    mut writer: EventWriter<WindowResized>
) {
    let Ok((entity, window)) = windows.get_single() else { return };

    writer.send(WindowResized {
        window: entity,
        width: window.width(),
        height: window.height()
    });
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
            .observe(trigger_window_resize)

            .add_systems(Update, window_resized
                .run_if(on_event::<WindowResized>()))


            // .add_systems(Update, update_bottom_row_trigger
            //     .run_if(on_event::<BottomRowContentChangedEvent>()))

            ;

        }
}

