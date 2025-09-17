use crate::engine::CollisionMask;
use crate::engine::Vec3;
use crate::engine::entity::{EntityId, EntityType};
use crate::graphics::{Color, Primitive, PrimitiveType};
use crate::scene::PhysicsManager;
use rand;

/// Types of large gravitational bodies in the game
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LargeBodyType {
    /// Massive gravitational body with extreme pull
    BlackHole,
    /// Massive gravitational body with extreme repulsion (negative mass)
    WhiteHole,
    /// Large rocky body with moderate gravity
    Star,
    /// Habitable world with Earth-like gravity
    Planet,
    /// Artificial structure with artificial gravity
    NeutronStar,
    /// Gas giant with strong gravity and large radius
    GasGiant,
    /// Exotic matter that oscillates between attractive and repulsive gravity
    ExoticMatter,
}

impl LargeBodyType {
    /// Get default mass for this body type (in kg, scaled for gameplay)
    pub fn default_mass(self) -> f32 {
        match self {
            LargeBodyType::BlackHole => 1_000_000.0,  // Extreme mass
            LargeBodyType::WhiteHole => -900_000.0,   // Slightly less negative mass for stability
            LargeBodyType::NeutronStar => 500_000.0,  // Very high mass
            LargeBodyType::Star => 200_000.0,         // Very high mass for strong gravity
            LargeBodyType::GasGiant => 100_000.0,     // Large mass
            LargeBodyType::Planet => 50_000.0,        // Medium mass
            LargeBodyType::ExoticMatter => 250_000.0, // High mass for strong oscillating effects
        }
    }

    /// Get default radius for this body type (for rendering and collision)
    pub fn default_radius(self) -> f32 {
        match self {
            LargeBodyType::BlackHole => 2.0,     // Small but visible
            LargeBodyType::WhiteHole => 2.0,     // Same size as black hole, but opposite effect
            LargeBodyType::NeutronStar => 2.5,   // Very small but dense
            LargeBodyType::Star => 80.0,         // Large and bright for visibility
            LargeBodyType::GasGiant => 20.0,     // Very large
            LargeBodyType::Planet => 10.0,       // Medium size
            LargeBodyType::ExoticMatter => 15.0, // Large and visible for its effects
        }
    }

    /// Get the color for rendering this body type
    pub fn color(self) -> Color {
        match self {
            LargeBodyType::BlackHole => Color::MAGENTA,
            LargeBodyType::WhiteHole => Color::WHITE,
            LargeBodyType::NeutronStar => Color::GREEN,
            LargeBodyType::Star => Color::RED,
            LargeBodyType::GasGiant => Color::YELLOW,
            LargeBodyType::Planet => Color::CYAN,
            LargeBodyType::ExoticMatter => Color::MAGENTA,
        }
    }

    /// Get default collision radius ratio for this body type (multiplier of visual radius)
    pub fn default_collision_radius_ratio(self) -> f32 {
        match self {
            // Smaller collision radii for better orbital mechanics
            LargeBodyType::Planet => 0.6,
            LargeBodyType::Star => 0.6,
            LargeBodyType::GasGiant => 0.6,
            // Keep larger collision radii for extreme objects
            LargeBodyType::BlackHole => 0.6,
            LargeBodyType::WhiteHole => 0.6,
            LargeBodyType::NeutronStar => 0.6,
            LargeBodyType::ExoticMatter => 0.6, // Large collision area for oscillating effects
        }
    }

    /// Get default angular velocity for this body type (radians per second)
    pub fn default_angular_velocity(self) -> f32 {
        match self {
            LargeBodyType::BlackHole => 3.0, // Fast spinning black hole for frame-dragging
            LargeBodyType::WhiteHole => -3.0, // Counter-rotating white hole
            LargeBodyType::NeutronStar => 12.0, // Extremely fast pulsar rotation
            LargeBodyType::Star => 0.5,      // Moderate stellar rotation
            LargeBodyType::GasGiant => 1.0,  // Fast rotation like Jupiter
            LargeBodyType::Planet => 0.3,    // Earth-like rotation (slower)
            LargeBodyType::ExoticMatter => 6.0, // Rapid oscillating rotation for visual effect
        }
    }

