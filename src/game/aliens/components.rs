use bevy::utils::HashMap;

use crate::*;

#[derive(Component, Default)]
pub struct Alien {
    damages_to_apply: HashMap<DamageSource, DamageToApply>,
    pub health: f32,
    pub max_health: f32,
    pub path_progress: f32
}

impl Alien {

    pub fn new(max_health: f32) -> Self {
        Self { 
            max_health,
            health: max_health,
            ..default()
        }
    }
    
    pub fn received_damages(&self) -> impl Iterator<Item = &DamageToApply> {
        self.damages_to_apply.values()
    }    

    pub fn received_damages_mut(&mut self) -> impl Iterator<Item = &mut DamageToApply> {
        self.damages_to_apply.values_mut()
    }

    pub fn clear_applied_damages(&mut self) {
        self.damages_to_apply.retain(|_, damage| {
            match &damage.timing {
                DamageToApplyTiming::Instant(_) => false,
                DamageToApplyTiming::OverTime { timer, .. } => !timer.finished(),
            }
        });
    }

    pub fn add_damage(&mut self, damage: &Damage) {

        match damage.timing {
            DamageTiming::Instant(instant_damage) => {
    
                if let Some(damage_from_source) = self.damages_to_apply.get_mut(&damage.source) {
                    if let DamageToApplyTiming::Instant(ref mut dmg) = damage_from_source.timing {
                        *dmg += instant_damage;
                    }
                } else {
                    self.damages_to_apply.insert(
                        damage.source,
                        DamageToApply {
                            timing: DamageToApplyTiming::Instant(instant_damage),
                            damage_type: damage.damage_type
                        }
                    );
                }
            }

            DamageTiming::OverTime { dps, duration } => {
                self.damages_to_apply.insert(
                    damage.source,
                    DamageToApply {
                        timing: DamageToApplyTiming::OverTime {
                            dps, timer: Timer::from_seconds(duration, TimerMode::Once)
                        },
                        damage_type: damage.damage_type
                    }
                 );
            }
        }
    }
}

#[derive(Component)]
pub struct CoreDamage(pub f32);

#[derive(Component)]
pub struct DamageResistant(pub f32);

#[derive(Component)]
pub struct KineticDamageResistant(pub f32);

#[derive(Component)]
pub struct EnergyDamageResistant(pub f32);

#[derive(Component)]
pub struct ChemicalDamageResistant(pub f32);

#[derive(Component)]
pub struct CrowdControlResistant(pub f32);

#[derive(Component)]
pub(crate) struct DamageToApply {
    pub timing: DamageToApplyTiming,
    pub damage_type: DamageType
}

pub(crate) enum DamageToApplyTiming {
    Instant(f32),
    OverTime { dps: f32, timer: Timer }
}
