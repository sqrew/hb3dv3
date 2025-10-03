use super::effects::{OnExpireEffect, OnHitEffect, ProjectileEffects};
use super::visuals::BulletVisuals;
use crate::engine::entity::EntityType;
use crate::engine::{CollisionMask, EntityId, Vec3};
use crate::scene::GravityAffected;
use rand::Rng;
use std::collections::VecDeque;

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
    /// Implosion projectile that explodes with negative force on hit OR expire
    ImplosionExplosive {
        damage: f32,
        velocity: Vec3,
        lifetime: f32,
        mass: f32,
        explosion_radius: f32,
        explosion_force: f32, // Negative value for implosion
        explosion_duration: f32,
        visuals: BulletVisuals,
    },
    /// Fractal projectile that splits into mathematical patterns
    Fractal {
        damage: f32,
        velocity: Vec3,
        lifetime: f32,
        mass: f32,
        fractal_config: super::fractal::FractalConfig,
        visuals: BulletVisuals,
    },
    /// Laser projectile that leaves a visible trail and is affected by gravity
    Laser {
        damage: f32,
        velocity: Vec3, // Very high speed (e.g., 5000.0)
        lifetime: f32,
        mass: f32,               // Very small mass (e.g., 1e-6)
        max_trail_length: usize, // Number of trail points to keep
        trail_fade_rate: f32,    // How quickly trail fades (0.0-1.0)
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
    // Optional fractal metadata for bullets that can split
    fractal_data: Option<FractalBulletData>,
    // Optional laser trail data
    trail_data: Option<LaserTrailData>,
    // Pooling support
    active: bool,
}

/// Fractal data that can be attached to regular bullets to enable splitting
#[derive(Debug, Clone)]
pub struct FractalBulletData {
    pub config: super::fractal::FractalConfig,
    pub generation: usize,
    pub time_until_split: f32,
    pub has_split: bool,
}

