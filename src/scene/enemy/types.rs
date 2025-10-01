use crate::graphics::{Color, PrimitiveType};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnemyType {
    Heavy,  // Cyan Sphere
    Chaser, // Green Octahedron
    Drone,  // Pink Cube
    Splitter {
        current_generation: u8,
        max_generation: u8,
    }, // Multi-colored fractal miniboss that splits recursively
    Cannibal {
        meals_consumed: u8,
    }, // Dark red predator that eats other enemies and grows stronger
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
                speed: 70.0,           // Slower but steady
                mass: 50.0,            // Heavy mass for physics
                collision_radius: 1.0, // Larger collision radius
                visual_scale: 1.5,     // Bigger visual representation
                primitive_type: PrimitiveType::Cylinder,
                color: Color::PURPLE,
            },
            EnemyType::Chaser => EnemyConfig {
                health: 30.0,          // Glass cannon
                speed: 140.0,          // Fast and agile
                mass: 15.0,            // Light for quick movement
                collision_radius: 1.0, // Smaller, harder to hit
                visual_scale: 1.0,     // Slightly smaller
                primitive_type: PrimitiveType::Octahedron,
                color: Color::CYAN,
            },
            EnemyType::Drone => EnemyConfig {
                health: 50.0,          // Balanced
                speed: 105.0,          // Balanced speed
                mass: 25.0,            // Balanced mass
                collision_radius: 1.0, // Standard collision radius
                visual_scale: 1.0,     // Standard size
                primitive_type: PrimitiveType::Cube,
                color: Color::PINK,
            },
            EnemyType::Splitter {
                current_generation,
                max_generation,
            } => {
                // Base stats for generation 0 (the boss)
                let base_health = 300.0;
                let base_speed = 40.0;
                let base_mass = 100.0;
                let base_scale = 16.0;

                // Exponential scaling per generation (reduced for more gradual progression)
                let generation = *current_generation as f32;
                let health_decay = 0.80_f32.powf(generation); // Percentage of max health retained when splitting
                let speed_growth = 1.20_f32.powf(generation); // 1.35 -> 1.15 (slower speed increase)
                let mass_decay = 0.80_f32.powf(generation); // 0.55 -> 0.70 (less mass loss)
                let scale_decay = 0.80_f32.powf(generation); // 0.75 -> 0.80 (less visual shrinking)

                // Color gradient from yellow (gen 0) to white (max gen)
                let max_gen = (*max_generation).max(1) as f32; // Avoid division by zero
                let t = generation / max_gen; // Normalize to 0.0-1.0

                // Gradient: Yellow → Orange → Red → Magenta → Purple → Blue → Cyan → White
                let color = if t < 0.16 {
                    // Yellow to Orange
                    let local_t = t / 0.16;
                    Color::new(1.0, 1.0 - local_t * 0.5, 0.0, 1.0)
                } else if t < 0.33 {
                    // Orange to Red
                    let local_t = (t - 0.16) / 0.17;
                    Color::new(1.0, 0.5 - local_t * 0.5, 0.0, 1.0)
                } else if t < 0.50 {
                    // Red to Magenta
                    let local_t = (t - 0.33) / 0.17;
                    Color::new(1.0, 0.0, local_t, 1.0)
                } else if t < 0.66 {
                    // Magenta to Purple
                    let local_t = (t - 0.50) / 0.16;
                    Color::new(1.0 - local_t * 0.5, 0.0, 1.0, 1.0)
                } else if t < 0.83 {
                    // Purple to Blue
                    let local_t = (t - 0.66) / 0.17;
                    Color::new(0.5 - local_t * 0.5, local_t * 0.5, 1.0, 1.0)
                } else {
                    // Blue to White
                    let local_t = (t - 0.83) / 0.17;
                    Color::new(local_t, 0.5 + local_t * 0.5, 1.0, 1.0)
                };

                EnemyConfig {
                    health: base_health * health_decay,
                    speed: base_speed * speed_growth,
                    mass: base_mass * mass_decay,
                    collision_radius: base_scale * scale_decay, // Match visual scale exactly
                    visual_scale: base_scale * scale_decay,
                    primitive_type: PrimitiveType::Dodecahedron,
                    color,
                }
            }
            EnemyType::Cannibal { meals_consumed } => {
                // Cannibal: Fast predator that hunts other enemies
                let base_health = 1000.0;
                let base_speed = 200.0;
                let base_mass = 100.0;
                let base_scale = 3.0;

                // Growth per meal consumed (max 10 meals for balance)
                let meals = (*meals_consumed).min(10) as f32;
                let max_health_bonus = meals * 100.0; // +15 max HP per meal
                let scale_multiplier = 1.0 + (meals * 0.50); // +50% size per meal

                EnemyConfig {
                    health: base_health + max_health_bonus,
                    speed: base_speed,
                    mass: base_mass,
                    collision_radius: base_scale * scale_multiplier,
                    visual_scale: base_scale * scale_multiplier,
                    primitive_type: PrimitiveType::Tetrahedron, // Sharp, predatory shape
                    color: Color::new(0.6, 0.0, 0.0, 1.0),      // Dark red/crimson
                }
            }
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
