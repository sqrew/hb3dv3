use crate::engine::CollisionMask;
use crate::engine::Vec3;
use crate::engine::entity::{EntityId, EntityType};
use crate::graphics::{Color, Primitive, PrimitiveType};
use crate::scene::PhysicsManager;

// Particle trail configuration for large bodies
const LARGE_BODY_TRAIL_INTERVAL: f32 = 0.01; // Spawn particles every 20ms (50 Hz)

// Distance culling configuration
const MAX_DISTANCE_FROM_ORIGIN: f32 = 5000.0; // Auto-destroy bodies beyond this distance

/// Death sequence state for large bodies
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeathState {
    Alive,
    DeathSequence { timer: f32 }, // Death sequence in progress, timer = time remaining
    ReadyForRemoval,              // Death sequence complete, ready to be removed
}

/// Types of large gravitational bodies in the game
#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LargeBodyType {
    /// Massive gravitational body with extreme pull
    BlackHole,
    BlackHoleLarge,
    /// Massive gravitational body with extreme repulsion (negative mass)
    WhiteHole,
    /// Large rocky body with moderate gravity
    NeutronStar,
    ExoticMatter,
    Star,
    /// Habitable world with Earth-like gravity
    GasGiant,
    Planet,
    /// Artificial structure with artificial gravity
    /// Gas giant with strong gravity and large radius
    /// Exotic matter that oscillates between attractive and repulsive gravity
    LauncherMass,
    Debug,
}

impl LargeBodyType {
    /// Get default mass for this body type (in kg, scaled for gameplay)
    pub fn default_mass(self) -> f32 {
        match self {
            LargeBodyType::BlackHole => 1_000_000.0, // Extreme mass
            LargeBodyType::BlackHoleLarge => 10_000_000.0,
            LargeBodyType::WhiteHole => -900_000.0, // Slightly less negative mass for stability
            LargeBodyType::NeutronStar => 500_000.0, // Very high mass
            LargeBodyType::ExoticMatter => 250_000.0, // High mass for strong oscillating effects
            LargeBodyType::Star => 200_000.0,       // Very high mass for strong gravity
            LargeBodyType::GasGiant => 100_000.0,   // Large mass
            LargeBodyType::Planet => 50_000.0,      // Medium mass
            LargeBodyType::LauncherMass => 49_000.0,
            LargeBodyType::Debug => 100.0, // Debug body with small but reasonable mass
        }
    }

    /// Get default radius for this body type (for rendering and collision)
    pub fn default_radius(self) -> f32 {
        match self {
            LargeBodyType::BlackHole => 1.0, // Small but visible
            LargeBodyType::BlackHoleLarge => 50.0,
            LargeBodyType::WhiteHole => 1.0, // Same size as black hole, but opposite effect
            LargeBodyType::NeutronStar => 2.5, // Very small but dense
            LargeBodyType::ExoticMatter => 15.0, // Large and visible for its effects
            LargeBodyType::Star => 80.0,     // Large and bright for visibility
            LargeBodyType::GasGiant => 20.0, // Very large
            LargeBodyType::Planet => 10.0,   // Medium size
            LargeBodyType::LauncherMass => 3.0,
            LargeBodyType::Debug => 1.0, // Debug body with small but reasonable radius
        }
    }

    /// Get the color for rendering this body type
    pub fn color(self) -> Color {
        match self {
            LargeBodyType::BlackHole => Color::MAGENTA,
            LargeBodyType::BlackHoleLarge => Color::MAGENTA,
            LargeBodyType::WhiteHole => Color::WHITE,
            LargeBodyType::NeutronStar => Color::GREEN,
            LargeBodyType::ExoticMatter => Color::MAGENTA,
            LargeBodyType::Star => Color::RED,
            LargeBodyType::GasGiant => Color::YELLOW,
            LargeBodyType::Planet => Color::CYAN,
            LargeBodyType::LauncherMass => Color::WHITE,
            LargeBodyType::Debug => Color::random_color(),
        }
    }

