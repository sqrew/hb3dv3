use crate::engine::CollisionMask;
use crate::engine::Vec3;
use crate::engine::entity::{EntityId, EntityType};
use crate::graphics::{Color, Primitive, PrimitiveType};
use crate::scene::PhysicsManager;

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
}

impl LargeBodyType {
    /// Get default mass for this body type (in kg, scaled for gameplay)
    pub fn default_mass(self) -> f32 {
        match self {
            LargeBodyType::BlackHole => 1_000_000.0,  // Extreme mass
            LargeBodyType::WhiteHole => -1_000_000.0, // Extreme negative mass (repulsion)
            LargeBodyType::NeutronStar => 500_000.0,  // Very high mass
            LargeBodyType::Star => 200_000.0,         // Very high mass for strong gravity
            LargeBodyType::GasGiant => 100_000.0,     // Large mass
            LargeBodyType::Planet => 50_000.0,        // Medium mass
        }
    }

    /// Get default radius for this body type (for rendering and collision)
    pub fn default_radius(self) -> f32 {
        match self {
            LargeBodyType::BlackHole => 2.0,   // Small but visible
            LargeBodyType::WhiteHole => 2.0,   // Same size as black hole, but opposite effect
            LargeBodyType::NeutronStar => 1.5, // Very small but dense
            LargeBodyType::Star => 5.0,        // Large and bright for visibility
            LargeBodyType::GasGiant => 12.0,   // Very large
            LargeBodyType::Planet => 5.0,      // Medium size
        }
    }

    /// Get the color for rendering this body type
    pub fn color(self) -> Color {
        match self {
            LargeBodyType::BlackHole => Color::new(0.1, 0.0, 0.1, 1.0), // Dark purple
            LargeBodyType::WhiteHole => Color::new(1.0, 1.0, 1.0, 1.0), // Bright white (opposite of black hole)
            LargeBodyType::NeutronStar => Color::new(0.8, 0.9, 1.0, 1.0), // Bright white-blue
            LargeBodyType::Star => Color::new(1.0, 0.9, 0.3, 1.0),      // Bright yellow
            LargeBodyType::GasGiant => Color::new(0.8, 0.6, 0.3, 1.0),  // Brown-orange
            LargeBodyType::Planet => Color::new(0.3, 0.6, 0.8, 1.0),    // Blue-green
        }
    }

    /// Get default collision radius ratio for this body type (multiplier of visual radius)
    pub fn default_collision_radius_ratio(self) -> f32 {
        0.8
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

    // Physics integration
    physics_index: Option<usize>, // Index in PhysicsManager's gravitational_bodies array
}

impl LargeBody {
    /// Create a new large body with default properties for its type
    pub fn new(entity_id: EntityId, body_type: LargeBodyType, position: Vec3) -> Self {
        let radius = body_type.default_radius();
        Self {
            entity_id,
            body_type,
            position,
            velocity: Vec3::zeros(),
            mass: body_type.default_mass(),
            radius,
            collision_radius: radius * body_type.default_collision_radius_ratio(),
            collision_mask: CollisionMask::from(EntityType::LargeBody),
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
    ) -> Self {
        Self {
            entity_id,
            body_type,
            position,
            velocity,
            mass,
            radius,
            collision_radius,
            collision_mask: CollisionMask::from(EntityType::LargeBody),
            physics_index: None,
        }
    }

    /// Update the large body (currently just updates position from velocity)
    pub fn update(&mut self, _delta_time: f32) {
        // Position will be updated by the PhysicsManager's N-body simulation
        // This is mainly for any game-specific logic we might add later

        // For now, we could add some basic bounds checking or special effects
        match self.body_type {
            LargeBodyType::BlackHole => {
                // Black holes could have special visual effects here
            }
            LargeBodyType::WhiteHole => {
                // White holes could have special repulsion visual effects here
            }
            LargeBodyType::Star => {
                // Stars could pulsate or have particle effects
            }
            _ => {
                // Most bodies just follow physics
            }
        }
    }

    /// Register this body with the physics system
    pub fn register_with_physics(&mut self, physics: &mut PhysicsManager) -> usize {
        let index =
            physics.add_gravitational_body(self.position, self.mass, self.velocity, self.radius);
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
}

impl LargeBodyManager {
    pub fn new() -> Self {
        Self { bodies: Vec::new() }
    }

    /// Spawn a new large body
    pub fn spawn_body(
        &mut self,
        body_type: LargeBodyType,
        position: Vec3,
        velocity: Vec3,
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
        );
        body.register_with_physics(physics);

        self.bodies.push(body);

        entity_id
    }

    /// Update all large bodies
    pub fn update(&mut self, delta_time: f32, physics: &PhysicsManager) {
        for body in &mut self.bodies {
            body.update(delta_time);
            // Update position/velocity from physics simulation
            body.update_from_physics(physics);
        }
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
