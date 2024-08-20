// pub mod bundles;
// pub use bundles::*;

// pub mod systems;
// pub use systems::*;

// pub mod components;
// pub use components::*;


// use bevy::prelude::*;

// pub struct TurretsPlugin;
// impl Plugin for TurretsPlugin {
//     fn build(&self, app: &mut App) {
//         app
//             // .add_systems(Startup, spawn_target)

//             .add_systems(Update, idle_rotation_system)

//             .add_systems(Update, flag_idle_turrets)

//             .add_systems(Update, turret_targeting_system)

//             .add_systems(Update, projectile_system)

//             .add_systems(Update, decaying_projectile_system)

//             .add_systems(Update, projectile_turret_attack_system)

//             // .add_systems(Update, homing_projectile_system)

//             .add_systems(Update, explosion_spawn_system);
//     }
// }