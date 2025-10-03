use crate::engine::Vec3;
use crate::graphics::{Color, PrimitiveType};
use super::super::types::EnemyConfig;
use super::DeathEffect;

pub fn config() -> EnemyConfig {
    EnemyConfig {
        health: 30.0,          // Glass cannon
        speed: 140.0,          // Fast and agile
        mass: 15.0,            // Light for quick movement
        collision_radius: 1.0, // Smaller, harder to hit
        visual_scale: 1.0,     // Slightly smaller
        primitive_type: PrimitiveType::Octahedron,
        color: Color::CYAN,
    }
}

pub fn on_death(_pos: Vec3) -> DeathEffect {
    DeathEffect::None
}