    /// Get default collision radius ratio for this body type (multiplier of visual radius)
    pub fn default_collision_radius_ratio(self) -> f32 {
        match self {
            LargeBodyType::BlackHole => 1.0,
            LargeBodyType::BlackHoleLarge => 1.0,
            LargeBodyType::WhiteHole => 1.0,
            LargeBodyType::NeutronStar => 1.0,
            LargeBodyType::ExoticMatter => 1.0, // Large collision area for oscillating effects
            LargeBodyType::Star => 1.0,
            LargeBodyType::GasGiant => 1.0,
            LargeBodyType::Planet => 1.0,
            LargeBodyType::LauncherMass => 1.0,
            LargeBodyType::Debug => 1.0, // Standard collision radius ratio
        }
    }

    /// Get default angular velocity for this body type (radians per second)
    pub fn default_angular_velocity(self) -> f32 {
        match self {
            LargeBodyType::BlackHole => 2.0, // Fast spinning black hole for frame-dragging
            LargeBodyType::BlackHoleLarge => 1.0, // Spinning black hole for frame-dragging
            LargeBodyType::WhiteHole => -3.0, // Counter-rotating white hole
            LargeBodyType::NeutronStar => 12.0, // Extremely fast pulsar rotation
            LargeBodyType::ExoticMatter => 6.0, // Rapid oscillating rotation for visual effect
            LargeBodyType::Star => 0.5,      // Moderate stellar rotation
            LargeBodyType::GasGiant => 1.0,  // Fast rotation like Jupiter
            LargeBodyType::Planet => 0.3,    // Earth-like rotation (slower)
            LargeBodyType::LauncherMass => 3.0,
            LargeBodyType::Debug => 0.5, // Debug body with moderate rotation
        }
    }

    /// Get default ergosphere radius ratio (multiplied by visual radius)
    pub fn default_ergosphere_radius_ratio(self) -> f32 {
        match self {
            LargeBodyType::BlackHole => 100.0, // Much larger ergosphere for visible frame-dragging
            LargeBodyType::BlackHoleLarge => 2.0, // Reduced to match playable area
            LargeBodyType::NeutronStar => 20.0, // Large intense ergosphere
            LargeBodyType::WhiteHole => 20.0,  // Significant ergosphere effect
            LargeBodyType::ExoticMatter => 20.0, //
            LargeBodyType::LauncherMass => 20.0,
            _ => 0.0,
        }
    }

    /// Get default frame-dragging strength (based on mass and angular velocity)
    pub fn default_frame_dragging_strength(self) -> f32 {
        let mass = self.default_mass();
        let angular_vel = self.default_angular_velocity().abs(); // Use absolute value
        let strength_factor = match self {
            LargeBodyType::BlackHole => 0.2,      // Strong frame-dragging
            LargeBodyType::BlackHoleLarge => 0.5, // Strong frame-dragging
            LargeBodyType::NeutronStar => 0.25,   // Very strong (dense + fast spinning)
            LargeBodyType::WhiteHole => 0.15,     // Moderate frame-dragging
            LargeBodyType::ExoticMatter => 0.8,   //
            LargeBodyType::LauncherMass => 5.0,
            _ => 0.0, // No frame-dragging for other types
        };
        mass * angular_vel * strength_factor
    }

    /// Get the primitive type for rendering
    pub fn primitive_type(self) -> PrimitiveType {
        match self {
            LargeBodyType::BlackHole => PrimitiveType::Sphere,
            LargeBodyType::BlackHoleLarge => PrimitiveType::Sphere,
            LargeBodyType::WhiteHole => PrimitiveType::Sphere,
            LargeBodyType::NeutronStar => PrimitiveType::Sphere,
            LargeBodyType::Star => PrimitiveType::Sphere,
            LargeBodyType::GasGiant => PrimitiveType::Sphere,
            LargeBodyType::Planet => PrimitiveType::Sphere,
            LargeBodyType::ExoticMatter => PrimitiveType::Sphere,
            LargeBodyType::LauncherMass => PrimitiveType::Icosahedron,
            LargeBodyType::Debug => PrimitiveType::Sphere,
        }
    }
}

