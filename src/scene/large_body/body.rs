use crate::engine::CollisionMask;
use crate::engine::Vec3;
use crate::engine::entity::{EntityId, EntityType};
use crate::graphics::{Color, Primitive, PrimitiveType};
use crate::scene::PhysicsManager;

use super::body_type::{DeathState, LargeBodyType};

// Particle trail configuration for large bodies
const LARGE_BODY_TRAIL_INTERVAL: f32 = 0.01; // Spawn particles every 20ms (50 Hz)

/// Request to spawn a new large body (queued during death sequences)
#[derive(Debug, Clone)]
pub struct BodySpawnRequest {
    pub body_type: LargeBodyType,
    pub position: Vec3,
    pub velocity: Vec3,
    pub lifetime: Option<f32>, // None = eternal
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

    // Lifecycle properties
    age: f32,                  // Current age in seconds
    max_lifetime: Option<f32>, // Maximum lifetime in seconds (None = eternal)
    death_state: DeathState,   // Current death sequence state

    // Particle trail system
    trail_particle_timer: f32, // Timer for spawning trail particles

    // Event queue for death sequence effects
    pending_events: Vec<crate::engine::dispatcher::EventType>,

    // Body spawn queue for death sequences (e.g., star -> neutron star)
    pending_body_spawns: Vec<BodySpawnRequest>,

    // Absorption tracking (for BlackHoleLarge)
    absorption_count: u32, // How many bodies have been absorbed

    // Death animation tracking
    death_animation_start_radius: Option<f32>, // Radius when death sequence began
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
            pending_events: Vec::new(),
            pending_body_spawns: Vec::new(),
            absorption_count: 0,
            death_animation_start_radius: None,
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
            pending_events: Vec::new(),
            pending_body_spawns: Vec::new(),
            absorption_count: 0,
            death_animation_start_radius: None,
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
                // Animate radius growth/shrink during death sequence
                if let Some(start_radius) = self.death_animation_start_radius {
                    // Get the total duration from the initial timer value
                    let total_duration = match self.body_type {
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
                    };

                    // Calculate progress (0.0 at start → 1.0 at end)
                    let progress = 1.0 - (timer / total_duration);
                    let progress = progress.clamp(0.0, 1.0);

                    // Get the target radius multiplier for this body type
                    let target_multiplier = match self.body_type {
                        LargeBodyType::BlackHole => 2.5,
                        LargeBodyType::BlackHoleLarge => 4.0,
                        LargeBodyType::Star => 5.0,
                        LargeBodyType::NeutronStar => 0.5, // Collapse inward
                        LargeBodyType::Planet => 1.5,
                        LargeBodyType::GasGiant => 3.0,
                        LargeBodyType::LauncherMass => 1.2,
                        LargeBodyType::ExoticMatter => 8.0,
                        _ => 1.0, // No animation for others
                    };

                    // Interpolate radius (with optional easing)
                    let progress_eased = progress * progress; // Quadratic ease-in
                    self.radius = start_radius * (1.0 + (target_multiplier - 1.0) * progress_eased);
                    self.collision_radius =
                        self.radius * self.body_type.default_collision_radius_ratio();
                }

                // Count down death sequence timer
                let new_timer = timer - delta_time;
                if new_timer <= 0.0 {
                    self.on_death_sequence_complete();
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
    pub fn body_type(&self) -> LargeBodyType {
        self.body_type
    }
    pub fn radius(&self) -> f32 {
        self.radius
    }
    pub fn mass(&self) -> f32 {
        self.mass
    }
    pub fn velocity(&self) -> Vec3 {
        self.velocity
    }
    pub fn solar_wind_timer(&self) -> f32 {
        self.solar_wind_timer
    }
    pub fn solar_wind_interval(&self) -> f32 {
        self.solar_wind_interval
    }
    pub fn trail_particle_timer(&self) -> f32 {
        self.trail_particle_timer
    }
    pub fn rotation(&self) -> f32 {
        self.rotation
    }
    pub fn physics_index(&self) -> Option<usize> {
        self.physics_index
    }
    pub fn death_state(&self) -> DeathState {
        self.death_state
    }

    // Setters
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }
    pub fn set_velocity(&mut self, velocity: Vec3) {
        self.velocity = velocity;
    }
    pub fn set_solar_wind_timer(&mut self, timer: f32) {
        self.solar_wind_timer = timer;
    }
    pub fn set_trail_particle_timer(&mut self, timer: f32) {
        self.trail_particle_timer = timer;
    }
    pub fn set_death_state(&mut self, state: DeathState) {
        self.death_state = state;
    }