    /// Get default ergosphere radius ratio (multiplied by visual radius)
    pub fn default_ergosphere_radius_ratio(self) -> f32 {
        match self {
            LargeBodyType::BlackHole => 20.0, // Much larger ergosphere for visible frame-dragging
            LargeBodyType::NeutronStar => 20.0, // Large intense ergosphere
            LargeBodyType::WhiteHole => 20.0, // Significant ergosphere effect
            LargeBodyType::Star => 0.0,       // No ergosphere effect
            LargeBodyType::GasGiant => 0.0,   // No ergosphere effect
            LargeBodyType::Planet => 0.0,     // No ergosphere effect
            LargeBodyType::ExoticMatter => 20.0, // No ergosphere (oscillating gravity is the main effect)
        }
    }

    /// Get default frame-dragging strength (based on mass and angular velocity)
    pub fn default_frame_dragging_strength(self) -> f32 {
        let mass = self.default_mass();
        let angular_vel = self.default_angular_velocity().abs(); // Use absolute value
        let strength_factor = match self {
            LargeBodyType::BlackHole => 0.2,    // Strong frame-dragging
            LargeBodyType::NeutronStar => 0.15, // Very strong (dense + fast spinning)
            LargeBodyType::WhiteHole => 0.05,   // Moderate frame-dragging
            LargeBodyType::ExoticMatter => 0.5, // No frame-dragging (oscillation is main effect)
            _ => 0.0,                           // No frame-dragging for other types
        };
        mass * angular_vel * strength_factor
    }

    /// Get the primitive type for rendering
    pub fn primitive_type(self) -> PrimitiveType {
        match self {
            LargeBodyType::BlackHole => PrimitiveType::Sphere, // Dark sphere
            LargeBodyType::WhiteHole => PrimitiveType::Sphere, // Bright sphere
            LargeBodyType::NeutronStar => PrimitiveType::Sphere, // Bright sphere
            LargeBodyType::Star => PrimitiveType::Sphere,      // Glowing sphere
            LargeBodyType::GasGiant => PrimitiveType::Sphere,  // Large sphere
            LargeBodyType::Planet => PrimitiveType::Sphere,    // Earth-like sphere
            LargeBodyType::ExoticMatter => PrimitiveType::Sphere, // Oscillating sphere
        }
    }
}

/// A large gravitational body in the game world
#[derive(Debug, Clone)]
pub struct LargeBody {
    entity_id: EntityId,
    body_type: LargeBodyType,
    position: Vec3,
    velocity: Vec3,
    mass: f32,
    radius: f32,           // Visual radius for rendering
    collision_radius: f32, // Collision radius for gameplay mechanics
    collision_mask: CollisionMask,
    angular_velocity: f32,  // Radians per second (positive = counterclockwise)
    rotation: f32,          // Current rotation angle in radians VISUALS
    ergosphere_radius: f32, // Radius of frame-dragging effect (0.0 = no ergosphere)
    frame_dragging_strength: f32, // Strength of frame-dragging effect

    // Solar wind effects (for stars)
    solar_wind_timer: f32,    // Time until next solar wind emission
    solar_wind_interval: f32, // How often to emit solar winds (seconds)

    // Physics integration
    physics_index: Option<usize>, // Index in PhysicsManager's gravitational_bodies array
}

