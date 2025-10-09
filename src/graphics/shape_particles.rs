//! Shape Particle System - Spawns 2D primitives as particles for visual effects
//!
//! This system manages particles that are rendered as 2D shapes (circles, squares, triangles, etc.)
//! rather than point sprites. Each particle has position, velocity, rotation, lifetime, and color.

use crate::graphics::{Color, PrimitiveType, Vec3};

/// Maximum number of shape particles in the system
const MAX_SHAPE_PARTICLES: usize = 512;

/// Individual shape particle data
#[derive(Debug, Clone)]
pub struct ShapeParticle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: Vec3,
    pub angular_velocity: Vec3,
    pub life: f32,
    pub max_life: f32,
    pub color: Color,
    pub primitive_type: PrimitiveType,
    pub scale: f32,
}

impl ShapeParticle {
    fn new(
        position: Vec3,
        velocity: Vec3,
        angular_velocity: Vec3,
        lifetime: f32,
        color: Color,
        primitive_type: PrimitiveType,
        scale: f32,
    ) -> Self {
        Self {
            position,
            velocity,
            rotation: Vec3::zeros(),
            angular_velocity,
            life: lifetime,
            max_life: lifetime,
            color,
            primitive_type,
            scale,
        }
    }

    /// Update particle physics and lifetime
    fn update(&mut self, dt: f32) {
        if self.life > 0.0 {
            // Update position
            self.position += self.velocity * dt;

            // Update rotation
            self.rotation += self.angular_velocity * dt;

            // Apply drag to velocity
            let drag = 0.98;
            self.velocity *= drag;

            // Decrease life
            self.life -= dt;
        }
    }

    /// Check if particle is still alive
    fn is_alive(&self) -> bool {
        self.life > 0.0
    }

    /// Get alpha based on lifetime (fade out near end)
    pub fn alpha(&self) -> f32 {
        if self.life < 0.5 {
            self.life / 0.5 // Fade out in last 0.5 seconds
        } else {
            1.0
        }
    }
}

/// Spawn request for shape particles
pub struct ShapeParticleSpawnRequest {
    pub position: Vec3,
    pub velocity: Vec3,
    pub count: u32,
    pub lifetime: f32,
    pub color: Color,
    pub primitive_type: PrimitiveType,
    pub angular_velocity: Vec3,
    pub scale: f32,
}

/// CPU-side shape particle system
///
/// Unlike the GPU particle system, this runs on the CPU and uses the existing
/// primitive rendering system, making it simpler and more flexible.
pub struct ShapeParticleSystem {
    particles: Vec<ShapeParticle>,
    spawn_queue: Vec<ShapeParticleSpawnRequest>,
}

impl ShapeParticleSystem {
    pub fn new() -> Self {
        Self {
            particles: Vec::with_capacity(MAX_SHAPE_PARTICLES),
            spawn_queue: Vec::new(),
        }
    }

    /// Queue a batch of shape particles to spawn
    pub fn spawn_particles(
        &mut self,
        position: Vec3,
        velocity: Vec3,
        count: u32,
        lifetime: f32,
        color: Color,
        primitive_type: PrimitiveType,
        angular_velocity: Vec3,
        scale: f32,
    ) {
        self.spawn_queue.push(ShapeParticleSpawnRequest {
            position,
            velocity,
            count,
            lifetime,
            color,
            primitive_type,
            angular_velocity,
            scale,
        });
    }

    /// Process spawn queue and create new particles
    fn process_spawn_queue(&mut self) {
        use rand::Rng;
        let mut rng = rand::rng();

        for request in self.spawn_queue.drain(..) {
            let available_slots = MAX_SHAPE_PARTICLES.saturating_sub(self.particles.len());
            let spawn_count = (request.count as usize).min(available_slots);

            for _ in 0..spawn_count {
                // Add random variation to velocity (spherical distribution)
                let theta = rng.random::<f32>() * 2.0 * std::f32::consts::PI;
                let phi = rng.random::<f32>() * std::f32::consts::PI;
                let spread_speed = rng.random::<f32>() * 5.0;

                let random_vel = Vec3::new(
                    phi.sin() * theta.cos() * spread_speed,
                    phi.sin() * theta.sin() * spread_speed,
                    phi.cos() * spread_speed,
                );

                let particle_velocity = request.velocity + random_vel;

                // Random angular velocity variation
                let random_angular = Vec3::new(
                    rng.random::<f32>() * 10.0 - 5.0,
                    rng.random::<f32>() * 10.0 - 5.0,
                    rng.random::<f32>() * 10.0 - 5.0,
                );

                let particle_angular = request.angular_velocity + random_angular;

                // Randomize lifetime slightly
                let lifetime_variation = rng.random::<f32>() * 0.5 - 0.25;
                let particle_lifetime = (request.lifetime + lifetime_variation).max(0.1);

                // Randomize scale slightly
                let scale_variation = rng.random::<f32>() * 0.2 - 0.1;
                let particle_scale = (request.scale + scale_variation).max(0.1);

                self.particles.push(ShapeParticle::new(
                    request.position,
                    particle_velocity,
                    particle_angular,
                    particle_lifetime,
                    request.color,
                    request.primitive_type,
                    particle_scale,
                ));
            }
        }
    }

    /// Update all shape particles
    pub fn update(&mut self, dt: f32) {
        // Process spawn queue
        self.process_spawn_queue();

        // Update all particles
        for particle in &mut self.particles {
            particle.update(dt);
        }

        // Remove dead particles
        self.particles.retain(|p| p.is_alive());
    }

    /// Get particles for rendering
    pub fn particles(&self) -> &[ShapeParticle] {
        &self.particles
    }

    /// Get particle count for debugging
    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }
}

impl Default for ShapeParticleSystem {
    fn default() -> Self {
        Self::new()
    }
}