    // Mutable references
    pub fn physics_index_mut(&mut self) -> &mut Option<usize> {
        &mut self.physics_index
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

    /// Drain pending events from this body
    pub fn drain_events(&mut self) -> Vec<crate::engine::dispatcher::EventType> {
        std::mem::take(&mut self.pending_events)
    }

    /// Drain pending body spawn requests from this body
    pub fn drain_body_spawns(&mut self) -> Vec<BodySpawnRequest> {
        std::mem::take(&mut self.pending_body_spawns)
    }

    /// Get absorption count
    pub fn absorption_count(&self) -> u32 {
        self.absorption_count
    }

    /// Absorb another large body (for BlackHoleLarge)
    /// Returns true if absorption was successful
    pub fn absorb_body(&mut self, victim: &LargeBody) -> bool {
        // Only BlackHoleLarge can absorb
        if self.body_type != LargeBodyType::BlackHoleLarge {
            return false;
        }

        // converts victim's mass to absolute to allow whiteholes to grant mass to BlackHoleLarge
        let mass_gain = victim.mass.abs();
        self.mass += mass_gain;

        // Increase radius by 5%
        self.radius += victim.radius;
        self.collision_radius = self.radius * self.body_type.default_collision_radius_ratio();

        // REDUCE lifetime by 2 seconds per absorption (burning too hot = dies faster)
        if let Some(ref mut lifetime) = self.max_lifetime {
            *lifetime = (*lifetime - 2.0).max(5.0); // Minimum 5 seconds remaining
        }

        // Increment absorption counter
        self.absorption_count += 1;

        println!(
            "🕳️ BlackHoleLarge absorbed {:?} (mass +{:.0}, radius: {:.1}, absorptions: {}, lifetime: {:.1}s)",
            victim.body_type,
            mass_gain,
            self.radius,
            self.absorption_count,
            self.max_lifetime.unwrap_or(0.0)
        );

        // Queue absorption particle effects
        self.pending_events
            .push(crate::engine::dispatcher::EventType::Graphics(
                crate::engine::dispatcher::GraphicsEvent::SpawnParticles {
                    position: victim.position, // Spawn at victim's position
                    velocity: (self.position - victim.position).normalize() * 50.0, // Pull toward black hole
                    count: 300,
                    lifetime: 2.0,
                    color: victim.body_type.color(),
                },
            ));

        true
    }

    /// Trigger death sequence for this large body type
    fn trigger_death_sequence(&mut self) {
        // Store the starting radius for death animation
        self.death_animation_start_radius = Some(self.radius);

        match self.body_type {
            LargeBodyType::BlackHole => {
                // Hawking radiation evaporation - dramatic size increase then collapse
                println!("💥 BlackHole evaporating in burst of Hawking radiation!");
                // Radius will animate to 2.5x in update() during death sequence

                self.pending_events
                    .push(crate::engine::dispatcher::EventType::Explosion(
                        crate::engine::dispatcher::ExplosionEvent::Custom {
                            position: self.position,
                            max_radius: self.radius * 500.0,
                            force_strength: -50000.0,
                            duration: 2.0,
                            falloff_type: crate::scene::FalloffType::Linear,
                            damage: 0.0,
                            damage_radius: 0.0,
                            explosion_color: Color::MAGENTA,
                            particle_color: Color::MAGENTA,
                            particle_count: 500,
                        },
                    ));
            }

            LargeBodyType::BlackHoleLarge => {
                // Massive black hole evaporation - even more dramatic
                println!("🕳️💥 Supermassive BlackHole undergoing final evaporation!");
                // Radius will animate to 4.0x in update() during death sequence
                // Could spawn intense gravitational waves, spacetime distortion effects
                self.pending_events
                    .push(crate::engine::dispatcher::EventType::Explosion(
                        crate::engine::dispatcher::ExplosionEvent::Custom {
                            position: self.position,
                            max_radius: self.radius * 100.0,
                            force_strength: -500000.0,
                            duration: 2.0,
                            falloff_type: crate::scene::FalloffType::Linear,
                            damage: 0.0,
                            damage_radius: 0.0,
                            explosion_color: Color::MAGENTA,
                            particle_color: Color::MAGENTA,
                            particle_count: 5000,
                        },
                    ));
            }

            LargeBodyType::WhiteHole => {
                // Matter ejection finale - explosive outward burst
                println!("🌟 WhiteHole ejecting all accumulated matter!");
                // Radius stays same - no animation for WhiteHole
                self.mass *= 0.1; // Lose most mass in final ejection
                // Could spawn outward-moving matter particles
                self.pending_events
                    .push(crate::engine::dispatcher::EventType::Explosion(
                        crate::engine::dispatcher::ExplosionEvent::Custom {
                            position: self.position,
                            max_radius: self.radius * 500.0,
                            force_strength: 50000.0,
                            duration: 2.0,
                            falloff_type: crate::scene::FalloffType::Linear,
                            damage: 0.0,
                            damage_radius: 0.0,
                            explosion_color: Color::WHITE,
                            particle_color: Color::WHITE,
                            particle_count: 5000,
                        },
                    ));
            }

            LargeBodyType::Star => {
                // Supernova explosion - classic stellar death
                println!("⭐ Star going supernova!");
                // Radius will animate to 5.0x in update() during death sequence
                // Could spawn shockwave, change to red giant color
                self.pending_events
                    .push(crate::engine::dispatcher::EventType::Explosion(
                        crate::engine::dispatcher::ExplosionEvent::Custom {
                            position: self.position,
                            max_radius: self.radius * 10.0,
                            force_strength: 50000.0,
                            duration: 2.0,
                            falloff_type: crate::scene::FalloffType::Linear,
                            damage: 0.0,
                            damage_radius: 0.0,
                            explosion_color: Color::WHITE,
                            particle_color: Color::WHITE,
                            particle_count: 5000,
                        },
                    ));
            }

            LargeBodyType::NeutronStar => {
                // Collapse to black hole - density limit exceeded
                println!("🕳️  NeutronStar collapsing into black hole!");
                // Radius will animate to 0.5x in update() during death sequence
                self.mass *= 2.0; // Gravitational intensification
            }

            LargeBodyType::Planet => {
                // Atmospheric loss and core fragmentation
                println!("🌍 Planet losing atmosphere and breaking apart!");
                // Radius will animate to 1.5x in update() during death sequence
                // Could spawn debris particles
            }

            LargeBodyType::GasGiant => {
                // Gas dispersion - gradual deflation
                println!("🪐 Gas Giant dispersing atmospheric layers!");
                // Radius will animate to 3.0x in update() during death sequence
                self.mass *= 0.3; // Lose gas mass
            }

            LargeBodyType::LauncherMass => {
                // Fragmentation into smaller pieces
                println!("☄️  Asteroid fragmenting into debris field!");
                // Radius will animate to 1.2x in update() during death sequence
                // Could spawn multiple smaller asteroid particles
            }

            LargeBodyType::ExoticMatter => {
                // Matter/antimatter annihilation - most dramatic
                println!("⚡ Exotic Matter undergoing catastrophic annihilation!");
                // Radius will animate to 8.0x in update() during death sequence
                // Could spawn high-energy particles, light effects
            }

            LargeBodyType::Debug => {
                // Simple debug death
                println!("🔧 Debug body completing lifecycle test");
                // No special effects, just clean removal
            }
        }
    }

    /// Called when death sequence completes (right before removal)
    fn on_death_sequence_complete(&mut self) {
        match self.body_type {
            LargeBodyType::BlackHole => {
                // Final collapse - spawn intense particle burst and shockwave
                println!("🌌 BlackHole final collapse!");

                // Spawn final explosion
                self.pending_events
                    .push(crate::engine::dispatcher::EventType::Explosion(
                        crate::engine::dispatcher::ExplosionEvent::Custom {
                            position: self.position,
                            max_radius: self.radius * 500.0,
                            force_strength: 5000.0,
                            duration: 2.0,
                            falloff_type: crate::scene::FalloffType::Linear,
                            damage: 0.0,
                            damage_radius: 0.0,
                            explosion_color: Color::ORANGE,
                            particle_color: Color::ORANGE,
                            particle_count: 1000,
                        },
                    ));
            }

            LargeBodyType::BlackHoleLarge => {
                // Supermassive collapse - even more dramatic
                println!("💫 Supermassive BlackHole final collapse!");

                // Multiple shockwaves
                self.pending_events
                    .push(crate::engine::dispatcher::EventType::Explosion(
                        crate::engine::dispatcher::ExplosionEvent::Custom {
                            position: self.position,
                            max_radius: self.radius * 10.0,
                            force_strength: 100000.0,
                            duration: 2.0,
                            falloff_type: crate::scene::FalloffType::Linear,
                            damage: 0.0,
                            damage_radius: 0.0,
                            explosion_color: Color::MAGENTA,
                            particle_color: Color::MAGENTA,
                            particle_count: 2000,
                        },
                    ));
            }

            LargeBodyType::Star => {
                // Supernova remnant - spawn neutron star core
                println!("⭐ Star supernova complete - neutron star remnant forming!");

                // Final explosion burst
                self.pending_events
                    .push(crate::engine::dispatcher::EventType::Explosion(
                        crate::engine::dispatcher::ExplosionEvent::Custom {
                            position: self.position,
                            max_radius: self.radius * 10.0,
                            force_strength: 100000.0,
                            duration: 1.5,
                            falloff_type: crate::scene::FalloffType::Linear,
                            damage: 0.0,
                            damage_radius: 0.0,
                            explosion_color: Color::RED,
                            particle_color: Color::ORANGE,
                            particle_count: 5000,
                        },
                    ));

                // Spawn a NeutronStar remnant at the star's position
                // Inherits some of the star's velocity for realistic physics
                // Give it a longer lifetime than normal bodies (45s) since it's a dense remnant
                self.pending_body_spawns.push(BodySpawnRequest {
                    body_type: LargeBodyType::NeutronStar,
                    position: self.position,
                    velocity: self.velocity * 0.5, // Carry forward half the momentum
                    lifetime: Some(45.0), // Dense remnant lasts longer than typical spawned bodies
                });

                println!("🌟 → Neutron star remnant spawned (45s lifetime)");
            }

            LargeBodyType::ExoticMatter => {
                // Annihilation complete - reality warping effects
                println!("⚡ Exotic Matter annihilation complete!");

                // Multiple expanding energy rings
                self.pending_events
                    .push(crate::engine::dispatcher::EventType::Explosion(
                        crate::engine::dispatcher::ExplosionEvent::Custom {
                            position: self.position,
                            max_radius: self.radius * 10.0,
                            force_strength: 100000.0,
                            duration: 1.5,
                            falloff_type: crate::scene::FalloffType::Linear,
                            damage: 0.0,
                            damage_radius: 0.0,
                            explosion_color: Color::MAGENTA,
                            particle_color: Color::WHITE,
                            particle_count: 2000,
                        },
                    ));
            }

            LargeBodyType::Planet | LargeBodyType::GasGiant => {
                // Planetary breakup complete - debris field
                println!("🪐 Planetary body fragmentation complete!");
            }

            LargeBodyType::LauncherMass => {
                self.pending_events
                    .push(crate::engine::dispatcher::EventType::Graphics(
                        crate::engine::dispatcher::GraphicsEvent::SpawnShapeParticles {
                            position: self.position,
                            velocity: Vec3::zeros(),
                            count: 8,
                            lifetime: 1.0,
                            color: Color::WHITE,
                            primitive_type: PrimitiveType::Star2D,
                            angular_velocity: Vec3::new(3.0, 3.0, 3.0),
                            scale: 0.5,
                        },
                    ));

                self.pending_events
                    .push(crate::engine::dispatcher::EventType::Explosion(
                        crate::engine::dispatcher::ExplosionEvent::Custom {
                            position: self.position,
                            max_radius: self.radius * 10.0,
                            force_strength: 5000.0,
                            duration: 1.0,
                            falloff_type: crate::scene::FalloffType::Linear,
                            damage: 0.0,
                            damage_radius: 0.0,
                            explosion_color: Color::WHITE,
                            particle_color: Color::WHITE,
                            particle_count: 500,
                        },
                    ));
            }

            _ => {
                // Default: small particle effect
                self.pending_events
                    .push(crate::engine::dispatcher::EventType::Graphics(
                        crate::engine::dispatcher::GraphicsEvent::SpawnParticles {
                            position: self.position,
                            velocity: Vec3::zeros(),
                            count: 200,
                            lifetime: 5.0,
                            color: self.body_type.color(),
                        },
                    ));
            }
        }
    }

    /// Get the trail particle interval constant
    pub fn trail_interval() -> f32 {
        LARGE_BODY_TRAIL_INTERVAL
    }
}
