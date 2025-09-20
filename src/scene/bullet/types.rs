use crate::engine::{CollisionMask, EntityId, Vec3};
use crate::engine::entity::EntityType;
use crate::scene::GravityAffected;
use super::visuals::BulletVisuals;
use super::effects::{OnHitEffect, OnExpireEffect, ProjectileEffects};

/// Unified projectile type system - defines what kind of projectile to spawn
#[derive(Debug, Clone)]
pub enum ProjectileType {
    /// Basic projectile - fast, lightweight, no special effects
    Basic {
        damage: f32,
        velocity: Vec3,
        lifetime: f32,
        mass: f32,
        visuals: BulletVisuals,
    },
    /// Custom projectile with arbitrary effects
    Custom {
        damage: f32,
        velocity: Vec3,
        lifetime: f32,
        mass: f32,
        effects: ProjectileEffects,
        visuals: BulletVisuals,
    },
    /// Seeking projectile that homes in on enemies and explodes on expire
    SeekingExplosive {
        damage: f32,
        velocity: Vec3,
        lifetime: f32,
        mass: f32,
        seeking_force: f32,
        seeking_range: f32,
        explosion_radius: f32,
        explosion_force: f32,
        explosion_duration: f32,
        visuals: BulletVisuals,
    },
}

/// Basic bullet struct for simple projectiles
pub struct Bullet {
    entity_id: EntityId,
    pos: Vec3,
    vel: Vec3,
    ttl: f32,
    damage: f32,
    collision_radius: f32,
    collision_mask: CollisionMask,
    marked_for_removal: bool,
    mass: f32,
    applied_force: Vec3,
    visuals: BulletVisuals,
}

impl Bullet {
    pub fn new(
        entity_id: EntityId,
        pos: Vec3,
        vel: Vec3,
        ttl: f32,
        damage: f32,
        mass: f32,
        visuals: BulletVisuals,
    ) -> Self {
        Bullet {
            entity_id,
            pos,
            vel,
            ttl,
            damage,
            collision_radius: 0.1,
            collision_mask: CollisionMask::from(EntityType::PlayerBullet),
            marked_for_removal: false,
            mass, // Custom mass - can be negative!
            applied_force: Vec3::zeros(),
            visuals,
        }
    }

    pub fn visuals(&self) -> &BulletVisuals {
        &self.visuals
    }

    pub fn update(&mut self, dt: f32) {
        // Apply gravitational forces (F = ma, so a = F/m)
        let gravity_acceleration = self.applied_force / self.mass;

        // Update velocity with gravity
        self.vel += gravity_acceleration * dt;

        // Update position
        self.pos += self.vel * dt;

        // Decrease time to live
        self.ttl -= dt;

        // Apply orbital decay effects (gravitational wave radiation simulation)
        self.apply_orbital_decay(dt);

        // Reset applied force for next frame
        self.applied_force = Vec3::zeros();
    }

    fn apply_orbital_decay(&mut self, dt: f32) {
        let velocity_decay_threshold = 15.0; // Above this speed, decay applies
        let current_speed = self.vel.magnitude();
        let mut decay_factor = 1.0;

        // High-speed bullets experience more drag (relativistic effects)
        if current_speed > velocity_decay_threshold {
            let excess_velocity_factor = (current_speed / velocity_decay_threshold - 1.0).min(1.5);
            let high_velocity_decay = 0.995_f32.powf(excess_velocity_factor * dt * 60.0);
            decay_factor *= high_velocity_decay;
        }

        // Stronger gravitational wave-like effects for close orbits
        let distance_from_origin = self.pos.magnitude();
        if distance_from_origin < 15.0 {
            // Larger effect radius
            let proximity_factor = (15.0 - distance_from_origin) / 15.0;
            let proximity_decay = 0.999_f32.powf(proximity_factor * dt * 60.0);
            decay_factor *= proximity_decay;
        }

        // Apply the calculated decay to velocity
        self.vel *= decay_factor;

        // If velocity becomes very small, consider the bullet "captured"
        if current_speed > 8.0 && self.vel.magnitude() < 1.0 {
            // More aggressive capture
            // Bullet has been significantly slowed down by orbital decay
            self.ttl = self.ttl.min(2.0); // Give it 2 seconds to live
        }
    }

    pub fn is_alive(&self) -> bool {
        self.ttl > 0.0 && !self.marked_for_removal
    }

    pub fn mark_for_removal(&mut self) {
        self.marked_for_removal = true;
    }

    pub fn position(&self) -> Vec3 {
        self.pos
    }

    pub fn velocity(&self) -> Vec3 {
        self.vel
    }

    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    pub fn damage(&self) -> f32 {
        self.damage
    }

    pub fn collision_radius(&self) -> f32 {
        self.collision_radius
    }

    pub fn collision_mask(&self) -> CollisionMask {
        self.collision_mask
    }

    pub fn set_velocity(&mut self, velocity: Vec3) {
        self.vel = velocity;
    }
}

