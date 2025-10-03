pub mod heavy;
pub mod chaser;
pub mod drone;
pub mod cannibal;
pub mod splitter;
pub mod shield;
pub mod snake;

use crate::engine::Vec3;
use super::entity::Enemy;

/// Context passed to enemy behaviors during update
pub struct UpdateContext<'a> {
    pub dt: f32,
    pub target_pos: Vec3,
    pub all_enemies: &'a [Enemy],
}

/// Effects that occur when an enemy dies
#[derive(Debug, Clone)]
pub enum DeathEffect {
    None,
    Split {
        position: Vec3,
        current_generation: u8,
        max_generation: u8,
        count: u8,
    },
    SpawnShields {
        position: Vec3,
        current_generation: u8,
        max_generation: u8,
        core_id: crate::engine::entity::EntityId,
        orbit_angle: f32,
        orbit_inclination: f32,
        orbit_radius: f32,
    },
}
