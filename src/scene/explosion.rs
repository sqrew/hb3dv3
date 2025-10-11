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
    /// Force increases linearly with distance (inverted)
    InverseLinear,
    /// Force increases with square of distance (inverted quadratic)
    InverseQuadratic,
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

    /// Whether to render expanding torus shockwave rings
    pub has_rings: bool,
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
            has_rings: false,
        }
    }

    /// Enable expanding torus shockwave rings for this explosion
    pub fn with_rings(mut self) -> Self {
        self.has_rings = true;
        self
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
            FalloffType::InverseLinear => distance / self.current_radius, // Stronger at edges
            FalloffType::InverseQuadratic => {
                let normalized_distance = distance / self.current_radius;
                normalized_distance * normalized_distance // Much stronger at edges
            }
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

    /// Spawn a new explosion and return a mutable reference to it for chaining
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
    ) -> &mut Explosion {
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
        self.explosions.last_mut().unwrap()
    }

    /// Spawn a new explosion with torus shockwave rings enabled
    pub fn spawn_explosion_with_rings(
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
        )
        .with_rings();
        self.explosions.push(explosion);
    }

    /// Spawn a shockwave explosion (for large body collisions) - no rings by default
    pub fn spawn_simple_shockwave(&mut self, position: Vec3) {
        let _ = self.spawn_explosion(
            position,
            50.0,   // Large radius
            2000.0, // Strong force
            0.1,    // Duration in seconds
            FalloffType::Quadratic,
            Color::ORANGE,
            Color::ORANGE,
            100,
        );
    }

    /// Spawn a solar wind explosion (for stars) - no rings by default
    pub fn spawn_solar_wind(&mut self, position: Vec3) {
        let _ = self.spawn_explosion(
            position,
            500.0,  // Very large radius
            5000.0, // Moderate force
            3.0,    // Short duration
            FalloffType::Linear,
            Color::ORANGE,
            Color::ORANGE,
            100,
        );
    }

    /// Spawn an anti-wind explosion (for neutron stars) - no rings by default
    pub fn spawn_anti_wind(&mut self, position: Vec3) {
        let _ = self.spawn_explosion(
            position,
            500.0,   // Very large radius
            -5000.0, // Moderate force
            3.0,     // Short duration
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
                    .with_uniform_scale(explosion.current_radius * 2.0); // Sphere primitive has diameter 1.0

                primitives.push(primitive);

                // Add expanding torus shockwave rings if enabled
                if explosion.has_rings {
                    let progress = explosion.elapsed_time / explosion.duration;

                    // Primary shockwave ring (leading edge)
                    let ring1_offset = 0.1; // Slightly ahead of main sphere
                    let ring1_progress = (progress + ring1_offset).min(1.0);
                    let ring1_radius = explosion.max_radius * ring1_progress;

                    if ring1_radius > 0.5 {
                        let mut ring1_color = explosion.explosion_color;
                        // Sharper fade for the leading ring
                        let ring1_alpha = (1.0 - progress).powf(1.5) * 0.6;
                        ring1_color.a = ring1_alpha;

                        let ring1 = Primitive::new(PrimitiveType::Torus, explosion.position, ring1_color)
                            .with_uniform_scale(ring1_radius * 2.0)
                            .with_rotation(Vec3::new(std::f32::consts::FRAC_PI_2, 0.0, 0.0)); // Horizontal ring

                        primitives.push(ring1);
                    }

                    // Secondary ring (follows behind)
                    if explosion.max_radius > 20.0 { // Only for larger explosions
                        let ring2_offset = -0.15; // Slightly behind main sphere
                        let ring2_progress = (progress + ring2_offset).max(0.0);
                        let ring2_radius = explosion.max_radius * ring2_progress;

                        if ring2_radius > 0.5 {
                            let mut ring2_color = explosion.explosion_color;
                            // Different fade curve for secondary ring
                            let ring2_alpha = (1.0 - ring2_progress).powf(2.0) * 0.4;
                            ring2_color.a = ring2_alpha;

                            let ring2 = Primitive::new(PrimitiveType::Torus, explosion.position, ring2_color)
                                .with_uniform_scale(ring2_radius * 2.0)
                                .with_rotation(Vec3::new(std::f32::consts::FRAC_PI_2, 0.0, 0.0));

                            primitives.push(ring2);
                        }
                    }

                    // Tertiary vertical ring for very large explosions (crossing shockwave)
                    if explosion.max_radius > 50.0 {
                        let ring3_offset = 0.05;
                        let ring3_progress = (progress + ring3_offset).min(1.0);
                        let ring3_radius = explosion.max_radius * ring3_progress * 0.8; // Slightly smaller

                        if ring3_radius > 0.5 {
                            let mut ring3_color = explosion.explosion_color;
                            let ring3_alpha = (1.0 - progress).powf(1.8) * 0.35;
                            ring3_color.a = ring3_alpha;

                            // Vertical ring (rotated 90 degrees from horizontal)
                            let ring3 = Primitive::new(PrimitiveType::Torus, explosion.position, ring3_color)
                                .with_uniform_scale(ring3_radius * 2.0)
                                .with_rotation(Vec3::new(0.0, 0.0, 0.0)); // Vertical ring

                            primitives.push(ring3);
                        }
                    }
                }
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
