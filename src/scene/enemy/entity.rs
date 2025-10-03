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

    pub fn set_position(&mut self, pos: Vec3) {
        self.pos = pos;
    }

    pub fn velocity(&self) -> Vec3 {
        self.vel
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
        self.take_damage_internal(damage, false);
    }

    /// Take damage bypassing snake segment invulnerability (for routed damage from head)
    pub fn take_damage_routed(&mut self, damage: f32) {
        self.take_damage_internal(damage, true);
    }

    /// Internal damage function with option to bypass invulnerability checks
    fn take_damage_internal(&mut self, damage: f32, bypass_segment_invulnerability: bool) {
        // ShieldOrbCore is invulnerable when protected by shields
        if !self.is_vulnerable() {
            return; // Invulnerable - no damage taken
        }

        // Snake segments are invulnerable to direct damage (unless bypassed for routed damage)
        if !bypass_segment_invulnerability && matches!(self.enemy_type, EnemyType::SnakeSegment(_)) {
            return;
        }

        self.health -= damage;
    }

    /// Route damage from snake head to its last segment (returns segment_id if damage was routed)
    pub fn route_snake_damage(&self, _damage: f32) -> Option<EntityId> {
        if let EnemyType::Snake(data) = &self.enemy_type {
            // If snake has segments, route damage to last segment
            if let Some(&last_segment_id) = data.segment_ids.last() {
                return Some(last_segment_id);
            }
            // No segments - damage will be applied to head normally
        }
        None
    }

    /// Remove a segment from snake's chain (called when segment dies)
    pub fn remove_segment(&mut self, segment_id: EntityId) {
        if let EnemyType::Snake(data) = &mut self.enemy_type {
            data.segment_ids.retain(|&id| id != segment_id);
        }
    }

    pub fn enemy_type(&self) -> &EnemyType {
        &self.enemy_type
    }

    pub fn enemy_type_mut(&mut self) -> &mut EnemyType {
        &mut self.enemy_type
    }

    pub fn config(&self) -> &EnemyConfig {
        &self.config
    }

    pub fn update_config(&mut self, new_config: EnemyConfig) {
        self.collision_radius = new_config.collision_radius;
        self.mass = new_config.mass;
        self.config = new_config;
        // Note: Don't update health/max_health to preserve current damage state
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

    /// Consume another enemy - heals to full, gains max health, grows in size
    pub fn consume_enemy(&mut self) {
        if let EnemyType::Cannibal(data) = &mut self.enemy_type {
            // Consume and get updated config
            let new_config = data.consume();

            // Update enemy stats
            self.config = new_config;
            self.health = self.config.health;
            self.max_health = self.config.health;
            self.collision_radius = self.config.collision_radius;

            println!("ENEMY CONSUMED!!");
        }
    }

    pub fn is_cannibal(&self) -> bool {
        matches!(self.enemy_type, EnemyType::Cannibal(..))
    }

    pub fn is_basic_enemy(&self) -> bool {
        matches!(
            self.enemy_type,
            EnemyType::Heavy | EnemyType::Chaser | EnemyType::Drone
        )
    }

    pub fn is_shield(&self) -> bool {
        matches!(self.enemy_type, EnemyType::Shield(..))
    }

    pub fn is_shield_orb_core(&self) -> bool {
        matches!(self.enemy_type, EnemyType::ShieldOrbCore(..))
    }

    /// Update shield orbital position around its core using spherical coordinates
    pub fn update_shield_orbit(&mut self, dt: f32, core_pos: Vec3) {
        if let EnemyType::Shield(data) = &mut self.enemy_type {
            let new_pos = data.update_orbit(dt, core_pos);
            self.pos = new_pos;
            self.vel = Vec3::zeros();
            self.applied_force = Vec3::zeros();
        }
    }

    /// Get the core_id if this is a shield
    pub fn shield_core_id(&self) -> Option<EntityId> {
        if let EnemyType::Shield(data) = &self.enemy_type {
            Some(data.core_id)
        } else {
            None
        }
    }

    /// Get the shield IDs if this is a ShieldOrbCore
    pub fn core_shield_ids(&self) -> Option<&Vec<EntityId>> {
        if let EnemyType::ShieldOrbCore(data) = &self.enemy_type {
            Some(&data.shield_ids)
        } else {
            None
        }
    }

    /// Check if core is vulnerable (can take damage)
    pub fn is_vulnerable(&self) -> bool {
        if let EnemyType::ShieldOrbCore(data) = &self.enemy_type {
            data.is_vulnerable
        } else {
            true // Non-cores are always vulnerable
        }
    }

    /// Update core vulnerability based on shield count
    pub fn update_vulnerability(&mut self, active_shield_count: usize) {
        if let EnemyType::ShieldOrbCore(data) = &mut self.enemy_type {
            data.update_vulnerability(active_shield_count);
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
