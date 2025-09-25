use crate::engine::EntityId;
use crate::graphics::{Color, Primitive, PrimitiveType, Vec3};

/// How the explosion force falls off with distance
#[derive(Debug, Clone, Copy)]
pub enum FalloffType {
    /// Force decreases linearly with distance
    Linear,
    /// Force decreases with square of distance
    Quadratic,
    /// Constant force within radius, zero outside
    Constant,
}

/// A single explosion effect
#[derive(Debug, Clone)]
pub struct Explosion {
    /// Center position of the explosion
    pub position: Vec3,
    /// Current blast radius (grows over time)
    pub current_radius: f32,
    /// Maximum blast radius
    pub max_radius: f32,
    /// Peak force magnitude at center
    pub force_strength: f32,
    /// Total explosion lifetime in seconds
    pub duration: f32,
    /// Time elapsed since explosion started
    pub elapsed_time: f32,
    /// How force decreases with distance
    pub falloff_type: FalloffType,

    /// Color of the explosion animation
    pub explosion_color: Color,
    pub particle_color: Color,
    pub particle_count: u32,

    /// Whether this explosion affects all object types
    pub affects_all: bool,
}

impl Explosion {
    /// Create a new explosion
    pub fn new(
        position: Vec3,
        max_radius: f32,
        force_strength: f32,
        duration: f32,
        falloff_type: FalloffType,
        explosion_color: Color,
        particle_color: Color,
        particle_count: u32,
    ) -> Self {
        Self {
            position,
            current_radius: 0.0,
            max_radius,
            force_strength,
            duration,
            elapsed_time: 0.0,
            falloff_type,
            explosion_color,
            particle_color,
            particle_count,

            affects_all: true,
        }
    }

    /// Update explosion state
    pub fn update(&mut self, delta_time: f32) {
        self.elapsed_time += delta_time;

        // Explosion radius grows linearly over time
        let progress = (self.elapsed_time / self.duration).min(1.0);
        self.current_radius = self.max_radius * progress;
    }

    /// Check if explosion is finished
    pub fn is_finished(&self) -> bool {
        self.elapsed_time >= self.duration
    }

    /// Calculate force magnitude at given distance from center
    pub fn force_at_distance(&self, distance: f32) -> f32 {
        if distance > self.current_radius {
            return 0.0;
        }

        // Time-based falloff (explosion weakens over time)
        let time_falloff = 1.0 - (self.elapsed_time / self.duration);

        // Distance-based falloff
        let distance_factor = match self.falloff_type {
            FalloffType::Linear => 1.0 - (distance / self.current_radius),
            FalloffType::Quadratic => {
                let normalized_distance = distance / self.current_radius;
                1.0 - normalized_distance * normalized_distance
            }
            FalloffType::Constant => 1.0,
        };

        self.force_strength * time_falloff * distance_factor
    }
}

/// Manages all active explosions in the game
pub struct ExplosionManager {
    /// List of active explosions
    explosions: Vec<Explosion>,
}

impl ExplosionManager {
    /// Create a new explosion manager
    pub fn new() -> Self {
        Self {
            explosions: Vec::new(),
        }
    }

    /// Spawn a new explosion
    pub fn spawn_explosion(
        &mut self,
        position: Vec3,
        max_radius: f32,
        force_strength: f32,
        duration: f32,
        falloff_type: FalloffType,
        explosion_color: Color,
        particle_color: Color,
        particle_count: u32,
    ) {
        let explosion = Explosion::new(
            position,
            max_radius,
            force_strength,
            duration,
            falloff_type,
            explosion_color,
            particle_color,
            particle_count,
        );
        self.explosions.push(explosion);
    }

    /// Spawn a shockwave explosion (for large body collisions)
    pub fn spawn_simple_shockwave(&mut self, position: Vec3) {
        self.spawn_explosion(
            position,
            50.0,   // Large radius
            1000.0, // Strong force
            0.3,    // Duration in seconds
            FalloffType::Quadratic,
            Color::ORANGE,
            Color::ORANGE,
            100,
        );
    }

    /// Spawn a solar wind explosion (for stars)
    pub fn spawn_solar_wind(&mut self, position: Vec3) {
        self.spawn_explosion(
            position,
            500.0, // Very large radius
            500.0, // Moderate force
            3.0,   // Short duration
            FalloffType::Linear,
            Color::ORANGE,
            Color::ORANGE,
            100,
        );
    }

    pub fn spawn_anti_wind(&mut self, position: Vec3) {
        self.spawn_explosion(
            position,
            500.0,  // Very large radius
            -500.0, // Moderate force
            3.0,    // Short duration
            FalloffType::Linear,
            Color::GREEN,
            Color::GREEN,
            100,
        );
    }

    /// Update all explosions
    pub fn update(&mut self, delta_time: f32) {
        // Update all explosions
        for explosion in &mut self.explosions {
            explosion.update(delta_time);
        }

        // Remove finished explosions
        self.explosions.retain(|explosion| !explosion.is_finished());
    }

    /// Get all active explosions (for physics calculations)
    pub fn explosions(&self) -> &[Explosion] {
        &self.explosions
    }

    /// Get render data for visual effects
    pub fn get_render_data(&self) -> Vec<Primitive> {
        let mut primitives = Vec::new();

        for explosion in &self.explosions {
            if explosion.current_radius > 0.1 {
                // Create expanding sphere primitive

                let alpha = 1.0 - (explosion.elapsed_time / explosion.duration); // Fade out over time

                // Get color from explosion, modify it to use alpha
                let mut color = explosion.explosion_color;
                color.a = alpha * 0.3;

                let primitive = Primitive::new(PrimitiveType::Sphere, explosion.position, color)
                    .with_uniform_scale(explosion.current_radius);

                primitives.push(primitive);
            }
        }

        primitives
    }

    /// Get number of active explosions
    pub fn explosion_count(&self) -> usize {
        self.explosions.len()
    }

    /// Clear all explosions (useful for testing)
    pub fn clear(&mut self) {
        self.explosions.clear();
    }

    /// Calculate explosion damage for enemies within range
    /// Returns list of (enemy_id, damage) pairs for scheduler to apply
    pub fn calculate_explosion_damage(
        &self,
        explosion_position: Vec3,
        damage: f32,
        damage_radius: f32,
        enemies: &[crate::scene::enemy::Enemy],
    ) -> Vec<(EntityId, f32)> {
        let mut damage_events = Vec::new();

        for enemy in enemies {
            let distance = (enemy.position() - explosion_position).magnitude();
            if distance <= damage_radius {
                // Calculate damage falloff based on distance (closer = more damage)
                let damage_falloff: f32 = if damage_radius > 0.0 {
                    1.0 - (distance / damage_radius)
                } else {
                    1.0 // Full damage if radius is 0
                };
                let actual_damage = damage * damage_falloff.max(0.0);

                if actual_damage > 0.0 {
                    damage_events.push((enemy.entity_id(), actual_damage));
                }
            }
        }

        damage_events
    }
}

impl Default for ExplosionManager {
    fn default() -> Self {
        Self::new()
    }
}
