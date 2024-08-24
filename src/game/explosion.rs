use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use game::GameLayerOrder;

use crate::*;

#[derive(Component, Clone)]
pub struct ExplosionToSpawn {
    pub damage: Option<Damage>,
    pub falloff: Option<DamageFalloff>,
    pub radius: f32,
    pub position: Vec3,
    pub animation: AoEAnimation
}

#[derive(Component)]
pub struct Explosion {
    pub damage: Damage,
    pub falloff: Option<DamageFalloff>,
    pub radius: f32,
    pub position: Vec3
}

#[derive(Clone, Copy)]
pub enum DamageFalloff {
    Linear { min_damage_percent: f32 },
    Exponential { min_damage_percent: f32 },
}

#[derive(Component, Clone)]
pub struct AoEAnimation {
    pub timer: Timer,
    pub radius_animation: Option<RadiusAnimation>,
    pub color_animation: Option<ColorAnimation>,
    pub despawn_on_end: bool
}

#[derive(Clone, Copy)]
pub enum RadiusAnimation {
    FromBaseRadius { grow_speed: f32 },
    FromStartToEnd { start_radius: f32, end_radius: f32 },
}


#[derive(Clone, Copy)]
pub struct ColorAnimation {
    pub start_color: Color,
    pub end_color: Color,
    pub alpha_factor: Option<f32>,
    pub animate_alpha: bool,
}

fn explosion_spawn_system(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
	query: Query<(Entity, &ExplosionToSpawn)>,
) {
    for (entity, explosion_to_spawn) in &query {
        commands.entity(entity).despawn();

        let start_color = if let Some(color_animation) = &explosion_to_spawn.animation.color_animation {
            color_animation.start_color
        } else {
            Color::srgb(1.0, 1.0, 0.0)
        };

        commands.spawn((
            MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Circle { radius: explosion_to_spawn.radius })),
                material: materials.add(start_color),
                transform: Transform {
                    translation: explosion_to_spawn.position.on(GameLayer::PROJECTILE),
                    ..default()
                },
                ..default()
            },
            explosion_to_spawn.animation.clone(),
            RenderLayers::layer(1)
        ));

        if let Some(damage) = explosion_to_spawn.damage {
            commands.spawn(
                Explosion {
                    damage,
                    falloff: explosion_to_spawn.falloff,
                    radius: explosion_to_spawn.radius,
                    position: explosion_to_spawn.position,
                }
            );
        }
    }
}

fn explosion_damage_system(
    mut commands: Commands,
    explosions: Query<(Entity, &Explosion)>,
    mut aliens: Query<(&mut Alien, &GlobalTransform)>
) {
    for (explosion_entity, explosion) in &explosions {
        for (mut alien, alien_transform) in &mut aliens {
            let distance = (explosion.position - alien_transform.translation()).truncate().length();

            if distance > explosion.radius {
                continue;
            }

            let falloff_factor = match explosion.falloff {
                Some(DamageFalloff::Linear { min_damage_percent }) => {
                    let factor = 1.0 - (distance / explosion.radius);
                    factor.max(min_damage_percent)
                }
                Some(DamageFalloff::Exponential { min_damage_percent }) => {
                    let factor = (1.0 - (distance / explosion.radius)).powi(2);
                    factor.max(min_damage_percent)
                }
                None => 1.0,
            };

            let mut scaled_damage = explosion.damage;
            match scaled_damage.kind {
                DamageKind::Instant(instant) => scaled_damage.kind = DamageKind::Instant(instant * falloff_factor),
                DamageKind::OverTime { dps, duration } => scaled_damage.kind = DamageKind::OverTime { dps: dps * falloff_factor, duration },
            }

            alien.add_damage(&scaled_damage);
        }

        commands.entity(explosion_entity).despawn();
    }
}

fn aoe_animation_system(
	mut commands: Commands,
	time: Res<Time>,
    mut materials: ResMut<Assets<ColorMaterial>>,
	mut animations: Query<(Entity, &mut AoEAnimation, &mut Transform, &Handle<ColorMaterial>)>
) {
    for (
        entity,
        mut animation,
        mut animation_transform,
        handle
    ) in &mut animations {
        animation.timer.tick(time.delta());
        let t = animation.timer.fraction();
        let t_1 = animation.timer.fraction_remaining();

        if let Some(radius_animation) = &animation.radius_animation {
            match radius_animation {
                RadiusAnimation::FromBaseRadius { grow_speed } => {
                    let scale = animation_transform.scale.x + grow_speed * time.delta_seconds();
                    animation_transform.scale = Vec3::splat(scale);
                },
                RadiusAnimation::FromStartToEnd { start_radius, end_radius } => {
                    let radius = start_radius.lerp(*end_radius, t);
                    animation_transform.scale = Vec3::splat(radius);
                },
            }
        }

        if let Some(color_animation) = &animation.color_animation {
            if let Some(material) = materials.get_mut(handle) {
                let start = color_animation.start_color.to_srgba();
                let end = color_animation.end_color.to_srgba();

                let color = Color::srgba(
                    start.red * t_1 + end.red * t,
                    start.green * t_1 + end.green * t,
                    start.blue * t_1 + end.blue * t,
                    if color_animation.animate_alpha {
                        color_animation.alpha_factor.unwrap_or(1.0) * t_1
                    } else {
                        material.color.alpha()
                    }
                );

                material.color = color;
            }
        }   

        if animation.despawn_on_end && animation.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub struct ExplosionPlugin;
impl Plugin for ExplosionPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                explosion_spawn_system,
                aoe_animation_system,
                explosion_damage_system
            ))

            ;
    }
}