/// Laser trail data for rendering curved laser beams
#[derive(Debug, Clone)]
pub struct LaserTrailData {
    pub trail_points: VecDeque<Vec3>,
    pub max_trail_length: usize,
    pub trail_fade_rate: f32,
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
            collision_radius: 0.5,
            collision_mask: CollisionMask::from(EntityType::PlayerBullet),
            marked_for_removal: false,
            mass, // Custom mass - can be negative!
            applied_force: Vec3::zeros(),
            visuals,
            fractal_data: None,
            trail_data: None,
            active: true,
        }
    }

    /// Create a fractal bullet that can split into multiple children
    pub fn new_fractal(
        entity_id: EntityId,
        pos: Vec3,
        vel: Vec3,
        ttl: f32,
        damage: f32,
        mass: f32,
        visuals: BulletVisuals,
        fractal_config: super::fractal::FractalConfig,
        generation: usize,
    ) -> Self {
        let time_until_split = if generation == 0 {
            fractal_config.split_delay
        } else {
            fractal_config.split_delay
        };

        Bullet {
            entity_id,
            pos,
            vel,
            ttl,
            damage,
            collision_radius: 1.0,
            collision_mask: CollisionMask::from(EntityType::PlayerBullet),
            marked_for_removal: false,
            mass,
            applied_force: Vec3::zeros(),
            visuals,
            fractal_data: Some(FractalBulletData {
                config: fractal_config,
                generation,
                time_until_split,
                has_split: false,
            }),
            trail_data: None,
            active: true,
        }
    }

    /// Create a laser bullet with trail rendering
    pub fn new_laser(
        entity_id: EntityId,
        pos: Vec3,
        vel: Vec3,
        ttl: f32,
        damage: f32,
        mass: f32,
        visuals: BulletVisuals,
        max_trail_length: usize,
        trail_fade_rate: f32,
    ) -> Self {
        let mut trail_points = VecDeque::new();
        trail_points.push_back(pos); // Start with current position

        Bullet {
            entity_id,
            pos,
            vel,
            ttl,
            damage,
            collision_radius: 0.5,
            collision_mask: CollisionMask::from(EntityType::PlayerBullet),
            marked_for_removal: false,
            mass,
            applied_force: Vec3::zeros(),
            visuals,
            fractal_data: None,
            trail_data: Some(LaserTrailData {
                trail_points,
                max_trail_length,
                trail_fade_rate,
            }),
            active: true,
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

        // Update fractal split timer if this is a fractal bullet
        if let Some(ref mut fractal_data) = self.fractal_data {
            if !fractal_data.has_split && fractal_data.generation < fractal_data.config.max_depth {
                fractal_data.time_until_split -= dt;
            }
        }

        // Update laser trail if this is a laser bullet
        if let Some(ref mut trail_data) = self.trail_data {
            // Add current position to trail
            trail_data.trail_points.push_back(self.pos);

            // Remove old trail points if we exceed max length
            while trail_data.trail_points.len() > trail_data.max_trail_length {
                trail_data.trail_points.pop_front();
            }
        }

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
        self.active && self.ttl > 0.0 && !self.marked_for_removal
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

    pub fn ttl(&self) -> f32 {
        self.ttl
    }

    pub fn collision_radius(&self) -> f32 {
        self.collision_radius
    }

    pub fn collision_mask(&self) -> CollisionMask {
        self.collision_mask
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn set_velocity(&mut self, velocity: Vec3) {
        self.vel = velocity;
    }

    /// Check if this bullet should split now (for fractal bullets only)
    pub fn should_split(&self, dt: f32) -> bool {
        if !self.active {
            return false;
        }
        if let Some(ref fractal_data) = self.fractal_data {
            if fractal_data.has_split || fractal_data.generation >= fractal_data.config.max_depth {
                return false;
            }
            fractal_data.time_until_split <= dt
        } else {
            false
        }
    }

    /// Mark this bullet as having split (for fractal bullets only)
    pub fn mark_split(&mut self) {
        if let Some(ref mut fractal_data) = self.fractal_data {
            fractal_data.has_split = true;
        }
    }

    /// Get fractal data (for fractal bullets only)
    pub fn fractal_data(&self) -> Option<&FractalBulletData> {
        self.fractal_data.as_ref()
    }

    /// Get trail data (for laser bullets only)
    pub fn trail_data(&self) -> Option<&LaserTrailData> {
        self.trail_data.as_ref()
    }

    /// Check if this is a laser bullet
    pub fn is_laser(&self) -> bool {
        self.trail_data.is_some()
    }

    /// Create a child fractal bullet from this parent
    pub fn create_fractal_child(
        &self,
        entity_id: EntityId,
        split_direction: Vec3,
    ) -> Option<Bullet> {
        if let Some(ref fractal_data) = self.fractal_data {
            // Calculate new velocity using the 3D split direction
            let parent_speed = self.vel.magnitude();
            let new_velocity = split_direction.normalize()
                * parent_speed
                * fractal_data.config.velocity_inheritance;

            // Offset child position in the split direction to prevent immediate collisions
            // Use dynamic separation based on pattern complexity to prevent immediate sibling collisions
            let base_separation = 1.5; // Base separation distance
            let pattern_multiplier = match fractal_data.config.pattern {
                super::fractal::FractalPattern::BinaryTree
                | super::fractal::FractalPattern::DragonCurve
                | super::fractal::FractalPattern::FibonacciSpiral => 1.0, // 2-way splits
                super::fractal::FractalPattern::SierpinskiTriangle
                | super::fractal::FractalPattern::HelixSpiral3D => 1.2, // 3-way splits
                super::fractal::FractalPattern::KochSnowflake
                | super::fractal::FractalPattern::Tetrahedron3D => 1.5, // 4-way splits
                super::fractal::FractalPattern::Octahedron3D => 2.0, // 6-way splits
                super::fractal::FractalPattern::Cube3D
                | super::fractal::FractalPattern::SphereExplosion3D => 2.5, // 8+ way splits
                super::fractal::FractalPattern::Icosahedron3D => 3.0, // 12-way splits - maximum separation
            };
            let separation_distance = base_separation * pattern_multiplier;
            let child_position = self.pos + split_direction.normalize() * separation_distance;

            // Child gets smaller visuals
            let mut child_visuals = self.visuals.clone();
            child_visuals.scale *= fractal_data.config.size_decay;

            // Randomize child lifetime to prevent simultaneous disappearance
            // Base lifetime is 80% of parent, but randomly vary it by ±25%
            let mut rng = rand::rng();
            let base_child_lifetime = self.ttl * 0.8;
            let lifetime_variation = 0.25; // ±25% variation
            let random_factor =
                rng.random_range(1.0 - lifetime_variation..=1.0 + lifetime_variation);
            let randomized_lifetime = base_child_lifetime * random_factor;

            Some(Bullet::new_fractal(
                entity_id,
                child_position,
                new_velocity,
                randomized_lifetime, // Randomized individual lifetime
                self.damage * 0.7,   // Reduced damage per generation
                self.mass * 0.8,     // Lighter children
                child_visuals,
                fractal_data.config.clone(),
                fractal_data.generation + 1,
            ))
        } else {
            None
        }
    }

    /// Get the split directions for creating children (for fractal bullets only)
    pub fn get_split_directions(&self) -> Vec<Vec3> {
        if let Some(ref fractal_data) = self.fractal_data {
            fractal_data
                .config
                .pattern
                .get_split_directions(self.vel.normalize(), fractal_data.generation)
        } else {
            Vec::new()
        }
    }

    /// Reset bullet for pooling reuse
    pub fn reset(
        &mut self,
        entity_id: EntityId,
        pos: Vec3,
        vel: Vec3,
        ttl: f32,
        damage: f32,
        mass: f32,
        visuals: BulletVisuals,
    ) {
        self.entity_id = entity_id;
        self.pos = pos;
        self.vel = vel;
        self.ttl = ttl;
        self.damage = damage;
        self.mass = mass;
        self.visuals = visuals;
        self.marked_for_removal = false;
        self.applied_force = Vec3::zeros();
        self.fractal_data = None;
        self.trail_data = None;
        self.active = true;
    }

    /// Reset bullet as fractal for pooling reuse
    pub fn reset_fractal(
        &mut self,
        entity_id: EntityId,
        pos: Vec3,
        vel: Vec3,
        ttl: f32,
        damage: f32,
        mass: f32,
        visuals: BulletVisuals,
        fractal_config: super::fractal::FractalConfig,
        generation: usize,
    ) {
        self.entity_id = entity_id;
        self.pos = pos;
        self.vel = vel;
        self.ttl = ttl;
        self.damage = damage;
        self.mass = mass;
        self.visuals = visuals;
        self.marked_for_removal = false;
        self.applied_force = Vec3::zeros();
        self.trail_data = None;

        let time_until_split = if generation == 0 {
            fractal_config.split_delay
        } else {
            fractal_config.split_delay
        };

        self.fractal_data = Some(FractalBulletData {
            config: fractal_config,
            generation,
            time_until_split,
            has_split: false,
        });
        self.active = true;
    }

    /// Reset bullet as laser for pooling reuse
    pub fn reset_laser(
        &mut self,
        entity_id: EntityId,
        pos: Vec3,
        vel: Vec3,
        ttl: f32,
        damage: f32,
        mass: f32,
        visuals: BulletVisuals,
        max_trail_length: usize,
        trail_fade_rate: f32,
    ) {
        self.entity_id = entity_id;
        self.pos = pos;
        self.vel = vel;
        self.ttl = ttl;
        self.damage = damage;
        self.mass = mass;
        self.visuals = visuals;
        self.marked_for_removal = false;
        self.applied_force = Vec3::zeros();
        self.fractal_data = None;

        let mut trail_points = VecDeque::new();
        trail_points.push_back(pos);

        self.trail_data = Some(LaserTrailData {
            trail_points,
            max_trail_length,
            trail_fade_rate,
        });
        self.active = true;
    }

    /// Deactivate bullet for pooling reuse
    pub fn deactivate(&mut self) {
        self.active = false;
        self.marked_for_removal = true;
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
    // Pooling support
    active: bool,
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
            collision_radius: 0.5,
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
            active: true,
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
            collision_radius: 0.5,
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
            active: true,
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
        self.active && self.ttl > 0.0 && !self.marked_for_removal
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

    pub fn active(&self) -> bool {
        self.active
    }

    /// Reset MetaBullet for pooling reuse
    pub fn reset(
        &mut self,
        entity_id: EntityId,
        pos: Vec3,
        vel: Vec3,
        ttl: f32,
        damage: f32,
        mass: f32,
        on_hit: Option<Vec<Box<dyn OnHitEffect>>>,
        on_expire: Option<Vec<Box<dyn OnExpireEffect>>>,
        visuals: BulletVisuals,
    ) {
        self.entity_id = entity_id;
        self.pos = pos;
        self.vel = vel;
        self.ttl = ttl;
        self.damage = damage;
        self.mass = mass;
        self.on_hit = on_hit;
        self.on_expire = on_expire;
        self.visuals = visuals;
        self.marked_for_removal = false;
        self.applied_force = Vec3::zeros();
        self.seeking = false;
        self.seeking_force = 0.0;
        self.max_seeking_range = 0.0;
        self.active = true;
    }

    /// Reset MetaBullet for seeking projectile pooling reuse
    pub fn reset_seeking(
        &mut self,
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
    ) {
        self.entity_id = entity_id;
        self.pos = pos;
        self.vel = vel;
        self.ttl = ttl;
        self.damage = damage;
        self.mass = mass;
        self.on_hit = on_hit;
        self.on_expire = on_expire;
        self.visuals = visuals;
        self.marked_for_removal = false;
        self.applied_force = Vec3::zeros();
        self.seeking = true;
        self.seeking_force = seeking_force;
        self.max_seeking_range = max_seeking_range;
        self.active = true;
    }

    /// Deactivate MetaBullet for pooling reuse
    pub fn deactivate(&mut self) {
        self.active = false;
        self.marked_for_removal = true;
        // Clear effects to prevent memory leaks
        self.on_hit = None;
        self.on_expire = None;
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
