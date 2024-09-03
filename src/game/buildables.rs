use std::fmt;

use bevy::{prelude::Component, time::Timer};
use strum_macros::{EnumCount, EnumIter};

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

#[derive(EnumIter, EnumCount, Clone, Copy, Debug, PartialEq, Eq, Hash, Component)]
pub enum Turret {
    PulseBlaster,  
    IonCannon,     
    PhotonScatter,   
    PlasmaRay,          
    CryoGenerator, 
    Tesla,         
    AcidSprayer, 
    FireThrower,
    
    SeekerLauncher,
    Sentinel,
    Recycler,
    RailGun,
}

impl fmt::Display for Turret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Turret::PulseBlaster => "Pulse Blaster",
            Turret::IonCannon => "Ion Cannon",
            Turret::PhotonScatter => "Photon Scatter",
            Turret::PlasmaRay => "Plasma Ray",
            Turret::CryoGenerator => "Cryo Generator",
            Turret::Tesla => "Tesla",
            Turret::AcidSprayer => "Acid Sprayer",
            Turret::FireThrower => "Fire Thrower",
            Turret::SeekerLauncher => "Seeker Launcher",
            Turret::Sentinel => "Sentinel",
            Turret::Recycler => "Recycler",
            Turret::RailGun => "Rail Gun",
        };
        write!(f, "{}", name)
    }
}

#[derive(EnumIter, EnumCount, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Consumable {
    ProximityMine,
    RotorBlades,
}

impl fmt::Display for Consumable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Consumable::ProximityMine => "Proximity Mine",
            Consumable::RotorBlades => "Rotor Blades",
        };
        write!(f, "{}", name)
    }
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