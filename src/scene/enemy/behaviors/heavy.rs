use super::super::types::EnemyConfig;
use super::DeathEffect;
use crate::engine::Vec3;
use crate::graphics::{Color, PrimitiveType};

pub fn config() -> EnemyConfig {
    EnemyConfig {
        health: 100.0,         // Tankier than base
        speed: 70.0,           // Slower but steady
        mass: 40.0,            // Heavy mass for physics
        collision_radius: 1.0, // Larger collision radius
        visual_scale: 1.5,     // Bigger visual representation
        primitive_type: PrimitiveType::Cylinder,
        color: Color::PURPLE,
    }
}

pub fn on_death(_pos: Vec3) -> DeathEffect {
    DeathEffect::None
}
