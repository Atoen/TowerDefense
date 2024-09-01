use bevy::ecs::component::StorageType;
use ui::DISABLED_BUTTON_COLOR;

use crate::*;

#[derive(Debug)]
pub struct WeaponPage;

impl Component for WeaponPage {
    const STORAGE_TYPE: StorageType = StorageType::Table;
    
    fn register_component_hooks(_hooks: &mut bevy::ecs::component::ComponentHooks) {
        _hooks.on_add(|mut world, entity, _| {
        
            let page = *world.resource::<WeaponSelectorPage>();
            let cash = world.resource::<Cash>().0;
            let game_state = *world.resource::<State<GameState>>().get();

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
                        Turret::PhotonScatter,
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

                    let can_afford = get_buildable_cost(*element) <= cash;

                    let mut entity_commands = page_container.spawn((
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
                        BuildableButton {
                            buildable: *element,
                            can_afford
                        },
                        On::<Pointer<Click>>::run(buildable_clicked)
                    ));

                    if element.is_module() {
                        entity_commands.insert(ModuleButton);

                        let is_building_phase = game_state == GameState::BuildingPhase;
                        if is_building_phase {
                            entity_commands.insert(InteractionColors::BUTTON_DEFAULT);
                        }

                        entity_commands.with_children(|module_button| {
                            module_button.spawn((
                                TextBundle::from_section(element.to_string(), name_style(is_building_phase))
                                    .with_text_justify(JustifyText::Center),
                                Pickable::IGNORE,
                                TextSegment::Name
                            ));
                        });

                        continue;
                    }

                    if can_afford {
                        entity_commands.insert(InteractionColors::BUTTON_DEFAULT);
                    }

                    entity_commands.with_children(|buildable| {
                        buildable.spawn((
                            TextBundle::from_section(element.to_string(), name_style(can_afford))
                                .with_text_justify(JustifyText::Center),
                            Pickable::IGNORE,
                            TextSegment::Name
                        ));

                        buildable.spawn((
                            TextBundle::from_sections([
                                TextSection {
                                    value: "$".into(),
                                    style: price_style(can_afford)
                                },
                                TextSection {
                                    value: get_buildable_cost(*element).to_string(),
                                    style: price_style(can_afford)
                                }
                            ]),
                            Pickable::IGNORE,
                            TextSegment::Price
                        ));
                    });
                }
            });
        });
    }
}

fn name_style(affordable: bool) -> TextStyle {
    TextStyle {
        font_size: 18.0,
        color: if affordable { Color::GRAY_400 } else { DISABLED_BUTTON_COLOR },
        ..default()
    }
}

fn price_style(affordable: bool) -> TextStyle {
    TextStyle {
        font_size: 16.0,
        color: if affordable { Color::GRAY_400 } else { DISABLED_BUTTON_COLOR },
        ..default()
    }
}

#[derive(Component)]
struct ModuleButton;

#[derive(Component)]
struct BuildableButton {
    buildable: Buildable,
    can_afford: bool
}

#[derive(Component)]
enum TextSegment {
    Name,
    Price
}

fn buildable_clicked(
    listener: Listener<Pointer<Click>>,
    buttons: Query<&BuildableButton>,
    mut commands: Commands,
    mut bottom_row: ResMut<BottomRowContent>,
    game_state: Res<State<GameState>>
) {
    let target = listener.target;
    let Ok(button) = buttons.get(target) else { return };

    let buildable = button.buildable;

    info!("Clicked: {}", buildable);

    if !button.can_afford {
        let message = format!("Need more cash to build {}", button.buildable);
        commands.trigger(InfoMessageAddedEvent(message));
        return;
    }

    if buildable.is_module() && *game_state == GameState::AttackWave {
        commands.trigger(InfoMessageAddedEvent("Modules can only be placed in the build phase".into()));
        return;
    }

    *bottom_row = BottomRowContent::Build(button.buildable);
    commands.trigger(BottomRowContentChangedEvent(*bottom_row));
}

fn cash_updated_trigger(
    _trigger: Trigger<CashChangedEvent>,
    cash: Res<Cash>,
    mut buttons: Query<(Entity, &mut BuildableButton, &Children)>,
    mut text: Query<(&mut Text, &TextSegment)>,
    mut commands: Commands
) {
    for (entity, mut button, children) in &mut buttons {
        let can_afford = cash.0 >= get_buildable_cost(button.buildable);
        if can_afford == button.can_afford {
            continue;
        }

        button.can_afford = can_afford;

        if can_afford {
            commands.entity(entity).insert(InteractionColors::BUTTON_DEFAULT);
        } else {
            commands.entity(entity).remove::<InteractionColors>();
        }

        for child in children {
            if let Ok((mut text, segment)) = text.get_mut(*child) {
                let style = match segment {
                    TextSegment::Name => name_style(can_afford),
                    TextSegment::Price => price_style(can_afford),
                };
            
                for section in text.sections.iter_mut() {
                    section.style = style.clone();
                }
            }
        }
    }
}

fn set_module_button<const ENABLED: bool>(
    mut commands: Commands,
    module_button: Query<(Entity, &Children), With<ModuleButton>>,
    mut texts: Query<&mut Text>
) {
    let Ok((entity, children)) = module_button.get_single() else { return };
    
    if ENABLED {
        commands.entity(entity).insert(InteractionColors::BUTTON_DEFAULT);
    } else {
        commands.entity(entity).remove::<InteractionColors>();
    }

    for child in children.iter() {
        if let Ok(mut text) = texts.get_mut(*child) {
            for section in text.sections.iter_mut() {
                section.style = name_style(ENABLED);
            }
        }
    }
}

pub struct WeaponPagePlugin;

impl Plugin for WeaponPagePlugin {
    fn build(&self, app: &mut App) {
        app
            .observe(cash_updated_trigger)

            .add_systems(OnEnter(GameState::AttackWave), set_module_button::<false>)
            .add_systems(OnEnter(GameState::BuildingPhase), set_module_button::<true>)
            ;
    }
}