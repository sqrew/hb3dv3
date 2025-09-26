use crate::graphics::{Color, PrimitiveType};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnemyType {
    Heavy,  // Cyan Sphere
    Chaser, // Green Octahedron
    Drone,  // Pink Cube
}

#[derive(Debug, Clone)]
pub struct EnemyConfig {
    pub health: f32,
    pub speed: f32,
    pub mass: f32,
    pub collision_radius: f32,
    pub visual_scale: f32,
    pub primitive_type: PrimitiveType,
    pub color: Color,
}

impl EnemyType {
    pub fn config(&self) -> EnemyConfig {
        match self {
            EnemyType::Heavy => EnemyConfig {
                health: 100.0,         // Tankier than base
                speed: 50.0,           // Slower but steady
                mass: 50.0,            // Heavy mass for physics
                collision_radius: 0.8, // Larger collision radius
                visual_scale: 1.2,     // Bigger visual representation
                primitive_type: PrimitiveType::Cylinder,
                color: Color::PURPLE,
            },
            EnemyType::Chaser => EnemyConfig {
                health: 30.0,          // Glass cannon
                speed: 150.0,          // Fast and agile
                mass: 15.0,            // Light for quick movement
                collision_radius: 0.5, // Smaller, harder to hit
                visual_scale: 0.9,     // Slightly smaller
                primitive_type: PrimitiveType::Octahedron,
                color: Color::CYAN,
            },
            EnemyType::Drone => EnemyConfig {
                health: 50.0,          // Balanced
                speed: 100.0,          // Balanced speed
                mass: 25.0,            // Balanced mass
                collision_radius: 0.6, // Standard collision radius
                visual_scale: 1.0,     // Standard size
                primitive_type: PrimitiveType::Cube,
                color: Color::PINK,
            },
        }
    }

    pub fn random() -> Self {
        use rand::Rng;
        match rand::rng().random_range(0..3) {
            0 => EnemyType::Heavy,
            1 => EnemyType::Chaser,
            _ => EnemyType::Drone,
        }
    }
}
