use crate::engine::Vec3;
use crate::graphics::{Color, PrimitiveType};
use super::super::types::EnemyConfig;
use super::DeathEffect;

pub fn config() -> EnemyConfig {
    EnemyConfig {
        health: 50.0,          // Balanced
        speed: 105.0,          // Balanced speed
        mass: 25.0,            // Balanced mass
        collision_radius: 1.0, // Standard collision radius
        visual_scale: 1.0,     // Standard size
        primitive_type: PrimitiveType::Cube,
        color: Color::PINK,
    }
}

pub fn on_death(_pos: Vec3) -> DeathEffect {
    DeathEffect::None
}
