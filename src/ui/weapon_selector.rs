use bevy::input::mouse::MouseWheel;
use button::Button;

use crate::{WeaponPage, *};

#[derive(Component, Debug, Default, Clone, PartialEq)]
pub struct WeaponSelector;

#[derive(Component, Debug, Default, Clone, PartialEq)]
struct WeaponSelectorUi;

fn build_component(
    mut commands: Commands,
    query: Query<Entity, Added<WeaponSelector>>,
    assets: Res<AssetServer>
) {
    for entity in &query {
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
                    image: Some(assets.load(AssetPath::ARROW_LEFT))
                },
                PageNavigation::Previous
            ));

            ui.spawn((
                row.add("Next Page"),
                UiLayout::window().pos((Rw(100.0) - Ab(40.0), Rh(50.0))).size(40.0).anchor(Anchor::Center).pack::<Base>(),
                Button {
                    hover_enlarge: false,
                    text: None,
                    image: Some(assets.load(AssetPath::ARROW_RIGHT))
                },
                PageNavigation::Next
            ));
        });
    }
}

#[derive(Component, Clone, PartialEq, Display)]
enum PageNavigation {
    Next,
    Previous
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash, Display)]
pub enum PageState {
    #[default]
    Standard,
    Advanced,
    Building
}

#[derive(Event)]
pub struct PageChangedEvent;

fn page_navigation_button_clicked_system(
    mut events: EventReader<UiClickEvent>,
    mut writer: EventWriter<PageChangedEvent>,
    query: Query<&PageNavigation>,
    state: Res<State<PageState>>,
    mut next_state: ResMut<NextState<PageState>>,
) {
    for event in events.read() {
        if let Ok(navigation) = query.get(event.target) { 
            let next_page = get_next_page(navigation, state.get());

            next_state.set(next_page.clone());
            writer.send(PageChangedEvent);
        }
    }
}

fn handle_scroll(
    mut scroll: EventReader<MouseWheel>,
    state: Res<State<PageState>>,
    mut next_state: ResMut<NextState<PageState>>,
    mut writer: EventWriter<PageChangedEvent>,
    windows: Query<&Window, With<PrimaryWindow>>
) {
    let Ok(window) = windows.get_single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    if window.size().y - cursor_pos.y > 100.0 {
        return
    }

    for event in scroll.read() {
        let navigation = if event.y > 0.0 { PageNavigation::Next } else { PageNavigation::Previous };
        let next_page = get_next_page(&navigation, state.get());

        next_state.set(next_page);
        writer.send(PageChangedEvent);
    }
}

fn get_next_page(navigation: &PageNavigation, current_page: &PageState) -> PageState {
    match (navigation, current_page) {
        (PageNavigation::Next, PageState::Standard) => PageState::Advanced,
        (PageNavigation::Next, PageState::Advanced) => PageState::Building,
        (PageNavigation::Next, PageState::Building) => PageState::Standard,
        (PageNavigation::Previous, PageState::Standard) => PageState::Building,
        (PageNavigation::Previous, PageState::Advanced) => PageState::Standard,
        (PageNavigation::Previous, PageState::Building) => PageState::Advanced
    }
}

fn page_changed_system(
    mut commands: Commands,
    state: Res<State<PageState>>,
    weapon_selector: Query<Entity, With<WeaponSelector>>,
    current_page: Query<Entity, With<WeaponPage>>,
) {
    let page = state.get();

    info!("Current page: {}", page);

    let Ok(weapon_selector) = weapon_selector.get_single() else { return };
    let Ok(current_page) = current_page.get_single() else { return };

    commands.entity(current_page).insert(DespawnAfterFrames::ROUTE_NAVIGATION);

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

            .add_systems(PostUpdate, handle_scroll)
            .add_systems(Update, build_component.before(UiSystems::Compute))

            .add_systems(PostUpdate, page_navigation_button_clicked_system
                .distributive_run_if(on_event::<UiClickEvent>())
                .distributive_run_if(input_just_pressed(MouseButton::Left)))

            .init_state::<PageState>()
            .add_systems(Update, page_changed_system
                .run_if(on_event::<PageChangedEvent>()));
    }
}
