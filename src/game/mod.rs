pub mod cursor_translation;

pub use cursor_translation::*;

pub mod cell_selecting;
pub use cell_selecting::*;

pub mod buildables;
pub use buildables::*;

pub mod grid;
pub use grid::*;

pub mod spawning_buildings;
pub use spawning_buildings::*;

pub mod path;
pub use path::*;

pub mod explosion;
pub use explosion::*;

pub mod aliens;
pub use aliens::*;

pub mod turrets;
pub use turrets::*;

pub mod data_tables;
pub use data_tables::*;

use crate::*;

#[derive(Component)]
pub struct ConstantRotation {
    pub speed: f32
}

#[derive(Component)]
pub struct Core;

#[derive(Component)]
pub struct Portal;

pub struct GameLayer;

impl GameLayer {

    pub const SELECTED_CELL: f32 = 100.0;
    pub const UPGRADE_ARROW: f32 = 99.0;

    pub const PATH_DOT: f32 = 8.0;
    pub const PATH_ALIEN: f32 = 9.0;

    pub const STANDALONE: f32 = 10.0;
    pub const PORTAL: f32 = 11.0;
    pub const TURRET_RING: f32 = 12.0;
    pub const TURRET: f32 = 13.0;
    pub const EXPLOSION_RANGE: f32 = 7.0;
    pub const RANGE: f32 = 7.5;

    pub const ALIEN: f32 = 14.0;
    pub const ALIEN_HEALTH_BAR: f32 = 14.1;

    pub const PROJECTILE: f32 = 15.0;

    pub const DEBUG_TARGET: f32 = 9.1;
}

pub trait GameLayerOrder {
    fn on(self, layer: f32) -> Vec3;
}

impl GameLayerOrder for Vec3 {
    fn on(mut self, layer: f32) -> Vec3 {
        self.z = layer;
        self
    }
}


fn setup_game(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    mut grid: ResMut<GameGrid>,
) {
    debug!("Setting up game level...");

    let core_pos = UVec2::new(32, 10);
    if let Some(core_cell) = grid.set_special(&core_pos, SpecialCell::Core) {
        commands.spawn((
            SpriteBundle {
                texture: game_textures.core.clone(),
                transform: Transform {
                    translation: core_cell.world_pos().on(GameLayer::STANDALONE),
                    scale: Vec3::splat(0.5),
                    ..default()
                },
                ..default()
            },
            Core,
            RenderLayers::layer(1)
        ));
    } else {
        error!("Invalid core position!");
    }

    let portal_pos = UVec2::new(8, 10);
    if let Some(portal_cell) = grid.set_special(&portal_pos, SpecialCell::Portal) {
        commands.spawn((
            SpriteBundle {
                texture: game_textures.portal.clone(),
                transform: Transform {
                    translation: portal_cell.world_pos().on(GameLayer::PORTAL),
                    scale: Vec3::splat(0.7),
                    ..default()
                },
                ..default()
            },
            ConstantRotation { speed: -std::f32::consts::FRAC_PI_4 },
            Portal,
            RenderLayers::layer(1)
        ));
    } else {
        error!("Invalid portal position!");
    }

    if let PathState::Updated = grid.calculate_path() {
        commands.trigger(PathChangedEvent);
        debug!("Level set up complete");
    } else {
        error!("Failed to set up level!");
    }
}

fn constant_rotation_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &ConstantRotation)>
) {
    for (mut transform, rotation) in &mut query {
        transform.rotate_local_z(rotation.speed * time.delta_seconds());
    }
}

fn proximity_mine_animation_system(
    time: Res<Time>,
    mut query: Query<(&mut ProximityMineAnimation, &mut TextureAtlas)>
) {
    for (mut mine, mut atlas) in &mut query {
        if mine.light_on {
            if mine.on_timer.tick(time.delta()).finished() {
                mine.light_on = false;
                mine.on_timer.reset();
                
                atlas.index = 1;
            }
        } else if mine.off_timer.tick(time.delta()).finished() {
            mine.light_on = true;
            mine.off_timer.reset();

            atlas.index = 0;
        }
    } 
}

