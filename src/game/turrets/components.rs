use bevy::{ecs::component::StorageType, utils::HashSet};

use crate::*;

#[derive(Component, Default, Clone, Copy)]
pub struct TurretLevel(pub u8);

#[derive(Component)]
pub struct RaiseTurretLevel;

#[derive(Component)]
pub struct ProjectileTurret;

#[derive(Default)]
pub struct BeamTurret {
    pub beam: Option<Entity>
}

impl Component for BeamTurret {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(_hooks: &mut bevy::ecs::component::ComponentHooks) {
        _hooks.on_remove(|mut wolrd, entity, _| {
            if let Some(beam) = wolrd.entity(entity)
                .get::<BeamTurret>()
                .and_then(|source| source.beam) {
                wolrd.commands().entity(beam).despawn();
            }
        });
    }   
}

#[derive(Component)]
pub struct AttackDelay(pub Timer);

impl AttackDelay {
    pub fn from_fire_rate(fire_rate: f32) -> Self {
        Self(Timer::from_seconds(fire_rate.recip(), TimerMode::Repeating))
    }
}

#[derive(Component, Default, Clone, Copy)]
pub struct TargetingTurret {
    pub current_angle: f32,
    pub current_target: Option<Entity>,
    pub mode: TargetingMode
}

#[derive(Default, Clone, Copy, Display)]
pub enum TargetingMode {
    #[default]
    First,
    Last,
    Strongest
}

impl TargetingMode {
    pub fn next(self) -> Self {
        match self {
            TargetingMode::First => Self::Last,
            TargetingMode::Last => Self::Strongest,
            TargetingMode::Strongest => Self::First,
        }
    }
}

#[derive(Component, Default)]
pub struct AttackDispersion(pub f32);

#[derive(Component, Default)]
pub struct ProjectileSpawnOffset(pub Vec3);
impl ProjectileSpawnOffset {
    pub fn from_vec2(offset: Vec2) -> Self {
        Self(Vec3::new(offset.x, offset.y, 0.0))
    }
}

#[derive(Component)]
pub struct RotationSpeed(pub f32);

#[derive(Component)]
pub struct PreciseAttack {
    pub max_angle_diff: f32,
    pub is_correct_angle: bool
}

#[derive(Component)]
pub struct IdleRotation {
    pub idle_timer: Timer,
    pub rotation_timer: Timer,
    pub is_idle: bool,
    pub target_angle: f32
}

impl Default for IdleRotation{
    fn default() -> Self {
        Self {
            idle_timer: Timer::from_seconds(3., TimerMode::Once),
            rotation_timer: Timer::from_seconds(2., TimerMode::Repeating),
            target_angle: 0.,
            is_idle: false
        }
    }
}

#[derive(Component)]
pub struct Beam {
    pub target: Option<Entity>,
    pub damage: Option<Damage>
}

#[derive(Component)]
pub struct Projectile {
    pub radius: f32,
    pub damage: Damage,
    pub pierce: Pierce,
    pub hit_targets: HashSet<Entity>
}
pub enum Pierce {
    Infinite,
    Finite(u32)
}

impl Pierce {
    pub const ONE: Pierce = Pierce::Finite(1);
}

#[derive(Clone, Copy)]
pub struct Damage {
    pub kind: DamageKind,
    pub source: DamageSource,
    pub damage_type: DamageType
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub enum DamageSource {
    Turret(Turret),
    Consumable(Consumable)
}

#[derive(Clone, Copy)]
pub enum DamageKind {
    Instant(f32),
    OverTime { dps: f32, duration: f32 }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DamageType {
    Kinetic,
    Energy,
    Chemical,
    True
}

#[derive(Component)]
pub struct LinearVelocity(pub f32);

#[derive(Component)]
pub struct Explosive {
    pub radius: f32,
    pub fallof: Option<DamageFalloff>,
    pub damage: Damage,
    pub animation: AoEAnimation
}

#[derive(Component)]
pub struct Homing {
    pub current_target: Option<Entity>,
    pub homing_distance: f32,
    pub homing_angle: f32,
    pub homing_speed: f32
}