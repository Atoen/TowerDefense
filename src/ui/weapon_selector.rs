use bevy::ecs::component::StorageType;
use bevy::input::mouse::MouseWheel;

use crate::*;

#[derive(Debug)]
pub struct WeaponSelector;

impl Component for WeaponSelector {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(_hooks: &mut bevy::ecs::component::ComponentHooks) {
        _hooks.on_add(|mut world, entity, _| {

            let arrow_left = world.resource::<UiTextures>().arrow_left.clone();
            let arrow_right = world.resource::<UiTextures>().arrow_right.clone();

            let mut commands = world.commands();

            commands.entity(entity).insert(
                NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(100.0),
                        height: Val::Px(100.0),
                        bottom: Val::Px(0.0),

                        display: Display::Flex,
                        justify_content: JustifyContent::SpaceBetween,
                        padding: UiRect::horizontal(Val::Px(10.0)),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::BLACK.with_alpha(0.5).into(),
                    z_index: ZIndex::Global(101),
                    ..default()
                }
            ).with_children(|selector| {

                selector.spawn((
                    ImageBundle {
                        image: Into::<UiImage>::into(arrow_left.clone()).with_color(Color::GRAY_500),
                        ..default()
                    },
                    PageNavigation::Previous,
                    On::<Pointer<Click>>::run(page_navigation_clicked),
                    InteractionColors::BUTTON_DEFAULT
                ));

                selector.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Px(0.0),
                            flex_grow: 1.0,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ..default()
                    },
                    PageContainer
                )).with_children(|page_container| {
                    page_container.spawn(WeaponPage);
                });

                selector.spawn((
                    ImageBundle {
                        image: Into::<UiImage>::into(arrow_right.clone()).with_color(Color::GRAY_500),
                        ..default()
                    },
                    PageNavigation::Next,
                    On::<Pointer<Click>>::run(page_navigation_clicked),
                    InteractionColors::BUTTON_DEFAULT
                ));
            });
        });
    }
}

#[derive(Component)]
struct PageContainer;

#[derive(Component, Clone, PartialEq, Display)]
enum PageNavigation {
    Next,
    Previous
}

#[derive(Resource, Default, Display, Clone, Copy, PartialEq)]
pub enum WeaponSelectorPage {
    #[default]
    Standard,
    Advanced,
    Building
}

#[derive(Event)]
pub struct PageChangedEvent(pub WeaponSelectorPage);

fn page_navigation_clicked(
    listener: Listener<Pointer<Click>>,
    buttons: Query<&PageNavigation>,
    mut commands: Commands,
    mut page: ResMut<WeaponSelectorPage>
) {
    let target = listener.target;
    let Ok(navigation) = buttons.get(target) else { return };

    *page = get_next_page(navigation, &page);
    commands.trigger(PageChangedEvent(*page));
}

fn handle_scroll(
    mut scroll: EventReader<MouseWheel>,
    mut commands: Commands,
    mut page: ResMut<WeaponSelectorPage>,
    windows: Query<&Window, With<PrimaryWindow>>
) {
    let Ok(window) = windows.get_single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    if window.size().y - cursor_pos.y > 100.0 {
        return
    }

    for event in scroll.read() {
        let navigation = if event.y > 0.0 { PageNavigation::Next } else { PageNavigation::Previous };
        
        *page = get_next_page(&navigation, &page);
        commands.trigger(PageChangedEvent(*page));
    }
}

fn get_next_page(navigation: &PageNavigation, current_page: &WeaponSelectorPage) -> WeaponSelectorPage {
    match (navigation, current_page) {
        (PageNavigation::Next, WeaponSelectorPage::Standard) => WeaponSelectorPage::Advanced,
        (PageNavigation::Next, WeaponSelectorPage::Advanced) => WeaponSelectorPage::Building,
        (PageNavigation::Next, WeaponSelectorPage::Building) => WeaponSelectorPage::Standard,
        (PageNavigation::Previous, WeaponSelectorPage::Standard) => WeaponSelectorPage::Building,
        (PageNavigation::Previous, WeaponSelectorPage::Advanced) => WeaponSelectorPage::Standard,
        (PageNavigation::Previous, WeaponSelectorPage::Building) => WeaponSelectorPage::Advanced
    }
}

fn page_changed_trigger(
    trigger: Trigger<PageChangedEvent>,
    mut commands: Commands,
    container: Query<Entity, With<PageContainer>>,
    current_page: Query<Entity, With<WeaponPage>>
) {
    info!("Current page: {}", trigger.event().0);

    let Ok(page_container) = container.get_single() else { return };
    let Ok(current_page) = current_page.get_single() else { return };

    commands.entity(current_page).despawn_recursive();

    let new_page = commands.spawn(WeaponPage).id();
    commands.entity(page_container).add_child(new_page);
}

pub struct WaponSelectorPlugin;
impl Plugin for WaponSelectorPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<PageChangedEvent>()
            .init_resource::<WeaponSelectorPage>()

            .add_systems(Update, handle_scroll)
            
            .observe(page_changed_trigger)
            ;

            // .add_systems(Update, page_changed_trigger
            //     .run_if(on_event::<PageChangedEvent>()));
    }
}
