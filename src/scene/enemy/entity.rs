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
    eating_cooldown: f32, // Cooldown timer for Cannibal eating
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
            eating_cooldown: 0.0,
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

    pub fn eating_cooldown(&self) -> f32 {
        self.eating_cooldown
    }

    pub fn can_eat(&self) -> bool {
        matches!(self.enemy_type, EnemyType::Cannibal { .. }) && self.eating_cooldown <= 0.0
    }

    pub fn tick_eating_cooldown(&mut self, dt: f32) {
        if self.eating_cooldown > 0.0 {
            self.eating_cooldown -= dt;
        }
    }

    /// Consume another enemy - heals to full, gains max health, grows in size
    pub fn consume_enemy(&mut self) {
        if let EnemyType::Cannibal { meals_consumed } = &mut self.enemy_type {
            // Increment meal count
            let new_meals = (*meals_consumed + 1).min(10); // Cap at 10 meals
            self.enemy_type = EnemyType::Cannibal {
                meals_consumed: new_meals,
            };

            // Recalculate config with new meal count
            self.config = self.enemy_type.config();

            // Heal to full health
            self.health = self.config.health;
            self.max_health = self.config.health;

            // Update collision radius to match new size
            self.collision_radius = self.config.collision_radius;

            // Set eating cooldown (3 seconds between meals)
            self.eating_cooldown = 3.0;
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
