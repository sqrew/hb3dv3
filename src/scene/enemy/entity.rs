use super::types::{EnemyConfig, EnemyType};
use crate::engine::entity::{EntityId, EntityType};
use crate::engine::{CollisionMask, Vec3};
use crate::scene::GravityAffected;

pub struct Enemy {
    entity_id: EntityId,
    pos: Vec3,
    vel: Vec3,
    health: f32,
    max_health: f32,
    collision_radius: f32,
    collision_mask: CollisionMask,
    mass: f32,
    applied_force: Vec3,
    enemy_type: EnemyType,
    config: EnemyConfig,
}

impl Enemy {
    pub fn new(entity_id: EntityId, pos: Vec3, vel: Vec3, enemy_type: EnemyType) -> Self {
        let config = enemy_type.config();

        Enemy {
            entity_id,
            pos,
            vel,
            health: config.health,
            max_health: config.health,
            collision_radius: config.collision_radius,
            collision_mask: CollisionMask::from(EntityType::Enemy),
            mass: config.mass,
            applied_force: Vec3::zeros(),
            enemy_type,
            config,
        }
    }

    pub fn update(&mut self, dt: f32, player_pos: Vec3) {
        self.update_with_target(dt, player_pos);
    }

    pub fn update_with_target(&mut self, dt: f32, target_pos: Vec3) {
        // Apply gravitational forces (F = ma, so a = F/m)
        let gravity_acceleration = self.applied_force / self.mass;

        // Seeking AI: calculate direction to target (player or prey)
        let to_target = target_pos - self.pos;
        let distance_to_target = to_target.magnitude();

        // Seek target at any distance (but avoid division by zero)
        if distance_to_target > 0.01 {
            // Much closer threshold - keep seeking until very close
            let seek_direction = to_target.normalize();

            // Boost AI seeking when no gravitational forces are present
            let seeking_multiplier = if self.applied_force.magnitude() < 0.1 {
                3.0 // Much stronger seeking when no gravity to overcome increased drag
            } else {
                1.0 // Normal seeking when gravity is present
            };

            let seek_acceleration = seek_direction * self.config.speed * seeking_multiplier;
            self.vel += seek_acceleration * dt;
        }

        // Update velocity with both AI movement and gravity
        self.vel += gravity_acceleration * dt;

        // Apply drag - use stronger drag when no gravitational forces are present
        let drag_factor = if self.applied_force.magnitude() > self.config.speed * 2.0 {
            0.995 // Minimal drag when under strong forces (explosions)
        } else if self.applied_force.magnitude() < 0.1 {
            0.90 // Strong drag when no gravitational forces (helps AI regain control)
        } else {
            0.98 // Normal drag for AI movement
        };
        self.vel *= drag_factor;

        // Update position
        self.pos += self.vel * dt;

        // Reset applied force for next frame
        self.applied_force = Vec3::zeros();
    }

    pub fn position(&self) -> Vec3 {
        self.pos
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0.0
    }

    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    pub fn collision_radius(&self) -> f32 {
        self.collision_radius
    }

    pub fn collision_mask(&self) -> CollisionMask {
        self.collision_mask
    }

    pub fn take_damage(&mut self, damage: f32) {
        // ShieldOrbCore is invulnerable when protected by shields
        if !self.is_vulnerable() {
            return; // Invulnerable - no damage taken
        }
        self.health -= damage;
    }

    pub fn enemy_type(&self) -> &EnemyType {
        &self.enemy_type
    }

    pub fn config(&self) -> &EnemyConfig {
        &self.config
    }

    pub fn health(&self) -> f32 {
        self.health
    }

    pub fn max_health(&self) -> f32 {
        self.max_health
    }

    pub fn health_percentage(&self) -> f32 {
        if self.max_health > 0.0 {
            self.health / self.max_health
        } else {
            0.0
        }
    }

    pub fn eating_cooldown(&self) -> f32 {
        if let EnemyType::Cannibal { eating_cooldown, .. } = &self.enemy_type {
            *eating_cooldown
        } else {
            0.0
        }
    }

    pub fn can_eat(&self) -> bool {
        if let EnemyType::Cannibal { eating_cooldown, .. } = &self.enemy_type {
            *eating_cooldown <= 0.0
        } else {
            false
        }
    }

    pub fn tick_eating_cooldown(&mut self, dt: f32) {
        if let EnemyType::Cannibal {
            meals_consumed: _,
            eating_cooldown,
        } = &mut self.enemy_type
        {
            if *eating_cooldown > 0.0 {
                *eating_cooldown -= dt;
            }
        }
    }

