use button::Button;

use crate::{WeaponPage, *};

#[derive(Component, Debug, Default, Clone, PartialEq)]
pub struct WeaponSelector;

#[derive(Component, Debug, Default, Clone, PartialEq)]
struct WeaponSelectorUi;

fn build_component(
    mut commands: Commands,
    query: Query<Entity, Added<WeaponSelector>>,
    assets: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    for entity in &query {
        commands.entity(entity).insert(
            UiTreeBundle::<WeaponSelectorUi>::from(UiTree::new2d("Weapon Selector"))
        ).with_children(|ui| {

            let row = UiLink::<WeaponSelectorUi>::path("Row");
            ui.spawn((
                row.clone(),
                UiLayout::window_full().pack::<Base>(),
                UiMaterial2dBundle {
                    material: materials.add(Color::BLACK.with_alpha(0.5)),
                    ..default()
                }
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
                    image: Some(assets.load(AssetPath::CHEVRON_LEFT))
                },
                PageNavigationButton::Previous
            ));

            ui.spawn((
                row.add("Next Page"),
                UiLayout::window().pos((Rw(100.0) - Ab(40.0), Rh(50.0))).size(40.0).anchor(Anchor::Center).pack::<Base>(),
                Button {
                    hover_enlarge: false,
                    text: None,
                    image: Some(assets.load(AssetPath::CHEVRON_RIGHT))
                },
                PageNavigationButton::Next
            ));
        });
    }
}

#[derive(Component, Clone, PartialEq)]
enum PageNavigationButton {
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
    query: Query<&PageNavigationButton>,
    state: Res<State<PageState>>,
    mut next_state: ResMut<NextState<PageState>>,
) {
    for event in events.read() {
        if let Ok(navigation_button) = query.get(event.target) { 
            let next_page = match (navigation_button, state.get()) {
                (PageNavigationButton::Next, PageState::Standard) => PageState::Advanced,
                (PageNavigationButton::Next, PageState::Advanced) => PageState::Building,
                (PageNavigationButton::Next, PageState::Building) => PageState::Standard,
                (PageNavigationButton::Previous, PageState::Standard) => PageState::Building,
                (PageNavigationButton::Previous, PageState::Advanced) => PageState::Standard,
                (PageNavigationButton::Previous, PageState::Building) => PageState::Advanced
            };

            next_state.set(next_page.clone());
            writer.send(PageChangedEvent);
        }
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

    let Ok(weapon_selector) = weapon_selector.get_single() else { return; };
    let Ok(current_page) = current_page.get_single() else { return; };

    commands.entity(current_page).despawn_recursive();

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
            .add_systems(Update, build_component.before(UiSystems::Compute))
            .add_systems(PostUpdate, page_navigation_button_clicked_system
                .distributive_run_if(on_event::<UiClickEvent>())
                .distributive_run_if(input_just_pressed(MouseButton::Left)))
            .init_state::<PageState>()
            .add_systems(Update, page_changed_system
                .run_if(on_event::<PageChangedEvent>()));
    }
}
