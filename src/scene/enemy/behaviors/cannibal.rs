use crate::engine::Vec3;
use crate::graphics::{Color, PrimitiveType};
use super::super::types::EnemyConfig;
use super::super::entity::Enemy;
use super::DeathEffect;

#[derive(Debug, Clone, PartialEq)]
pub struct CannibalData {
    pub meals_consumed: u8,
    pub eating_cooldown: f32,
}

impl CannibalData {
    pub fn new() -> Self {
        Self {
            meals_consumed: 0,
            eating_cooldown: 0.0,
        }
    }

    pub fn can_eat(&self) -> bool {
        self.eating_cooldown <= 0.0
    }

    pub fn tick_cooldown(&mut self, dt: f32) {
        if self.eating_cooldown > 0.0 {
            self.eating_cooldown -= dt;
        }
    }

    pub fn consume(&mut self) -> EnemyConfig {
        // Increment meal count (cap at 10)
        self.meals_consumed = (self.meals_consumed + 1).min(10);

        // Set eating cooldown (3 seconds between meals)
        self.eating_cooldown = 3.0;

        // Return updated config
        self.config()
    }

    pub fn config(&self) -> EnemyConfig {
        let base_health = 1000.0;
        let base_speed = 200.0;
        let base_mass = 100.0;
        let base_scale = 3.0;

        // Growth per meal consumed (max 10 meals for balance)
        let meals = self.meals_consumed.min(10) as f32;
        let max_health_bonus = meals * 100.0; // +100 max HP per meal
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

/// Find the nearest basic enemy for hunting
pub fn find_prey_target(hunter_pos: Vec3, all_enemies: &[Enemy], hunter_id: crate::engine::entity::EntityId) -> Option<Vec3> {
    let mut nearest_distance_sq = f32::MAX;
    let mut target_pos = None;

    for enemy in all_enemies {
        if enemy.entity_id() != hunter_id && enemy.is_basic_enemy() {
            let prey_pos = enemy.position();
            let diff = prey_pos - hunter_pos;
            let dist_sq = diff.magnitude_squared();

            if dist_sq < nearest_distance_sq {
                nearest_distance_sq = dist_sq;
                target_pos = Some(prey_pos);
            }
        }
    }

    target_pos
}

/// Find prey within eating range (returns enemy index)
pub fn find_prey_in_range(hunter_pos: Vec3, all_enemies: &[Enemy], hunter_id: crate::engine::entity::EntityId) -> Option<usize> {
    let mut nearest_distance_sq = f32::MAX;
    let mut nearest_index = None;
    let eating_range_sq = 4.0; // 2.0 units radius squared

    for (idx, enemy) in all_enemies.iter().enumerate() {
        if enemy.entity_id() != hunter_id && enemy.is_basic_enemy() {
            let prey_pos = enemy.position();
            let diff = prey_pos - hunter_pos;
            let dist_sq = diff.magnitude_squared();

            if dist_sq < eating_range_sq && dist_sq < nearest_distance_sq {
                nearest_distance_sq = dist_sq;
                nearest_index = Some(idx);
            }
        }
    }

    nearest_index
}

pub fn on_death(_pos: Vec3) -> DeathEffect {
    DeathEffect::None
}