    /// Consume another enemy - heals to full, gains max health, grows in size
    pub fn consume_enemy(&mut self) {
        if let EnemyType::Cannibal { meals_consumed, eating_cooldown } = &mut self.enemy_type {
            // Increment meal count
            let new_meals = (*meals_consumed + 1).min(10); // Cap at 10 meals

            // Set eating cooldown (3 seconds between meals)
            *eating_cooldown = 3.0;

            self.enemy_type = EnemyType::Cannibal {
                meals_consumed: new_meals,
                eating_cooldown: *eating_cooldown,
            };

            // Recalculate config with new meal count
            self.config = self.enemy_type.config();

            // Heal to full health
            self.health = self.config.health;
            self.max_health = self.config.health;

            // Update collision radius to match new size
            self.collision_radius = self.config.collision_radius;

            println!("ENEMY CONSUMED!!");
        }
    }

    pub fn is_cannibal(&self) -> bool {
        matches!(self.enemy_type, EnemyType::Cannibal { .. })
    }

    pub fn is_basic_enemy(&self) -> bool {
        matches!(
            self.enemy_type,
            EnemyType::Heavy | EnemyType::Chaser | EnemyType::Drone
        )
    }

    pub fn is_shield(&self) -> bool {
        matches!(self.enemy_type, EnemyType::Shield { .. })
    }

    pub fn is_shield_orb_core(&self) -> bool {
        matches!(self.enemy_type, EnemyType::ShieldOrbCore { .. })
    }

    /// Update shield orbital position around its core using spherical coordinates
    pub fn update_shield_orbit(&mut self, dt: f32, core_pos: Vec3) {
        if let EnemyType::Shield {
            current_generation: _,
            max_generation: _,
            core_id: _,
            orbit_angle,
            orbit_inclination,
            orbit_radius,
        } = &mut self.enemy_type
        {
            // Rotate around the core (azimuthal angle)
            let rotation_speed = 1.0; // radians per second
            *orbit_angle += rotation_speed * dt;

            // Wrap angle to 0-2π
            if *orbit_angle > std::f32::consts::TAU {
                *orbit_angle -= std::f32::consts::TAU;
            }

            // Calculate 3D orbital position using spherical coordinates
            // x = r * sin(phi) * cos(theta)
            // y = r * cos(phi)
            // z = r * sin(phi) * sin(theta)
            let sin_phi = orbit_inclination.sin();
            let x = core_pos.x + *orbit_radius * sin_phi * orbit_angle.cos();
            let y = core_pos.y + *orbit_radius * orbit_inclination.cos();
            let z = core_pos.z + *orbit_radius * sin_phi * orbit_angle.sin();

            self.pos = Vec3::new(x, y, z);
            self.vel = Vec3::zeros(); // Shields don't have velocity, they just orbit

            // Reset applied force for next frame
            self.applied_force = Vec3::zeros();
        }
    }

    /// Get the core_id if this is a shield
    pub fn shield_core_id(&self) -> Option<EntityId> {
        if let EnemyType::Shield { core_id, .. } = &self.enemy_type {
            Some(*core_id)
        } else {
            None
        }
    }

    /// Get the shield IDs if this is a ShieldOrbCore
    pub fn core_shield_ids(&self) -> Option<&Vec<EntityId>> {
        if let EnemyType::ShieldOrbCore { shield_ids, .. } = &self.enemy_type {
            Some(shield_ids)
        } else {
            None
        }
    }

    /// Check if core is vulnerable (can take damage)
    pub fn is_vulnerable(&self) -> bool {
        if let EnemyType::ShieldOrbCore { is_vulnerable, .. } = &self.enemy_type {
            *is_vulnerable
        } else {
            true // Non-cores are always vulnerable
        }
    }

    /// Update core vulnerability based on shield count
    pub fn update_vulnerability(&mut self, active_shield_count: usize) {
        if let EnemyType::ShieldOrbCore {
            shield_ids,
            is_vulnerable,
        } = &mut self.enemy_type
        {
            // Core becomes vulnerable when 50% or more shields are destroyed
            let initial_shield_count = shield_ids.len();
            *is_vulnerable = if initial_shield_count == 0 {
                true // No shields = vulnerable
            } else {
                let shield_percentage = active_shield_count as f32 / initial_shield_count as f32;
                shield_percentage <= 0.5
            };

            // Update config to reflect new color based on vulnerability
            self.config = self.enemy_type.config();
        }
    }
}

impl GravityAffected for Enemy {
    fn position(&self) -> Vec3 {
        self.pos
    }

    fn mass(&self) -> f32 {
        self.mass
    }

    fn apply_force(&mut self, force: Vec3) {
        self.applied_force += force;
    }
}
