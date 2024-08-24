use bevy::ecs::component::StorageType;
use bevy::input::mouse::MouseWheel;
use button::Button;

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
                UiTreeBundle::<WeaponSelectorUi>::from(UiTree::new2d("Weapon Selector"))
            ).with_children(|ui| {
                let row = UiLink::<WeaponSelectorUi>::path("Row");
                ui.spawn((
                    row.clone(),
                    UiLayout::window_full().pack::<Base>()
                ));
    
                ui.spawn((
                    row.add("Page"),
                    UiLayout::window()
                        .pos((100.0, 0.0))
                        .size((Rw(100.0) - Ab(200.0), Rh(100.0)))
                        .pack::<Base>(),
                    WeaponPage
                ));
    
                ui.spawn((
                    row.add("Previous Page"),
                    UiLayout::window().pos((40.0, Rh(50.0))).size(40.0).anchor(Anchor::Center).pack::<Base>(),
                    Button {
                        hover_enlarge: false,
                        text: None,
                        image: Some(arrow_left)
                    },
                    PageNavigation::Previous
                ));
    
                ui.spawn((
                    row.add("Next Page"),
                    UiLayout::window().pos((Rw(100.0) - Ab(40.0), Rh(50.0))).size(40.0).anchor(Anchor::Center).pack::<Base>(),
                    Button {
                        hover_enlarge: false,
                        text: None,
                        image: Some(arrow_right)
                    },
                    PageNavigation::Next
                ));
            });
        });
    }
}

#[derive(Component, Debug, Default, Clone, PartialEq)]
struct WeaponSelectorUi;

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

fn page_navigation_button_clicked_system(
    mut events: EventReader<UiClickEvent>,
    mut commands: Commands,
    mut page: ResMut<WeaponSelectorPage>,
    query: Query<&PageNavigation>
) {
    for event in events.read() {
        if let Ok(navigation) = query.get(event.target) { 
            *page = get_next_page(navigation, &page);
            commands.trigger(PageChangedEvent(*page));
        }
    }
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
    weapon_selector: Query<Entity, With<WeaponSelector>>,
    current_page: Query<Entity, With<WeaponPage>>
) {
    info!("Current page: {}", trigger.event().0);

    let Ok(weapon_selector) = weapon_selector.get_single() else { return };
    let Ok(current_page) = current_page.get_single() else { return };

    commands.entity(current_page).insert(DespawnAfterFrames::TWO);

    let new_page = commands.spawn((
        UiLink::<WeaponSelectorUi>::path("Row/Page"),
        UiLayout::window()
            .pos((100.0, 0.0))
            .size((Rw(100.0) - Ab(200.0), Rh(100.0)))
            .pack::<Base>(),
        WeaponPage
    )).id();

    commands.entity(weapon_selector).add_child(new_page);
}


pub struct WaponSelectorPlugin;
impl Plugin for WaponSelectorPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<PageChangedEvent>()
            .add_plugins(UiGenericPlugin::<WeaponSelectorUi>::new())

            .init_resource::<WeaponSelectorPage>()

            .add_systems(PostUpdate, handle_scroll)

            .add_systems(PostUpdate, page_navigation_button_clicked_system
                .distributive_run_if(on_event::<UiClickEvent>())
                .distributive_run_if(input_just_pressed(MouseButton::Left)))
            
            .observe(page_changed_trigger)

            ;

            // .add_systems(Update, page_changed_trigger
            //     .run_if(on_event::<PageChangedEvent>()));
    }
}
