use std::fmt;

use bevy::{prelude::Component, time::Timer};
use strum_macros::{Display, EnumCount, EnumIter};

use super::Damage;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Buildable {
    Standalone(StandaloneBuildable),
    Turret(Turret)
}

impl Buildable {
    pub fn is_module(&self) -> bool {
        matches!(self, Buildable::Standalone(StandaloneBuildable::Module))
    }

    pub fn is_standalone(&self) -> bool {
        matches!(self, Buildable::Standalone(_))
    }

    pub fn is_turret(&self) -> bool {
        matches!(self, Buildable::Turret(_))
    }

    pub const MODULE: Buildable = Buildable::Standalone(StandaloneBuildable::Module);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StandaloneBuildable {
    Module,
    Consumable(Consumable)
}

impl fmt::Display for Buildable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Buildable::Turret(turret) => write!(f, "{}", turret),
            Buildable::Standalone(consumable) => write!(f, "{}", consumable),
        }
    }
}

impl fmt::Display for StandaloneBuildable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StandaloneBuildable::Module => write!(f, "Module"),
            StandaloneBuildable::Consumable(consumable) => write!(f, "{}", consumable),
        }
    }
}

#[derive(Display, EnumIter, EnumCount, Clone, Copy, Debug, PartialEq, Eq, Hash, Component)]
pub enum Turret {

    // Standard
    PulseBlaster,  
    IonCannon,     
    SwarmTurret,   
    PlasmaRay,          
    CryoGenerator, 
    Tesla,         
    AcidSprayer, 
    FireThrower,
    
    // Advanced
    SeekerLauncher,
    Sentinel,
    Recycler,
    RailGun,
}

#[derive(Display, EnumIter, EnumCount, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Consumable {
    ProximityMine,
    RotorBlades
}

#[derive(Component)]
pub struct ProximityMineAnimation {
    pub light_on: bool,
    pub on_timer: Timer,
    pub off_timer: Timer
}

#[derive(Component)]
pub struct ProximityMine {
    pub trigger_radius: f32,
    pub explosion_radius: f32,
    pub damage: Damage
}

#[derive(Component)]
pub struct RotorBlades {
    pub radius: f32,
    pub damage: Damage,
    pub durability: f32
}

#[derive(Component)]
pub struct TurretComponent(pub Turret);