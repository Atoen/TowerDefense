use super::{GameTextures, Turret};

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
        Turret::CyberOro => 10.0,
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