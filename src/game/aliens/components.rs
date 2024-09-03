use std::time::Duration;

use bevy::utils::HashMap;

use crate::*;

#[derive(Component, Default)]
pub struct Alien {
    // damages_to_apply: HashMap<DamageSource, DamageToApply>,

    dot_to_apply: HashMap<Entity, DamageToApply>,
    damage_to_apply: Vec<DamageToApply>,

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
        self.damage_to_apply.iter().chain(self.dot_to_apply.values())
    }    

    pub fn received_damages_mut(&mut self) -> impl Iterator<Item = &mut DamageToApply> {
        self.damage_to_apply.iter_mut().chain(self.dot_to_apply.values_mut())
    }

    pub fn clear_applied_damages(&mut self) {
        self.damage_to_apply.clear();

        self.dot_to_apply.retain(|_, damage| {
            match &damage.timing {
                DamageToApplyTiming::Instant(_) => false,
                DamageToApplyTiming::OverTime { timer, .. } => !timer.finished(),
            }
        });
    }

    pub fn add_damage(&mut self, damage: &Damage, source: Entity) {
        match damage.kind {
            DamageKind::Instant(instant) => {

                // Look for existing damage from the same source in the regular damage list
                if let Some(damage_of_type) = self.damage_to_apply.iter_mut()
                    .find(|d| d.damage_type == damage.damage_type) {
                    
                    if let DamageToApplyTiming::Instant(ref mut dmg) = damage_of_type.timing {
                        *dmg += instant;
                    } else {
                        warn!("Dropping damage! Incorrect timing type.");
                    }
                } else {
                    // No existing damage from this source, so add a new entry
                    self.damage_to_apply.push(
                        DamageToApply {
                            damage_type: damage.damage_type,
                            timing: DamageToApplyTiming::Instant(instant)
                        }
                    );
                }
            }

            DamageKind::OverTime { dps, duration } => {
                
                // Update the existing DOT
                if let Some(
                    DamageToApply {
                        timing: DamageToApplyTiming::OverTime {
                            dps: source_dps,
                            timer: source_timer
                        },
                        ..
                    }
                ) = self.dot_to_apply.get_mut(&source) {
                    
                    let new_duration = Duration::from_secs_f32(duration).max(source_timer.duration());
                    
                    source_timer.set_duration(new_duration);
                    source_timer.set_elapsed(Duration::ZERO);

                    *source_dps = dps.max(*source_dps);
                } else {

                    // No existing DOT from this source, so add a new entry
                    self.dot_to_apply.insert(
                        source,
                        DamageToApply {
                            timing: DamageToApplyTiming::OverTime {
                                timer: Timer::from_seconds(duration, TimerMode::Once),
                                dps
                            },
                            damage_type: damage.damage_type
                        }
                    );
                }
            }
        }
    }
}

#[derive(Component)]
pub struct CoreDamage(pub i32);

#[derive(Component)]
pub struct CashReward(pub i32);

#[derive(Component)]
pub struct DamageResistance {
    base: f32,
    kinetic: f32,
    energy: f32,
    chemical: f32,
}

impl DamageResistance {
    pub fn new(
        base: Option<f32>,
        kinetic: Option<f32>,
        energy: Option<f32>,
        chemical: Option<f32>,
    ) -> Self {
        let base_resistance = base.unwrap_or(0.0);
        Self {
            base: 1.0 - base_resistance,
            kinetic: 1.0 - kinetic.unwrap_or(0.0),
            energy: 1.0 - energy.unwrap_or(0.0),
            chemical: 1.0 - chemical.unwrap_or(0.0),
        }
    }

    pub fn calculate_vulnerability(&self, damage_type: DamageType) -> f32 {
        match damage_type {
            DamageType::Kinetic => self.base * self.kinetic,
            DamageType::Energy => self.base * self.energy,
            DamageType::Chemical => self.base * self.chemical,
            DamageType::True => 1.0,
        }
    }
}

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