impl GravityAffected for Bullet {
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

/// MetaBullet for complex projectiles with custom effects
pub struct MetaBullet {
    entity_id: EntityId,
    pos: Vec3,
    vel: Vec3,
    ttl: f32,
    damage: f32,
    collision_radius: f32,
    collision_mask: CollisionMask,
    marked_for_removal: bool,
    mass: f32,
    applied_force: Vec3,
    on_hit: Option<Vec<Box<dyn OnHitEffect>>>,
    on_expire: Option<Vec<Box<dyn OnExpireEffect>>>,
    seeking: bool,          // Whether this bullet seeks targets
    seeking_force: f32,     // Force strength for seeking behavior
    max_seeking_range: f32, // Maximum range for target acquisition
    visuals: BulletVisuals,
}

impl MetaBullet {
    pub fn new(
        entity_id: EntityId,
        pos: Vec3,
        vel: Vec3,
        ttl: f32,
        damage: f32,
        mass: f32,
        on_hit: Option<Vec<Box<dyn OnHitEffect>>>,
        on_expire: Option<Vec<Box<dyn OnExpireEffect>>>,
        visuals: BulletVisuals,
    ) -> Self {
        MetaBullet {
            entity_id,
            pos,
            vel,
            ttl,
            damage,
            collision_radius: 0.15,
            collision_mask: CollisionMask::from(EntityType::PlayerBullet),
            marked_for_removal: false,
            mass,
            applied_force: Vec3::zeros(),
            on_hit,
            on_expire,
            seeking: false,         // Default: no seeking
            seeking_force: 0.0,     // Default: no seeking force
            max_seeking_range: 0.0, // Default: no seeking range
            visuals,
        }
    }

    /// Create a seeking MetaBullet
    pub fn new_seeking(
        entity_id: EntityId,
        pos: Vec3,
        vel: Vec3,
        ttl: f32,
        damage: f32,
        mass: f32,
        on_hit: Option<Vec<Box<dyn OnHitEffect>>>,
        on_expire: Option<Vec<Box<dyn OnExpireEffect>>>,
        seeking_force: f32,
        max_seeking_range: f32,
        visuals: BulletVisuals,
    ) -> Self {
        MetaBullet {
            entity_id,
            pos,
            vel,
            ttl,
            damage,
            collision_radius: 0.15,
            collision_mask: CollisionMask::from(EntityType::PlayerBullet),
            marked_for_removal: false,
            mass,
            applied_force: Vec3::zeros(),
            on_hit,
            on_expire,
            seeking: true,
            seeking_force,
            max_seeking_range,
            visuals,
        }
    }

    pub fn visuals(&self) -> &BulletVisuals {
        &self.visuals
    }

    pub fn update(&mut self, dt: f32) {
        // Apply gravitational forces (F = ma, so a = F/m)
        let gravity_acceleration = self.applied_force / self.mass;

        // Update velocity with gravity
        self.vel += gravity_acceleration * dt;

        // Update position
        self.pos += self.vel * dt;

        // Decrease time to live
        self.ttl -= dt;

        // Apply orbital decay effects (gravitational wave radiation simulation)
        self.apply_orbital_decay(dt);

        // Reset applied force for next frame
        self.applied_force = Vec3::zeros();
    }

    fn apply_orbital_decay(&mut self, dt: f32) {
        let velocity_decay_threshold = 15.0; // Above this speed, decay applies
        let current_speed = self.vel.magnitude();
        let mut decay_factor = 1.0;

        // High-speed bullets experience more drag (relativistic effects)
        if current_speed > velocity_decay_threshold {
            let excess_velocity_factor = (current_speed / velocity_decay_threshold - 1.0).min(1.5);
            let high_velocity_decay = 0.995_f32.powf(excess_velocity_factor * dt * 60.0);
            decay_factor *= high_velocity_decay;
        }

        // Stronger gravitational wave-like effects for close orbits
        let distance_from_origin = self.pos.magnitude();
        if distance_from_origin < 15.0 {
            // Larger effect radius
            let proximity_factor = (15.0 - distance_from_origin) / 15.0;
            let proximity_decay = 0.999_f32.powf(proximity_factor * dt * 60.0);
            decay_factor *= proximity_decay;
        }

        // Apply the calculated decay to velocity
        self.vel *= decay_factor;

        // If velocity becomes very small, consider the bullet "captured"
        if current_speed > 8.0 && self.vel.magnitude() < 1.0 {
            // More aggressive capture
            // Bullet has been significantly slowed down by orbital decay
            self.ttl = self.ttl.min(2.0); // Give it 2 seconds to live
        }
    }

    pub fn is_alive(&self) -> bool {
        self.ttl > 0.0 && !self.marked_for_removal
    }

    pub fn mark_for_removal(&mut self) {
        self.marked_for_removal = true;
    }

    pub fn position(&self) -> Vec3 {
        self.pos
    }

    pub fn velocity(&self) -> Vec3 {
        self.vel
    }

    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    pub fn damage(&self) -> f32 {
        self.damage
    }

    pub fn collision_radius(&self) -> f32 {
        self.collision_radius
    }

    pub fn collision_mask(&self) -> CollisionMask {
        self.collision_mask
    }

    pub fn on_hit(&self) -> Option<&Vec<Box<dyn OnHitEffect>>> {
        self.on_hit.as_ref()
    }

    pub fn on_expire(&self) -> Option<&Vec<Box<dyn OnExpireEffect>>> {
        self.on_expire.as_ref()
    }

    pub fn seeking(&self) -> bool {
        self.seeking
    }

    pub fn seeking_force(&self) -> f32 {
        self.seeking_force
    }

    pub fn max_seeking_range(&self) -> f32 {
        self.max_seeking_range
    }

    pub fn set_velocity(&mut self, velocity: Vec3) {
        self.vel = velocity;
    }
}

impl GravityAffected for MetaBullet {
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