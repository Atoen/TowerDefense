use bevy::ecs::{component::StorageType, system::EntityCommands};

use crate::*;

#[derive(Resource, Default, Clone, Copy)]
pub struct AppSettings {
    pub interface_scale: InterfaceScale,
    pub fps_limit: FpsLimit,
    pub display_fps: bool
}

#[derive(Default, Clone, Copy, Display)]
pub enum InterfaceScale {
    Small,
    #[default]
    Medium,
    Large
}

#[derive(Default, Clone, Copy)]
pub enum FpsLimit {
    Fps30,
    Fps60,
    #[default]
    Display,
    Off
}

impl std::fmt::Display for FpsLimit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            FpsLimit::Fps30 => "30 fps",
            FpsLimit::Fps60 => "60 fps",
            FpsLimit::Display => "Display",
            FpsLimit::Off => "Off"
        };

        write!(f, "{output}")
    }
}

#[derive(Debug)]
pub struct AppSettingsPage;

impl Component for AppSettingsPage {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(_hooks: &mut bevy::ecs::component::ComponentHooks) {
        _hooks.on_add(|mut world, entity, _| {
        
            let app_settings = *world.resource::<AppSettings>();
            
            let mut commands = world.commands();

            commands.entity(entity).insert(
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),

                        justify_content: JustifyContent::Center,

                        ..default()
                    },
                    ..default()
                }
            ).with_children(|background| {

                background.spawn(
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(40.0),
                            height: Val::Percent(100.0),
                            min_width: Val::Px(300.0),

                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(10.0),
                            padding: UiRect::all(Val::Px(20.0)),
                            ..default()
                        },
                        background_color: Color::GRAY_800.into(),
                        ..default()
                    }
                ).spawn_layout(&app_settings)
            });
        });
    }
}

fn section_style() -> Style {
    Style {
        width: Val::Percent(100.0),
        height: Val::Px(80.0),
        display: Display::Flex,
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        ..default()
    }
}

fn button_style() -> Style {
    Style {
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
        width: Val::Px(100.0),
        ..default()
    }
}

fn text_style() -> TextStyle {
    TextStyle {
        color: Color::WHITE,
        font_size: 20.0,
        ..default()
    }
}

trait SettingsLayout {
    fn spawn_layout(&mut self, settings: &AppSettings);
}