impl LargeBody {
    /// Create a new large body with default properties for its type
    pub fn new(entity_id: EntityId, body_type: LargeBodyType, position: Vec3) -> Self {
        let radius = body_type.default_radius();

        let solar_wind_interval = match body_type {
            LargeBodyType::Star => 6.0,
            LargeBodyType::WhiteHole => 2.0,
            LargeBodyType::NeutronStar => 8.0,
            LargeBodyType::ExoticMatter => 0.2,
            _ => 0.0,
        };

        let base_mass = body_type.default_mass();

        Self {
            entity_id,
            body_type,
            position,
            velocity: Vec3::zeros(),
            mass: base_mass,
            radius,
            collision_radius: radius * body_type.default_collision_radius_ratio(),
            collision_mask: CollisionMask::from(EntityType::LargeBody),
            angular_velocity: body_type.default_angular_velocity(),
            rotation: 0.0, // Start with no rotation
            ergosphere_radius: radius * body_type.default_ergosphere_radius_ratio(),
            frame_dragging_strength: body_type.default_frame_dragging_strength(),
            solar_wind_timer: solar_wind_interval, // Start with first emission ready
            solar_wind_interval,
            physics_index: None,
        }
    }

    /// Create a new large body with custom properties
    pub fn new_custom(
        entity_id: EntityId,
        body_type: LargeBodyType,
        position: Vec3,
        velocity: Vec3,
        mass: f32,
        radius: f32,
        collision_radius: f32,
        angular_velocity: f32,
        ergosphere_radius: f32,
        frame_dragging_strength: f32,
    ) -> Self {
        let solar_wind_interval = match body_type {
            LargeBodyType::Star => 6.0,
            LargeBodyType::WhiteHole => 2.0,
            LargeBodyType::NeutronStar => 8.0,
            LargeBodyType::ExoticMatter => 0.2,
            _ => 0.0,
        };
        Self {
            entity_id,
            body_type,
            position,
            velocity,
            mass,
            radius,
            collision_radius,
            collision_mask: CollisionMask::from(EntityType::LargeBody),
            angular_velocity,
            rotation: 0.0, // Start with no rotation
            ergosphere_radius,
            frame_dragging_strength,
            solar_wind_timer: solar_wind_interval,
            solar_wind_interval,
            physics_index: None,
        }
    }

    /// Update the large body (updates rotation and visual effects only)
    pub fn update(&mut self, delta_time: f32) {
        // Position will be updated by the PhysicsManager's N-body simulation
        // But we handle rotation here since it's visual-only

        // Update rotation based on angular velocity
        self.rotation += self.angular_velocity * delta_time;

        // Keep rotation in [0, 2π] range for consistency
        self.rotation = self.rotation % (2.0 * std::f32::consts::PI);

        // Add other body-specific behaviors/effects (visual only)
        match self.body_type {
            LargeBodyType::BlackHole => {
                // Black holes could have special visual effects here
            }
            LargeBodyType::WhiteHole => {
                // White holes could have special repulsion visual effects here
            }
            LargeBodyType::Star => {
                // Stars could have pulsing visual effects here
            }
            LargeBodyType::ExoticMatter => {}
            _ => {
                // Most bodies just follow physics
            }
        }
    }

    /// Register this body with the physics system
    pub fn register_with_physics(&mut self, physics: &mut PhysicsManager) -> usize {
        let index = physics.add_gravitational_body(
            self.position,
            self.mass,
            self.velocity,
            self.collision_radius,
            self.angular_velocity,
            self.ergosphere_radius,
            self.frame_dragging_strength,
        );
        self.physics_index = Some(index);
        index
    }

    /// Update this body's physics representation
    pub fn sync_with_physics(&mut self, physics: &mut PhysicsManager) {
        if let Some(index) = self.physics_index {
            physics.update_gravitational_body(index, self.position, self.velocity);
        }
    }

    /// Update this body's state from the physics system (after N-body simulation)
    pub fn update_from_physics(&mut self, physics: &PhysicsManager) {
        if let Some(index) = self.physics_index {
            if let Some(body) = physics.gravitational_bodies().get(index) {
                self.position = Vec3::new(body.position[0], body.position[1], body.position[2]);
                self.velocity = Vec3::new(body.velocity[0], body.velocity[1], body.velocity[2]);
                self.angular_velocity = body.angular_velocity; // Update spin from physics!
            }
        }
    }