/// A large gravitational body ss in the game world
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

    // Lifecycle properties
    age: f32,                  // Current age in seconds
    max_lifetime: Option<f32>, // Maximum lifetime in seconds (None = eternal)
    death_state: DeathState,   // Current death sequence state

    // Particle trail system
    trail_particle_timer: f32, // Timer for spawning trail particles
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
            age: 0.0,
            max_lifetime: None, // Eternal by default
            death_state: DeathState::Alive,
            trail_particle_timer: 0.0,
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
            age: 0.0,
            max_lifetime: None, // Eternal by default
            death_state: DeathState::Alive,
            trail_particle_timer: 0.0,
        }
    }

    /// Create a new large body with a specific lifetime
    pub fn new_with_lifetime(
        entity_id: EntityId,
        body_type: LargeBodyType,
        position: Vec3,
        max_lifetime: f32,
    ) -> Self {
        let mut body = Self::new(entity_id, body_type, position);
        body.max_lifetime = Some(max_lifetime);
        body
    }

    /// Update the large body (updates rotation and visual effects only)
    pub fn update(&mut self, delta_time: f32) {
        // Position will be updated by the PhysicsManager's N-body simulation

        // Update age
        self.age += delta_time;

        // Update death sequence state machine
        match self.death_state {
            DeathState::Alive => {
                // Check if it's time to start death sequence
                if let Some(max_lifetime) = self.max_lifetime {
                    if self.age >= max_lifetime {
                        self.trigger_death_sequence();
                        // Start death sequence with 2 second duration
                        self.death_state = DeathState::DeathSequence {
                            timer: match self.body_type {
                                LargeBodyType::Debug => 1.0,
                                LargeBodyType::BlackHole => 2.0,
                                LargeBodyType::BlackHoleLarge => 2.0,
                                LargeBodyType::WhiteHole => 10.0,
                                LargeBodyType::ExoticMatter => 10.0,
                                LargeBodyType::NeutronStar => 2.0,
                                LargeBodyType::GasGiant => 5.0,
                                LargeBodyType::Planet => 5.0,
                                LargeBodyType::LauncherMass => 1.0,
                                _ => 2.0,
                            },
                        };
                    }
                }
            }
            DeathState::DeathSequence { timer } => {
                // Count down death sequence timer
                let new_timer = timer - delta_time;
                if new_timer <= 0.0 {
                    self.death_state = DeathState::ReadyForRemoval;
                } else {
                    self.death_state = DeathState::DeathSequence { timer: new_timer };
                }
            }
            DeathState::ReadyForRemoval => {
                // Body will be removed by manager in this state
            }
        }

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
        .with_uniform_scale(self.radius * 2.0) // Sphere primitive has diameter 1.0, so scale by diameter
        .with_rotation(Vec3::new(0.0, self.rotation, 0.0)) // Rotate around Y-axis
    }

    // Getters
    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }
    pub fn position(&self) -> Vec3 {
        self.position
    }
    pub fn angular_velocity(&self) -> f32 {
        self.angular_velocity
    }

    // Setters
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }
    pub fn set_velocity(&mut self, velocity: Vec3) {
        self.velocity = velocity;
    }

    // Collision methods
    pub fn collision_radius(&self) -> f32 {
        self.collision_radius
    }

    pub fn collision_mask(&self) -> CollisionMask {
        self.collision_mask
    }

    // Lifecycle methods
    pub fn age(&self) -> f32 {
        self.age
    }

    pub fn max_lifetime(&self) -> Option<f32> {
        self.max_lifetime
    }

    pub fn remaining_lifetime(&self) -> Option<f32> {
        self.max_lifetime.map(|max| (max - self.age).max(0.0))
    }

    pub fn is_dead(&self) -> bool {
        matches!(self.death_state, DeathState::ReadyForRemoval)
    }

    pub fn lifetime_progress(&self) -> Option<f32> {
        self.max_lifetime.map(|max| (self.age / max).min(1.0))
    }

    /// Trigger death sequence for this large body type
    fn trigger_death_sequence(&mut self) {
        match self.body_type {
            LargeBodyType::BlackHole => {
                // Hawking radiation evaporation - dramatic size increase then collapse
                println!("💥 BlackHole evaporating in burst of Hawking radiation!");
                self.radius *= 2.5; // Dramatic expansion before death

                // Could spawn particles, change color, etc.
            }

            LargeBodyType::BlackHoleLarge => {
                // Massive black hole evaporation - even more dramatic
                println!("🕳️💥 Supermassive BlackHole undergoing final evaporation!");
                self.radius *= 4.0; // Even larger expansion for supermassive
                // Could spawn intense gravitational waves, spacetime distortion effects
            }

            LargeBodyType::WhiteHole => {
                // Matter ejection finale - explosive outward burst
                println!("🌟 WhiteHole ejecting all accumulated matter!");
                self.mass *= 0.1; // Lose most mass in final ejection
                // Could spawn outward-moving matter particles
            }

            LargeBodyType::Star => {
                // Supernova explosion - classic stellar death
                println!("⭐ Star going supernova!");
                self.radius *= 5.0; // Massive expansion
                // Could spawn shockwave, change to red giant color
            }

            LargeBodyType::NeutronStar => {
                // Collapse to black hole - density limit exceeded
                println!("🕳️  NeutronStar collapsing into black hole!");
                self.radius *= 0.5; // Collapse inward
                self.mass *= 2.0; // Gravitational intensification
            }

            LargeBodyType::Planet => {
                // Atmospheric loss and core fragmentation
                println!("🌍 Planet losing atmosphere and breaking apart!");
                self.radius *= 1.5; // Expansion as core is exposed
                // Could spawn debris particles
            }

            LargeBodyType::GasGiant => {
                // Gas dispersion - gradual deflation
                println!("🪐 Gas Giant dispersing atmospheric layers!");
                self.radius *= 3.0; // Atmospheric expansion
                self.mass *= 0.3; // Lose gas mass
            }

            LargeBodyType::LauncherMass => {
                // Fragmentation into smaller pieces
                println!("☄️  Asteroid fragmenting into debris field!");
                self.radius *= 1.2; // Slight expansion as it breaks apart
                // Could spawn multiple smaller asteroid particles
            }

            LargeBodyType::ExoticMatter => {
                // Matter/antimatter annihilation - most dramatic
                println!("⚡ Exotic Matter undergoing catastrophic annihilation!");
                self.radius *= 8.0; // Massive energy release expansion
                // Could spawn high-energy particles, light effects
            }

            LargeBodyType::Debug => {
                // Simple debug death
                println!("🔧 Debug body completing lifecycle test");
                // No special effects, just clean removal
            }
        }
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

        self.bodies.push(body);

        entity_id
    }

    /// Spawn a large body with a specific lifetime
    pub fn spawn_body_with_lifetime(
        &mut self,
        body_type: LargeBodyType,
        position: Vec3,
        max_lifetime: f32,
        physics: &mut PhysicsManager,
        entity_manager: &mut crate::engine::entity::EntityManager,
    ) -> EntityId {
        let entity_id = entity_manager.create_entity(crate::engine::entity::EntityType::LargeBody);

        let mut body = LargeBody::new_with_lifetime(entity_id, body_type, position, max_lifetime);
        body.register_with_physics(physics);

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

    /// Spawn a large body with custom velocity and lifetime (for weapon launcher)
    pub fn spawn_body_with_velocity_and_lifetime(
        &mut self,
        body_type: LargeBodyType,
        position: Vec3,
        velocity: Vec3,
        max_lifetime: f32,
        physics: &mut PhysicsManager,
        entity_manager: &mut crate::engine::entity::EntityManager,
    ) -> EntityId {
        let entity_id = entity_manager.create_entity(crate::engine::entity::EntityType::LargeBody);

        let mut body = LargeBody::new_with_lifetime(entity_id, body_type, position, max_lifetime);
        body.set_velocity(velocity);
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
    pub fn update(
        &mut self,
        delta_time: f32,
        physics_manager: &mut PhysicsManager,
        entity_manager: &mut crate::engine::entity::EntityManager,
    ) {
        // First update all living bodies
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
                    self.pending_events.push(crate::engine::EventType::Graphics(
                        crate::engine::GraphicsEvent::SpawnParticles {
                            position: body.position,
                            velocity: Vec3::zeros(),
                            count: 1000,
                            lifetime: 15.0,
                            color: Color::RED,
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
                                damage: 0.0, // Solar wind doesn't deal damage, just physics force
                                damage_radius: 0.0,
                                explosion_color: Color::MAGENTA,
                                particle_color: Color::MAGENTA,
                                particle_count: 0,
                            },
                        ));
                }
            }

            // Particle trail system - spawn trail particles for all bodies
            body.trail_particle_timer -= delta_time;
            if body.trail_particle_timer <= 0.0 {
                // Reset timer
                body.trail_particle_timer = LARGE_BODY_TRAIL_INTERVAL;

                // Calculate spawn position behind the body's movement
                // This prevents particles from spawning inside the body and being launched
                let spawn_offset = if body.velocity.magnitude() > 0.01 {
                    // Body is moving - spawn behind it
                    let velocity_dir = body.velocity.normalize();
                    -velocity_dir * body.radius // Behind the body at radius distance
                } else {
                    // Body is stationary - spawn at radius distance in a consistent direction
                    Vec3::new(0.0, body.radius, 0.0) // Spawn above
                };

                let spawn_position = body.position + spawn_offset;

                // Queue particle spawn event with body-type-specific color
                self.pending_events
                    .push(crate::engine::dispatcher::EventType::Graphics(
                        crate::engine::dispatcher::GraphicsEvent::SpawnParticles {
                            position: spawn_position,
                            velocity: Vec3::zeros(), // Stationary - let gravity move them!
                            count: 1,                // Fewer than player trail
                            lifetime: 15.0,          // Longer lifetime to see gravity effects
                            color: body.body_type.color(), // Match body color
                        },
                    ));
            }

            body.update(delta_time);
            // Update position/velocity from physics simulation
            body.update_from_physics(physics_manager);

            // Distance culling - mark bodies that drift too far for removal
            let distance_from_origin = body.position.magnitude();
            if distance_from_origin > MAX_DISTANCE_FROM_ORIGIN {
                // Mark for immediate removal without death sequence
                body.death_state = DeathState::ReadyForRemoval;
            }
        }

        // Remove dead bodies (iterate in reverse to avoid index issues)
        let mut i = self.bodies.len();
        while i > 0 {
            i -= 1;
            if self.bodies[i].is_dead() {
                let body = self.bodies.remove(i);

                // Remove from physics system and fix indices
                if let Some(physics_index) = body.physics_index {
                    physics_manager.remove_gravitational_body(physics_index);

                    // Update physics indices for all remaining bodies that had higher indices
                    for remaining_body in &mut self.bodies {
                        if let Some(ref mut remaining_index) = remaining_body.physics_index {
                            if *remaining_index > physics_index {
                                *remaining_index -= 1;
                            }
                        }
                    }
                }

                // Remove from entity system
                entity_manager.destroy_entity(body.entity_id);

                println!(
                    "🪦 Large body {:?} died of old age at {:.1}s",
                    body.body_type, body.age
                );
            }
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

    /// Get body by entity ID
    pub fn get_body(&self, entity_id: EntityId) -> Option<&LargeBody> {
        self.bodies.iter().find(|b| b.entity_id() == entity_id)
    }
}
