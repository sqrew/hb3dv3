use crate::engine::entity::{EntityId, EntityType};
use crate::engine::{CollisionMask, Vec3};
use crate::scene::GravityAffected;
use super::types::{EnemyType, EnemyConfig};

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
    pub fn new(
        entity_id: EntityId,
        pos: Vec3,
        vel: Vec3,
        enemy_type: EnemyType,
    ) -> Self {
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
        // Apply gravitational forces (F = ma, so a = F/m)
        let gravity_acceleration = self.applied_force / self.mass;

        // Player-seeking AI: calculate direction to player
        let to_player = player_pos - self.pos;
        let distance_to_player = to_player.magnitude();

        // Seek player at any distance (but avoid division by zero)
        if distance_to_player > 0.1 {
            let seek_direction = to_player.normalize();

            // Apply seeking velocity using config speed (but don't override gravity completely)
            let seek_acceleration = seek_direction * self.config.speed;
            self.vel += seek_acceleration * dt;
        }

        // Update velocity with both AI movement and gravity
        self.vel += gravity_acceleration * dt;

        // Apply drag only when not under strong external forces
        let drag_factor = if self.applied_force.magnitude() > self.config.speed * 2.0 {
            0.995 // Minimal drag when under strong forces (explosions)
        } else {
            0.98 // Normal drag for AI movement
        };
        self.vel *= drag_factor;

        // Update position
        self.pos += self.vel * dt;

        // Boundary handling: bounce off far boundaries only if not under strong gravitational influence
        if self.pos.x.abs() > 50.0 && self.applied_force.magnitude() < 10.0 {
            self.vel.x = -self.vel.x * 0.8; // Reduce velocity on bounce
        }
        if self.pos.z.abs() > 50.0 && self.applied_force.magnitude() < 10.0 {
            self.vel.z = -self.vel.z * 0.8;
        }
        if self.pos.y.abs() > 50.0 && self.applied_force.magnitude() < 10.0 {
            self.vel.y = -self.vel.y * 0.8;
        }

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
        self.health -= damage;
    }

    pub fn enemy_type(&self) -> EnemyType {
        self.enemy_type
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