    /// Get render data for this large body
    pub fn get_render_data(&self) -> Primitive {
        Primitive::new(
            self.body_type.primitive_type(),
            self.position,
            self.body_type.color(),
        )
        .with_uniform_scale(self.radius)
        .with_rotation(Vec3::new(0.0, self.rotation, 0.0)) // Rotate around Y-axis
    }

    // Getters
    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }
    pub fn body_type(&self) -> LargeBodyType {
        self.body_type
    }
    pub fn position(&self) -> Vec3 {
        self.position
    }
    pub fn velocity(&self) -> Vec3 {
        self.velocity
    }
    pub fn mass(&self) -> f32 {
        self.mass
    }
    pub fn radius(&self) -> f32 {
        self.radius
    }
    pub fn physics_index(&self) -> Option<usize> {
        self.physics_index
    }
    pub fn angular_velocity(&self) -> f32 {
        self.angular_velocity
    }
    pub fn rotation(&self) -> f32 {
        self.rotation
    }
    pub fn ergosphere_radius(&self) -> f32 {
        self.ergosphere_radius
    }
    pub fn frame_dragging_strength(&self) -> f32 {
        self.frame_dragging_strength
    }

    // Setters
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }
    pub fn set_velocity(&mut self, velocity: Vec3) {
        self.velocity = velocity;
    }
    pub fn set_mass(&mut self, mass: f32) {
        self.mass = mass;
    }
    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
    }

    pub fn set_collision_radius(&mut self, collision_radius: f32) {
        self.collision_radius = collision_radius;
    }

    pub fn set_angular_velocity(&mut self, angular_velocity: f32) {
        self.angular_velocity = angular_velocity;
    }

    // Collision methods
    pub fn collision_radius(&self) -> f32 {
        self.collision_radius
    }

    pub fn collision_mask(&self) -> CollisionMask {
        self.collision_mask
    }
}

/// Manager for all large bodies in the game
pub struct LargeBodyManager {
    bodies: Vec<LargeBody>,
    pending_events: Vec<crate::engine::dispatcher::EventType>,
}

impl LargeBodyManager {
    pub fn new() -> Self {
        Self {
            bodies: Vec::new(),
            pending_events: Vec::new(),
        }
    }

    /// Spawn a new large body
    pub fn spawn_body(
        &mut self,
        body_type: LargeBodyType,
        position: Vec3,
        physics: &mut PhysicsManager,
        entity_manager: &mut crate::engine::entity::EntityManager,
    ) -> EntityId {
        let entity_id = entity_manager.create_entity(crate::engine::entity::EntityType::LargeBody);

        let mut body = LargeBody::new(entity_id, body_type, position);
        body.register_with_physics(physics);

        let physics_index = body.physics_index();
        self.bodies.push(body);

        entity_id
    }

    /// Spawn a large body with custom properties
    pub fn spawn_body_custom(
        &mut self,
        body_type: LargeBodyType,
        position: Vec3,
        velocity: Vec3,
        mass: f32,
        radius: f32,
        collision_radius: f32,
        physics: &mut PhysicsManager,
        entity_manager: &mut crate::engine::entity::EntityManager,
    ) -> EntityId {
        let entity_id = entity_manager.create_entity(crate::engine::entity::EntityType::LargeBody);

        let mut body = LargeBody::new_custom(
            entity_id,
            body_type,
            position,
            velocity,
            mass,
            radius,
            collision_radius,
            body_type.default_angular_velocity(),
            radius * body_type.default_ergosphere_radius_ratio(),
            body_type.default_frame_dragging_strength(),
        );
        body.register_with_physics(physics);

        self.bodies.push(body);

        entity_id
    }