impl<'a> SettingsLayout for EntityCommands<'a> {
    fn spawn_layout(&mut self, settings: &AppSettings) {

        self.with_children(|container| {

            container.spawn(
                NodeBundle {
                    style: section_style(),
                    ..default()
                },
            ).with_children(|section| {
                section.spawn(TextBundle::from_section("FPS limit", text_style()));

                section.spawn((
                    ButtonBundle {
                        style: button_style(),
                        ..default()
                    },
                    On::<Pointer<Click>>::run(fps_limit_button_clicked),
                )).with_children(|button| {
                    button.spawn((
                        TextBundle::from_section(settings.fps_limit.to_string(), text_style()),
                        FpsLimitText,
                        Pickable::IGNORE
                    ));
                });
            });

            container.spawn(
                NodeBundle {
                    style: section_style(),
                    ..default()
                },
            ).with_children(|section| {
                section.spawn(TextBundle::from_section("Show FPS", text_style()));

                section.spawn((
                    ButtonBundle {
                        style: button_style(),
                        ..default()
                    },
                    On::<Pointer<Click>>::run(show_fps_button_clicked),
                )).with_children(|button| {
                    button.spawn((
                        TextBundle::from_section(settings.display_fps.to_string(), text_style()),
                        ShowFpsText,
                        Pickable::IGNORE
                    ));
                });
            });

            container.spawn(
                NodeBundle {
                    style: section_style(),
                    ..default()
                },
            ).with_children(|section| {
                section.spawn(TextBundle::from_section("Interface scale", text_style()));

                section.spawn((
                    ButtonBundle {
                        style: button_style(),
                        ..default()
                    },
                    On::<Pointer<Click>>::run(intarface_scale_button_clicked),
                )).with_children(|button| {
                    button.spawn((
                        TextBundle::from_section(settings.interface_scale.to_string(), text_style()),
                        InterfaceScaleText,
                        Pickable::IGNORE
                    ));
                });
            });
            
            container.spawn(
                NodeBundle {
                    style: Style {
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                },
            ).with_children(|section| {
                section.spawn((
                    ButtonBundle {
                        style: button_style(),
                        ..default()
                    },
                    On::<Pointer<Click>>::run(back_button_clicked)
                )).with_children(|back_button| {
                    back_button.spawn((
                        TextBundle::from_section("Back", text_style()),
                        Pickable::IGNORE
                    ));
                });
            });
        });
    }
}

#[derive(Component)]
struct FpsLimitText;

#[derive(Component)]
struct ShowFpsText;

#[derive(Component)]
struct InterfaceScaleText;

fn back_button_clicked(
    query: Query<Entity, With<AppSettingsPage>>,
    mut commands: Commands,
) {
    let Ok(page) = query.get_single() else { return };

    commands.entity(page).despawn_recursive();
    commands.spawn(MainMenuPage);
}

fn intarface_scale_button_clicked(
    mut text: Query<&mut Text, With<InterfaceScaleText>>,
    mut app_settings: ResMut<AppSettings>,
    mut ui_scale: ResMut<UiScale>
) {
    let Ok(mut text) = text.get_single_mut() else { return };

    let next_scale = get_next_interface_scale(app_settings.interface_scale);
    app_settings.interface_scale = next_scale;
    
    match next_scale {
        InterfaceScale::Small => ui_scale.0 = 0.75,
        InterfaceScale::Medium => ui_scale.0 = 1.0,
        InterfaceScale::Large => ui_scale.0 = 1.5,
    }

    text.sections[0].value = next_scale.to_string();
}

fn show_fps_button_clicked(
    mut query: Query<&mut Visibility, With<FpsRoot>>,
    mut text: Query<&mut Text, With<ShowFpsText>>,
    mut app_settings: ResMut<AppSettings>,
) {
    let Ok(mut text) = text.get_single_mut() else { return };

    let mut vis = query.single_mut();
    *vis = match *vis {
        Visibility::Hidden => Visibility::Visible,
        _ => Visibility::Hidden,
    };

    app_settings.display_fps = matches!(*vis, Visibility::Inherited | Visibility::Visible);
    text.sections[0].value = app_settings.display_fps.to_string();
}

fn fps_limit_button_clicked(
    mut text: Query<&mut Text, With<FpsLimitText>>,
    mut app_settings: ResMut<AppSettings>,
    mut fps_settings: ResMut<bevy_framepace::FramepaceSettings>
) {
    let Ok(mut text) = text.get_single_mut() else { return };

    let next_limit = get_next_fps_limit(app_settings.fps_limit);
    app_settings.fps_limit = next_limit;
    text.sections[0].value = next_limit.to_string();

    use bevy_framepace::Limiter;
    fps_settings.limiter = match next_limit {
        FpsLimit::Fps30 => Limiter::from_framerate(30.0),
        FpsLimit::Fps60 => Limiter::from_framerate(60.0),
        FpsLimit::Display => Limiter::Auto,
        FpsLimit::Off => Limiter::Off
    }
}

fn get_next_fps_limit(current: FpsLimit) -> FpsLimit {
    match current {
        FpsLimit::Fps30 => FpsLimit::Off,
        FpsLimit::Fps60 => FpsLimit::Fps30,
        FpsLimit::Display => FpsLimit::Fps60,
        FpsLimit::Off => FpsLimit::Display,
    }
}

fn get_next_interface_scale(current: InterfaceScale) -> InterfaceScale {
    match current {
        InterfaceScale::Small => InterfaceScale::Medium,
        InterfaceScale::Medium => InterfaceScale::Large,
        InterfaceScale::Large => InterfaceScale::Small,
    }
}

pub struct SettingsPlugin;
impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {

        app
            .init_resource::<AppSettings>()

            .observe(|_trigger: Trigger<FpsCounterVisibilityChanged>, mut text: Query<&mut Text, With<ShowFpsText>>, app_settings: Res<AppSettings>| {
                let Ok(mut text) = text.get_single_mut() else { return };
            
                text.sections[0].value = app_settings.display_fps.to_string();
            });
    }
}