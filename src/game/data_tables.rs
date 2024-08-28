use super::{Buildable, GameTextures, Turret};

pub fn get_turret_range(turret: Turret, level: u8) -> f32 {

    let base_range = match turret {
        Turret::PulseBlaster => 200.0,
        Turret::IonCannon => 250.0,
        Turret::SwarmTurret => 150.0,
        Turret::PlasmaRay => 220.0,
        Turret::CryoGenerator => 100.0,
        Turret::Tesla => 100.0,
        Turret::SeekerLauncher => 300.0,
        Turret::AcidSprayer => 150.0,
        Turret::FireThrower => 180.0,

        Turret::Sentinel => 400.0,
        Turret::Recycler => 10.0,
        Turret::RailGun => 350.0,
    };

    if level == 0 {
        return base_range
    }

    let level = level.min(4);

    base_range * 1.1_f32.powi(level as i32)
}

pub fn get_turret_range_squared(turret: Turret, level: u8) -> f32 {
    let range = get_turret_range(turret, level);

    range * range
}

pub const fn get_mine_trigger_radius() -> f32 {
    15.0
}

pub const fn get_mine_explosion_radius() -> f32 {
    50.0
}

pub const fn get_rotor_blades_radius() -> f32 {
    25.0
}

pub fn get_turret_sprite(turret: Turret, textures: &GameTextures) -> bevy::prelude::Handle<bevy::prelude::Image> {
    match turret {
        Turret::PulseBlaster => textures.pulse_blaster.clone(),
        Turret::PlasmaRay => textures.plasma_ray.clone(),
        Turret::AcidSprayer => textures.acid_sprayer.clone(),
        Turret::RailGun => textures.rail_gun.clone(),
        _ => textures.pulse_blaster.clone()
    }
}

pub fn get_buildable_cost(buildable: Buildable) -> i32 {
    match buildable {
        Buildable::MODULE => 0,
        Buildable::Standalone(super::StandaloneBuildable::Consumable(consumable)) => {
            match consumable {
                super::Consumable::ProximityMine => 15,
                super::Consumable::RotorBlades => 30,
            }
        }
        Buildable::Turret(turret) => match turret {
            Turret::PulseBlaster => 70,
            Turret::IonCannon => 85,
            Turret::SwarmTurret => 80,
            Turret::PlasmaRay => 80,
            Turret::CryoGenerator => 100,
            Turret::Tesla => 90,
            Turret::AcidSprayer => 90,
            Turret::FireThrower => 120,
            Turret::SeekerLauncher => 130,
            Turret::Sentinel => 100,
            Turret::Recycler => 60,
            Turret::RailGun => 150,
        },
    }
}

pub fn get_turret_fire_rate(turret: Turret, level: u8) -> f32 {
    let base_fire_rate = match turret {
        Turret::PulseBlaster => 5.0,
        Turret::IonCannon => 1.0,
        Turret::SwarmTurret => 10.0,
        Turret::PlasmaRay => 100.0,
        Turret::CryoGenerator => 0.2,
        Turret::Tesla => 0.3,
        Turret::AcidSprayer => 10.0,
        Turret::FireThrower => 10.0,
        Turret::SeekerLauncher => 0.4,
        Turret::Sentinel => 0.0,
        Turret::Recycler => 0.0,
        Turret::RailGun => 0.2,
    };

    if level == 0 || matches!(turret, Turret::Sentinel | Turret::Recycler) {
        return base_fire_rate;
    }

    let level = level.min(4);

    base_fire_rate * 1.1_f32.powi(level as i32)
}

pub fn get_turret_rotation_speed(turret: Turret) -> f32 {
    match turret {
        Turret::PulseBlaster => std::f32::consts::TAU,
        Turret::IonCannon => std::f32::consts::PI * 0.66,
        Turret::SwarmTurret => std::f32::consts::PI,
        Turret::PlasmaRay => std::f32::consts::TAU,
        Turret::CryoGenerator => std::f32::consts::FRAC_PI_2,
        Turret::Tesla => std::f32::consts::FRAC_PI_2,
        Turret::AcidSprayer => std::f32::consts::PI,
        Turret::FireThrower => std::f32::consts::FRAC_PI_3,
        Turret::SeekerLauncher => std::f32::consts::PI,
        Turret::Sentinel => std::f32::consts::PI * 1.5,
        Turret::Recycler => std::f32::consts::PI,
        Turret::RailGun => std::f32::consts::PI * 1.2,
    }
}