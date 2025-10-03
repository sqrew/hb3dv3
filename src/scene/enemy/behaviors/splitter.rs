use crate::engine::Vec3;
use crate::graphics::{Color, PrimitiveType};
use super::super::types::EnemyConfig;
use super::DeathEffect;

#[derive(Debug, Clone, PartialEq)]
pub struct SplitterData {
    pub current_generation: u8,
    pub max_generation: u8,
}

impl SplitterData {
    pub fn new(max_generation: u8) -> Self {
        Self {
            current_generation: 0,
            max_generation,
        }
    }

    pub fn next_generation(&self) -> Self {
        Self {
            current_generation: self.current_generation + 1,
            max_generation: self.max_generation,
        }
    }

    pub fn config(&self) -> EnemyConfig {
        // Base stats for generation 0 (the boss)
        let base_health = 300.0;
        let base_speed = 40.0;
        let base_mass = 100.0;
        let base_scale = 16.0;

        // Exponential scaling per generation
        let generation = self.current_generation as f32;
        let health_decay = 0.80_f32.powf(generation);
        let speed_growth = 1.20_f32.powf(generation);
        let mass_decay = 0.80_f32.powf(generation);
        let scale_decay = 0.80_f32.powf(generation);

        // Color gradient from yellow (gen 0) to white (max gen)
        let max_gen = self.max_generation.max(1) as f32;
        let t = generation / max_gen;

        // Gradient: Yellow → Orange → Red → Magenta → Purple → Blue → Cyan → White
        let color = if t < 0.16 {
            let local_t = t / 0.16;
            Color::new(1.0, 1.0 - local_t * 0.5, 0.0, 1.0)
        } else if t < 0.33 {
            let local_t = (t - 0.16) / 0.17;
            Color::new(1.0, 0.5 - local_t * 0.5, 0.0, 1.0)
        } else if t < 0.50 {
            let local_t = (t - 0.33) / 0.17;
            Color::new(1.0, 0.0, local_t, 1.0)
        } else if t < 0.66 {
            let local_t = (t - 0.50) / 0.16;
            Color::new(1.0 - local_t * 0.5, 0.0, 1.0, 1.0)
        } else if t < 0.83 {
            let local_t = (t - 0.66) / 0.17;
            Color::new(0.5 - local_t * 0.5, local_t * 0.5, 1.0, 1.0)
        } else {
            let local_t = (t - 0.83) / 0.17;
            Color::new(local_t, 0.5 + local_t * 0.5, 1.0, 1.0)
        };

        EnemyConfig {
            health: base_health * health_decay,
            speed: base_speed * speed_growth,
            mass: base_mass * mass_decay,
            collision_radius: base_scale * scale_decay,
            visual_scale: base_scale * scale_decay,
            primitive_type: PrimitiveType::Dodecahedron,
            color,
        }
    }
}

pub fn on_death(data: &SplitterData, pos: Vec3) -> DeathEffect {
    if data.current_generation < data.max_generation {
        DeathEffect::Split {
            position: pos,
            current_generation: data.current_generation,
            max_generation: data.max_generation,
            count: 2,
        }
    } else {
        DeathEffect::None
    }
}