fn consumable_damage_system(
    time: Res<Time>,
    mut commands: Commands,
    mut aliens: Query<(&GlobalTransform, &mut Alien)>,
    mut consumables: Query<(Entity, &Transform, &CellEntity, AnyOf<(&mut RotorBlades, &ProximityMine)>)>,
    mut grid: ResMut<GameGrid>
) {
    'consumables: for (consumable_entity, consumable_transform, cell, mut blades_or_mine) in &mut consumables {
        for (alien_transform, mut alien) in &mut aliens {

            let distance_2 = alien_transform.translation().distance_squared(consumable_transform.translation);

            match blades_or_mine {
                (None, Some(mine)) => {
                    if mine.trigger_radius * mine.trigger_radius >= distance_2 {
                        
                        commands.entity(consumable_entity).despawn();
                        if let Some(cell) = grid.get_cell_mut(&cell.pos) { cell.remove_standalone() }

                        commands.spawn(
                            ExplosionToSpawn {
                                damage: Some(mine.damage),
                                falloff: Some(DamageFalloff::Linear { min_damage_percent: 0.5 }),
                                radius: mine.explosion_radius,
                                position: consumable_transform.translation,
                                animation: AoEAnimation {
                                    timer: Timer::from_seconds(1., TimerMode::Once),
                                    despawn_on_end: true,
                                    radius_animation: Some(RadiusAnimation::FromBaseRadius { grow_speed: 1.0 }),
                                    color_animation: Some(ColorAnimation {
                                        start_color: Color::srgb(0.3, 0.8, 1.0),
                                        end_color: Color::srgb(0.4, 0.0, 1.0),
                                        alpha_factor: None,
                                        animate_alpha: true
                                    })
                                }
                            }
                        );

                        continue 'consumables;
                    }
                }
                (Some(ref mut blades), None) => {
                    if blades.radius * blades.radius >= distance_2 {
                        alien.add_damage(&blades.damage);
                        blades.durability -= time.delta_seconds();

                        if blades.durability <= 0.0 {
                            commands.entity(consumable_entity).despawn_recursive();
                            if let Some(cell) = grid.get_cell_mut(&cell.pos) { cell.remove_standalone() }

                            commands.spawn(
                                ExplosionToSpawn {
                                    damage: None,
                                    falloff: None,
                                    radius: 20.0,
                                    position: consumable_transform.translation,
                                    animation: AoEAnimation {
                                        timer: Timer::from_seconds(0.5, TimerMode::Once),
                                        despawn_on_end: true,
                                        radius_animation: Some(RadiusAnimation::FromBaseRadius { grow_speed: 0.5 }),
                                        color_animation: Some(ColorAnimation {
                                            start_color: Color::srgb(1.0, 0.5, 0.0),
                                            end_color: Color::srgb(0.2, 0.2, 0.2),
                                            alpha_factor: None,
                                            animate_alpha: true
                                        })
                                    }
                                }
                            );
                            
                            continue 'consumables;
                        }
                    }
                }
                _ => { }
            }
        }
    } 
}

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app

            .add_plugins((
                PathPlugin,
                CellSelectionPlugin,
                CursorTranslationPlugin,
                AliensPlugin,
                TurretsPlugin,
                ExplosionPlugin
            ))

            .add_systems(Update, (
                constant_rotation_system,
                proximity_mine_animation_system,
            ).run_if(in_state(AppState::InGame)))

            .add_systems(Update, consumable_damage_system
                .run_if(in_state(GameState::AttackWave)))

            .add_systems(OnEnter(AppState::InGame), setup_game)
            
            .observe(build_clicked_trigger)

            ;
    }
}