    /// Spawn a binary pair of large bodies in orbit around each other
    pub fn spawn_binary_pair(
        &mut self,
        body_type1: LargeBodyType,
        body_type2: LargeBodyType,
        center_position: Vec3,
        separation_distance: f32,
        physics: &mut PhysicsManager,
        entity_manager: &mut crate::engine::entity::EntityManager,
    ) -> (EntityId, EntityId) {
        let mass1 = body_type1.default_mass();
        let mass2 = body_type2.default_mass();
        let total_mass = mass1 + mass2;

        // Handle special case: nearly equal and opposite masses (e.g., BlackHole + WhiteHole)
        let (pos1, pos2, orbital_speed) = if total_mass.abs() < 300_000.0 {
            // Place bodies equidistant from center
            let separation_vector = Vec3::new(separation_distance * 0.5, 0.0, 0.0);
            let pos1 = center_position - separation_vector;
            let pos2 = center_position + separation_vector;

            // For equal opposite masses, use reduced orbital speed based on individual masses
            let effective_mass = mass1.abs(); // Use absolute value of one mass
            let gravitational_constant = 6.674e-1; // Same as in shader
            let orbital_speed =
                (gravitational_constant * effective_mass / separation_distance).sqrt() * 0.5;

            (pos1, pos2, orbital_speed)
        } else {
            // Normal case: calculate center of mass positions
            let mass_ratio1 = mass2 / total_mass; // Distance ratio for body1
            let mass_ratio2 = mass1 / total_mass; // Distance ratio for body2

            // Position bodies around center of mass
            let separation_vector = Vec3::new(separation_distance, 0.0, 0.0);
            let pos1 = center_position - separation_vector * mass_ratio1;
            let pos2 = center_position + separation_vector * mass_ratio2;

            // Calculate circular orbital velocity: v = sqrt(G * total_mass / separation)
            let gravitational_constant = 6.674e-1; // Same as in shader
            let orbital_speed =
                (gravitational_constant * total_mass.abs() / separation_distance).sqrt();

            (pos1, pos2, orbital_speed)
        };

        // Give tangential velocities (perpendicular to separation)
        let orbital_direction = Vec3::new(0.0, 1.0, 0.0); // Orbit in XZ plane
        let (vel1, vel2) = if total_mass.abs() < 1.0 {
            // Equal and opposite masses: both get same speed in opposite directions
            let vel1 = orbital_direction * orbital_speed;
            let vel2 = -orbital_direction * orbital_speed;
            (vel1, vel2)
        } else {
            // Normal case: velocities proportional to mass ratios
            let mass_ratio1 = mass2 / total_mass;
            let mass_ratio2 = mass1 / total_mass;
            let vel1 = orbital_direction * orbital_speed * mass_ratio2;
            let vel2 = -orbital_direction * orbital_speed * mass_ratio1;
            (vel1, vel2)
        };

        println!("🌌 Creating binary system:");
        println!(
            "  Body 1: {:?} at {:?} with velocity {:?}",
            body_type1, pos1, vel1
        );
        println!(
            "  Body 2: {:?} at {:?} with velocity {:?}",
            body_type2, pos2, vel2
        );
        println!(
            "  Orbital speed: {:.2}, separation: {:.2}",
            orbital_speed, separation_distance
        );

        let entity1 = self.spawn_body_custom(
            body_type1,
            pos1,
            vel1,
            mass1,
            body_type1.default_radius(),
            body_type1.default_radius() * body_type1.default_collision_radius_ratio(),
            physics,
            entity_manager,
        );

        let entity2 = self.spawn_body_custom(
            body_type2,
            pos2,
            vel2,
            mass2,
            body_type2.default_radius(),
            body_type2.default_radius() * body_type2.default_collision_radius_ratio(),
            physics,
            entity_manager,
        );

        (entity1, entity2)
    }

