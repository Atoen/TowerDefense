use bevy::ecs::component::StorageType;

use crate::*;

#[derive(Debug)]
pub struct WeaponPage;


impl Component for WeaponPage {
    const STORAGE_TYPE: StorageType = StorageType::Table;
    
    fn register_component_hooks(_hooks: &mut bevy::ecs::component::ComponentHooks) {
        _hooks.on_add(|mut world, entity, _| {
        
            let page = *world.resource::<WeaponSelectorPage>();

            let mut commands = world.commands();

            commands.entity(entity).insert(
                NodeBundle {
                    style: Style {
                        display: Display::Flex,
                        column_gap: Val::Px(30.0),
                        ..default()
                    },
                    ..default()
                }
            ).with_children(|page_container| {

                let elements_to_display: Vec<Buildable> = match page {
                    WeaponSelectorPage::Standard => vec![
                        Turret::PulseBlaster,
                        Turret::IonCannon,
                        Turret::SwarmTurret,
                        Turret::PlasmaRay,
                        Turret::CryoGenerator,
                        Turret::Tesla,
                        Turret::AcidSprayer,
                        Turret::FireThrower
                    ].into_iter().map(Buildable::Turret).collect(),
                        
                        WeaponSelectorPage::Advanced => vec![
                        Turret::SeekerLauncher,
                        Turret::Sentinel,
                        Turret::Recycler,
                        Turret::RailGun
                    ].into_iter().map(Buildable::Turret).collect(),
    
                    WeaponSelectorPage::Building => vec![
                        StandaloneBuildable::Module,
                        StandaloneBuildable::Consumable(Consumable::ProximityMine),
                        StandaloneBuildable::Consumable(Consumable::RotorBlades)
                    ].into_iter().map(Buildable::Standalone).collect()
                };

                for element in elements_to_display.iter() {

                    page_container.spawn((
                        NodeBundle {
                            style: Style {
                                display: Display::Flex,
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                row_gap: Val::Px(10.0),
                                ..default()
                            },
                            ..default()
                        },
                        BuildableButton(*element),
                        On::<Pointer<Click>>::run(buildable_clicked)
                    )).with_children(|buildable| {
                        buildable.spawn((
                            TextBundle::from_section(element.to_string(), name_style())
                                .with_text_justify(JustifyText::Center),
                            Pickable::IGNORE
                        ));

                        if !element.is_module() {
                            buildable.spawn((
                                TextBundle::from_sections([
                                    TextSection {
                                        value: "$".into(),
                                        style: price_style()
                                    },
                                    TextSection {
                                        value: get_buildable_cost(*element).to_string(),
                                        style: price_style()
                                    }
                                ]),
                                Pickable::IGNORE
                            ));
                        }
                    });
                }
            });
        });
    }
}

fn name_style() -> TextStyle {
    TextStyle {
        font_size: 18.0,
        color: Color::WHITE,
        ..default()
    }
}

fn price_style() -> TextStyle {
    TextStyle {
        font_size: 16.0,
        color: Color::WHITE,
        ..default()
    }
}

#[derive(Component)]
struct BuildableButton(Buildable);

fn buildable_clicked(
    listener: Listener<Pointer<Click>>,
    buttons: Query<&BuildableButton>,
    mut commands: Commands,
    mut bottom_row: ResMut<BottomRowContent>
) {
    let target = listener.target;
    let Ok(button) = buttons.get(target) else { return };

    info!("Clicked: {}", button.0);

    *bottom_row = BottomRowContent::Build(button.0);
    commands.trigger(BottomRowContentChangedEvent(*bottom_row));
}
