use std::fmt;

use bevy::{prelude::{Component, Resource}, time::Timer};
use strum_macros::{Display, EnumCount, EnumIter};

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

#[derive(Display, EnumIter, EnumCount, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Turret {

    // Standard
    PulseBlaster,  
    IonCannon,     
    SwarmTurret,   
    PlasmaRay,          
    CryoGenerator, 
    Tesla,         
    SeekerLauncher,
    AcidSprayer,    
    FireThrower,

    // Advanced
    Sentinel,
    CyberOro,
    RailGun,
}

#[derive(Display, EnumIter, EnumCount, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Consumable {
    ProximityMine,
    RotorBlades
}

#[derive(Resource, Default)]
pub struct SelectedBuildable(pub Option<Buildable>);

#[derive(Component)]
pub struct ProximityMine {
    pub light_on: bool,
    pub on_timer: Timer,
    pub off_timer: Timer
}

#[derive(Component)]
pub struct RotorBlades;

#[derive(Component)]
pub struct TurretComponent(pub Turret);