    /// Update all large bodies
    pub fn update(&mut self, delta_time: f32, physics: &PhysicsManager) {
        for body in &mut self.bodies {
            // Update the body and check for solar wind events
            if (body.body_type == LargeBodyType::Star && body.solar_wind_interval > 0.0)
                || (body.body_type == LargeBodyType::WhiteHole && body.solar_wind_interval > 0.0)
            {
                body.solar_wind_timer -= delta_time;
                if body.solar_wind_timer <= 0.0 {
                    // Reset timer for next emission with random interval
                    body.solar_wind_timer = body.solar_wind_interval;

                    // Queue solar wind event
                    self.pending_events
                        .push(crate::engine::dispatcher::EventType::Explosion(
                            crate::engine::dispatcher::ExplosionEvent::SolarWind {
                                position: body.position,
                            },
                        ));
                }
            } else if body.body_type == LargeBodyType::NeutronStar && body.solar_wind_interval > 0.0
            {
                body.solar_wind_timer -= delta_time;
                if body.solar_wind_timer <= 0.0 {
                    body.solar_wind_timer = body.solar_wind_interval;
                    self.pending_events
                        .push(crate::engine::dispatcher::EventType::Explosion(
                            crate::engine::dispatcher::ExplosionEvent::AntiWind {
                                position: body.position,
                            },
                        ));
                }
            } else if body.body_type == LargeBodyType::ExoticMatter
                && body.solar_wind_interval > 0.0
            {
                body.solar_wind_timer -= delta_time;
                if body.solar_wind_timer <= 0.0 {
                    body.solar_wind_timer = body.solar_wind_interval;
                    self.pending_events
                        .push(crate::engine::dispatcher::EventType::Explosion(
                            crate::engine::dispatcher::ExplosionEvent::Custom {
                                position: body.position,
                                max_radius: 80.0,
                                force_strength: 10000.0,
                                duration: 2.0,
                                falloff_type: super::FalloffType::Quadratic,
                            },
                        ));
                }
            }

            body.update(delta_time);
            // Update position/velocity from physics simulation
            body.update_from_physics(physics);
        }
    }

    /// Drain pending events (called by dispatcher)
    pub fn drain_events(&mut self) -> Vec<crate::engine::dispatcher::EventType> {
        std::mem::take(&mut self.pending_events)
    }

    /// Get render data for all bodies
    pub fn get_render_data(&self) -> Vec<Primitive> {
        self.bodies
            .iter()
            .map(|body| body.get_render_data())
            .collect()
    }

    /// Get reference to all bodies
    pub fn bodies(&self) -> &[LargeBody] {
        &self.bodies
    }

    /// Get mutable reference to all bodies
    pub fn bodies_mut(&mut self) -> &mut [LargeBody] {
        &mut self.bodies
    }

    /// Remove a body by entity ID
    pub fn remove_body(&mut self, entity_id: EntityId) -> bool {
        if let Some(pos) = self.bodies.iter().position(|b| b.entity_id() == entity_id) {
            let removed = self.bodies.remove(pos);
            println!(
                "🗑️ Removed {} (entity: {})",
                format!("{:?}", removed.body_type()).to_lowercase(),
                entity_id.0
            );
            true
        } else {
            false
        }
    }

    /// Get body by entity ID
    pub fn get_body(&self, entity_id: EntityId) -> Option<&LargeBody> {
        self.bodies.iter().find(|b| b.entity_id() == entity_id)
    }

    /// Get mutable body by entity ID
    pub fn get_body_mut(&mut self, entity_id: EntityId) -> Option<&mut LargeBody> {
        self.bodies.iter_mut().find(|b| b.entity_id() == entity_id)
    }

    /// Clear all bodies
    pub fn clear(&mut self) {
        self.bodies.clear();
    }

    /// Get count of bodies by type
    pub fn count_by_type(&self, body_type: LargeBodyType) -> usize {
        self.bodies
            .iter()
            .filter(|b| b.body_type() == body_type)
            .count()
    }
}
