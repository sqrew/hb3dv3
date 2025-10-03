use super::super::types::EnemyConfig;
use super::DeathEffect;
use crate::engine::{Vec3, entity::EntityId};
use crate::graphics::{Color, PrimitiveType};

#[derive(Debug, Clone, PartialEq)]
pub struct ShieldData {
    pub current_generation: u8,
    pub max_generation: u8,
    pub core_id: EntityId,
    pub orbit_angle: f32,       // Azimuth angle (theta) - horizontal rotation
    pub orbit_inclination: f32, // Polar angle (phi) - vertical angle from pole
    pub orbit_radius: f32,
}

impl ShieldData {
    pub fn new(
        core_id: EntityId,
        orbit_angle: f32,
        orbit_inclination: f32,
        orbit_radius: f32,
        max_generation: u8,
    ) -> Self {
        Self {
            current_generation: 0,
            max_generation,
            core_id,
            orbit_angle,
            orbit_inclination,
            orbit_radius,
        }
    }

    pub fn next_generation(&self, new_orbit_angle: f32) -> Self {
        Self {
            current_generation: self.current_generation + 1,
            max_generation: self.max_generation,
            core_id: self.core_id,
            orbit_angle: new_orbit_angle,
            orbit_inclination: self.orbit_inclination,
            orbit_radius: self.orbit_radius,
        }
    }

    pub fn config(&self) -> EnemyConfig {
        let base_health = 100.0;
        let base_mass = 30.0;
        let base_scale = 5.0;

        // Scale down with generation
        let generation = self.current_generation as f32;
        let health_decay = 0.80_f32.powf(generation);
        let scale_decay = 0.70_f32.powf(generation);

        EnemyConfig {
            health: base_health * health_decay,
            speed: 0.0, // Shields don't use speed (they use orbital movement)
            mass: base_mass,
            collision_radius: base_scale * scale_decay,
            visual_scale: base_scale * scale_decay,
            primitive_type: PrimitiveType::Octahedron,
            color: Color::new(0.2, 0.5, 1.0, 1.0), // Bright blue
        }
    }

    /// Update orbital position around core
    pub fn update_orbit(&mut self, dt: f32, core_pos: Vec3) -> Vec3 {
        // Rotate around the core
        let rotation_speed = 1.0; // radians per second
        self.orbit_angle += rotation_speed * dt;

        // Wrap angle to 0-2π
        if self.orbit_angle > std::f32::consts::TAU {
            self.orbit_angle -= std::f32::consts::TAU;
        }

        // Calculate 3D orbital position using spherical coordinates
        let sin_phi = self.orbit_inclination.sin();
        let x = core_pos.x + self.orbit_radius * sin_phi * self.orbit_angle.cos();
        let y = core_pos.y + self.orbit_radius * self.orbit_inclination.cos();
        let z = core_pos.z + self.orbit_radius * sin_phi * self.orbit_angle.sin();

        Vec3::new(x, y, z)
    }
}

pub fn on_death(data: &ShieldData, pos: Vec3) -> DeathEffect {
    if data.current_generation < data.max_generation {
        DeathEffect::SpawnShields {
            position: pos,
            current_generation: data.current_generation,
            max_generation: data.max_generation,
            core_id: data.core_id,
            orbit_angle: data.orbit_angle,
            orbit_inclination: data.orbit_inclination,
            orbit_radius: data.orbit_radius,
        }
    } else {
        DeathEffect::None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShieldOrbCoreData {
    pub shield_ids: Vec<EntityId>,
    pub is_vulnerable: bool,
}

impl ShieldOrbCoreData {
    pub fn new(shield_ids: Vec<EntityId>) -> Self {
        Self {
            shield_ids,
            is_vulnerable: false,
        }
    }

    pub fn update_vulnerability(&mut self, active_shield_count: usize) {
        let initial_shield_count = self.shield_ids.len();
        self.is_vulnerable = if initial_shield_count == 0 {
            true
        } else {
            let shield_percentage = active_shield_count as f32 / initial_shield_count as f32;
            shield_percentage <= 0.5
        };
    }

    pub fn config(&self) -> EnemyConfig {
        // Color changes based on vulnerability state
        let color = if self.is_vulnerable {
            Color::new(0.2, 0.5, 1.0, 1.0) // Blue when vulnerable
        } else {
            Color::new(1.0, 0.2, 0.2, 1.0) // Red when invulnerable
        };

        EnemyConfig {
            health: 500.0,
            speed: 25.0,
            mass: 150.0,
            collision_radius: 3.0,
            visual_scale: 3.0,
            primitive_type: PrimitiveType::Sphere,
            color,
        }
    }
}

pub fn core_on_death(_pos: Vec3) -> DeathEffect {
    DeathEffect::None
}
