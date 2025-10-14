use crate::engine::Vec3;
use crate::engine::entity::EntityId;
use crate::graphics::{Color, Light, LightingUniforms, Primitive, PrimitiveType};
use crate::scene::PhysicsManager;

use super::body::{BodySpawnRequest, LargeBody};
use super::body_type::{DeathState, LargeBodyType};
use super::spawner::{BodySpawner, SpawnRequest};

// Distance culling configuration
const MAX_DISTANCE_FROM_ORIGIN: f32 = 5000.0; // Auto-destroy bodies beyond this distance

/// Manager for all large bodies in the game
pub struct LargeBodyManager {
    bodies: Vec<LargeBody>,
    pending_events: Vec<crate::engine::dispatcher::EventType>,
    spawner: Option<BodySpawner>,      // Optional automatic spawner
    pending_spawns: Vec<SpawnRequest>, // Queued spawn requests from spawner
    pending_body_spawns: Vec<BodySpawnRequest>, // Queued body spawn requests from death sequences
    pending_absorptions: Vec<(EntityId, EntityId)>, // (absorber_id, victim_id) pairs
}

impl LargeBodyManager {
    pub fn new() -> Self {
        Self {
            bodies: Vec::new(),
            pending_events: Vec::new(),
            spawner: None,
            pending_spawns: Vec::new(),
            pending_body_spawns: Vec::new(),
            pending_absorptions: Vec::new(),
        }
    }

    /// Enable automatic body spawning with default configuration
    pub fn enable_spawner(&mut self) {
        self.spawner = Some(BodySpawner::new());
        println!("🌌 Large body spawner enabled with default configuration");
    }

    /// Enable automatic body spawning with custom configuration
    pub fn enable_spawner_custom(&mut self, spawner: BodySpawner) {
        self.spawner = Some(spawner);
        println!("🌌 Large body spawner enabled with custom configuration");
    }

    /// Disable automatic body spawning
    pub fn disable_spawner(&mut self) {
        self.spawner = None;
        println!("🌌 Large body spawner disabled");
    }

    /// Get mutable reference to spawner for configuration
    pub fn spawner_mut(&mut self) -> Option<&mut BodySpawner> {
        self.spawner.as_mut()
    }

    /// Check if spawner is enabled
    pub fn is_spawner_enabled(&self) -> bool {
        self.spawner.is_some()
    }

    /// Queue an absorption (BlackHoleLarge absorbs another body)
    pub fn queue_absorption(&mut self, absorber_id: EntityId, victim_id: EntityId) {
        self.pending_absorptions.push((absorber_id, victim_id));
    }

    /// Trigger an asteroid shower event - spawns multiple launcher masses from a direction
    pub fn trigger_asteroid_shower(
        &mut self,
        direction_from: Vec3, // Direction the asteroids come from (will be normalized)
        count: usize,
        speed_range: (f32, f32), // (min, max) speed in units/second
        spread_angle: f32,       // Cone spread angle in radians (0.0 = parallel, PI = hemisphere)
        lifetime: f32,           // Lifetime for all asteroids
        physics: &mut PhysicsManager,
        entity_manager: &mut crate::engine::entity::EntityManager,
    ) {
        let direction = direction_from.normalize();

        // Spawn position: far out in the direction (beyond normal play area)
        let spawn_distance = 1000.0; // Start far outside the play area
        let base_spawn_pos = direction * spawn_distance;

        println!(
            "☄️  Triggering asteroid shower: {} asteroids from direction {:?}",
            count, direction
        );

        for i in 0..count {
            // Add random spread to spawn position (creates a cluster effect)
            let spread_radius = 50.0 + (i as f32 * 10.0); // Spread increases with each asteroid
            let random_offset = Vec3::new(
                (rand::random::<f32>() - 0.5) * spread_radius,
                (rand::random::<f32>() - 0.5) * spread_radius,
                (rand::random::<f32>() - 0.5) * spread_radius,
            );
            let spawn_pos = base_spawn_pos + random_offset;

            // Calculate velocity toward center with spread
            let base_velocity_direction = -direction; // Opposite of spawn direction

            // Add cone spread using spherical coordinates
            let theta = (rand::random::<f32>() - 0.5) * spread_angle;
            let phi = rand::random::<f32>() * std::f32::consts::TAU;

            // Create perpendicular vectors for rotation
            let arbitrary = if direction.y.abs() < 0.9 {
                Vec3::new(0.0, 1.0, 0.0)
            } else {
                Vec3::new(1.0, 0.0, 0.0)
            };
            let perp1 = direction.cross(&arbitrary).normalize();
            let perp2 = direction.cross(&perp1).normalize();

            // Apply spread rotation
            let spread_offset =
                perp1 * (theta.sin() * phi.cos()) + perp2 * (theta.sin() * phi.sin());
            let velocity_direction = (base_velocity_direction + spread_offset).normalize();

            // Random speed within range
            let speed = speed_range.0 + rand::random::<f32>() * (speed_range.1 - speed_range.0);
            let velocity = velocity_direction * speed;

            // Random lifetime multiplier per asteroid
            let lifetime = lifetime * rand::random_range(0.5..1.5);

            // Spawn the launcher mass with velocity and lifetime
            let _entity_id = self.spawn_body_with_velocity_and_lifetime(
                LargeBodyType::LauncherMass,
                spawn_pos,
                velocity,
                lifetime,
                physics,
                entity_manager,
            );
        }

        println!(
            "☄️  Asteroid shower launched! {} asteroids heading toward center",
            count
        );
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
        lifetime: f32,
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

        // Set lifetime for both bodies in the binary pair
        if let Some(body1) = self.bodies.iter_mut().find(|b| b.entity_id() == entity1) {
            body1.set_max_lifetime(Some(lifetime));
        }
        if let Some(body2) = self.bodies.iter_mut().find(|b| b.entity_id() == entity2) {
            body2.set_max_lifetime(Some(lifetime));
        }

        (entity1, entity2)
    }

    /// Update all large bodies
    pub fn update(
        &mut self,
        delta_time: f32,
        player_position: Vec3,
        physics_manager: &mut PhysicsManager,
        entity_manager: &mut crate::engine::entity::EntityManager,
    ) {
        // First update all living bodies
        for body in &mut self.bodies {
            // Update the body and check for solar wind events
            if (body.body_type() == LargeBodyType::Star && body.solar_wind_interval() > 0.0)
                || (body.body_type() == LargeBodyType::WhiteHole
                    && body.solar_wind_interval() > 0.0)
            {
                body.set_solar_wind_timer(body.solar_wind_timer() - delta_time);
                if body.solar_wind_timer() <= 0.0 {
                    // Reset timer for next emission with random interval
                    body.set_solar_wind_timer(body.solar_wind_interval());

                    // Queue solar wind event
                    self.pending_events
                        .push(crate::engine::dispatcher::EventType::Explosion(
                            crate::engine::dispatcher::ExplosionEvent::SolarWind {
                                position: body.position(),
                            },
                        ));
                    if body.body_type() == LargeBodyType::Star {
                        self.pending_events.push(crate::engine::EventType::Graphics(
                            crate::engine::GraphicsEvent::SpawnParticles {
                                position: body.position(),
                                velocity: Vec3::zeros(),
                                count: 1000,
                                lifetime: 15.0,
                                color: Color::RED,
                            },
                        ));
                    } else if body.body_type() == LargeBodyType::WhiteHole {
                        // Spawn radiating lightning bolts for WhiteHole pulse
                        let lightning_count = 8;
                        let pulse_radius = body.radius() * 100.0;

                        for i in 0..lightning_count {
                            // Create evenly distributed directions around the sphere
                            let theta = (i as f32 / lightning_count as f32) * std::f32::consts::TAU;
                            let phi = rand::random_range(0.0..std::f32::consts::PI);

                            // Calculate endpoint in spherical coordinates
                            let direction = Vec3::new(
                                phi.sin() * theta.cos(),
                                phi.sin() * theta.sin(),
                                phi.cos(),
                            );

                            let end_point = body.position() + direction * pulse_radius;

                            // Spawn lightning bolt radiating outward from white hole
                            self.pending_events.push(crate::engine::EventType::Graphics(
                                crate::engine::GraphicsEvent::SpawnLightning {
                                    start: body.position(),
                                    end: end_point,
                                },
                            ));
                        }
                    }
                }
            } else if body.body_type() == LargeBodyType::NeutronStar
                && body.solar_wind_interval() > 0.0
            {
                body.set_solar_wind_timer(body.solar_wind_timer() - delta_time);
                if body.solar_wind_timer() <= 0.0 {
                    body.set_solar_wind_timer(body.solar_wind_interval());
                    self.pending_events
                        .push(crate::engine::dispatcher::EventType::Explosion(
                            crate::engine::dispatcher::ExplosionEvent::AntiWind {
                                position: body.position(),
                            },
                        ));
                }
            } else if body.body_type() == LargeBodyType::ExoticMatter
                && body.solar_wind_interval() > 0.0
            {
                body.set_solar_wind_timer(body.solar_wind_timer() - delta_time);
                if body.solar_wind_timer() <= 0.0 {
                    body.set_solar_wind_timer(body.solar_wind_interval());
                    self.pending_events
                        .push(crate::engine::dispatcher::EventType::Explosion(
                            crate::engine::dispatcher::ExplosionEvent::Custom {
                                position: body.position(),
                                max_radius: 80.0,
                                force_strength: 10000.0,
                                duration: 2.0,
                                falloff_type: super::super::FalloffType::Quadratic,
                                damage: 0.0, // Solar wind doesn't deal damage, just physics force
                                damage_radius: 0.0,
                                explosion_color: Color::MAGENTA,
                                particle_color: Color::MAGENTA,
                                particle_count: 0,
                            },
                        ));

                    // Spawn radiating lightning bolts for ExoticMatter pulse
                    let lightning_count = 4;
                    let pulse_radius = body.radius() * 10.0;

                    for i in 0..lightning_count {
                        // Create evenly distributed directions around the sphere
                        let theta = (i as f32 / lightning_count as f32) * std::f32::consts::TAU;
                        let phi = rand::random_range(0.0..std::f32::consts::PI);

                        // Calculate endpoint in spherical coordinates
                        let direction =
                            Vec3::new(phi.sin() * theta.cos(), phi.sin() * theta.sin(), phi.cos());

                        let end_point = body.position() + direction * pulse_radius;

                        // Spawn lightning bolt radiating outward from exotic matter
                        self.pending_events.push(crate::engine::EventType::Graphics(
                            crate::engine::GraphicsEvent::SpawnLightning {
                                start: body.position(),
                                end: end_point,
                            },
                        ));
                    }
                    self.pending_events.push(crate::engine::EventType::Graphics(
                        crate::engine::GraphicsEvent::SpawnParticles {
                            position: body.position(),
                            velocity: Vec3::zeros(),
                            count: 10,
                            lifetime: 15.0,
                            color: Color::PINK,
                        },
                    ));
                }
            }

            // Particle trail system - spawn trail particles for all bodies
            body.set_trail_particle_timer(body.trail_particle_timer() - delta_time);
            if body.trail_particle_timer() <= 0.0 {
                // Reset timer
                body.set_trail_particle_timer(LargeBody::trail_interval());

                // Calculate spawn position behind the body's movement
                // This prevents particles from spawning inside the body and being launched
                let spawn_offset = if body.velocity().magnitude() > 0.01 {
                    // Body is moving - spawn behind it
                    let velocity_dir = body.velocity().normalize();
                    -velocity_dir * body.radius() // Behind the body at radius distance
                } else {
                    // Body is stationary - spawn at radius distance in a consistent direction
                    Vec3::new(0.0, body.radius(), 0.0) // Spawn above
                };

                let spawn_position = body.position() + spawn_offset;

                // Queue particle spawn event with body-type-specific color
                // Used to generate particle trail effect following around each large body
                self.pending_events
                    .push(crate::engine::dispatcher::EventType::Graphics(
                        crate::engine::dispatcher::GraphicsEvent::SpawnParticles {
                            position: spawn_position,
                            velocity: Vec3::zeros(), // Stationary - let gravity move them!
                            count: 1,                // Fewer than player trail
                            lifetime: 15.0,          // Longer lifetime to see gravity effects
                            color: body.body_type().color(), // Match body color
                        },
                    ));
            }

            body.update(delta_time);
            // Update position/velocity from physics simulation
            body.update_from_physics(physics_manager);

            // Collect events from body (e.g., death sequence effects)
            let body_events = body.drain_events();
            self.pending_events.extend(body_events);

            // Collect body spawn requests (e.g., star -> neutron star)
            let body_spawns = body.drain_body_spawns();
            self.pending_body_spawns.extend(body_spawns);

            // Distance culling - mark bodies that drift too far for removal
            let distance_from_origin = body.position().magnitude();
            if distance_from_origin > MAX_DISTANCE_FROM_ORIGIN {
                // Mark for immediate removal without death sequence
                body.set_death_state(DeathState::ReadyForRemoval);
            }
        }

        // Remove dead bodies (iterate in reverse to avoid index issues)
        let mut i = self.bodies.len();
        while i > 0 {
            i -= 1;
            if self.bodies[i].is_dead() {
                let body = self.bodies.remove(i);

                // Remove from physics system and fix indices
                if let Some(physics_index) = body.physics_index() {
                    physics_manager.remove_gravitational_body(physics_index);

                    // Update physics indices for all remaining bodies that had higher indices
                    for remaining_body in &mut self.bodies {
                        if let Some(remaining_index) = remaining_body.physics_index_mut() {
                            if *remaining_index > physics_index {
                                *remaining_index -= 1;
                            }
                        }
                    }
                }

                // Remove from entity system
                entity_manager.destroy_entity(body.entity_id());

                println!(
                    "🪦 Large body {:?} died of old age at {:.1}s",
                    body.body_type(),
                    body.age()
                );
            }
        }

        // Update spawner if enabled (after removing dead bodies so count is accurate)
        if let Some(spawner) = &mut self.spawner {
            if let Some(spawn_request) =
                spawner.update(delta_time, self.bodies.len(), player_position)
            {
                // Queue the spawn request instead of spawning immediately
                // This prevents index tracking issues during the update loop
                self.pending_spawns.push(spawn_request);
            }
        }

        // Process all pending spawns AFTER all physics and index updates are complete
        // This ensures physics indices remain consistent
        // Take ownership of pending spawns to avoid borrow checker issues
        let spawns_to_process = std::mem::take(&mut self.pending_spawns);
        for spawn_request in spawns_to_process {
            match spawn_request {
                SpawnRequest::SingleBody {
                    body_type,
                    position,
                    lifetime,
                } => {
                    let _entity_id = self.spawn_body_with_lifetime(
                        body_type,
                        position,
                        lifetime,
                        physics_manager,
                        entity_manager,
                    );

                    println!(
                        "🌌 Spawner: Created {:?} at {:?} with {:.1}s lifetime (total: {})",
                        body_type,
                        position,
                        lifetime,
                        self.bodies.len()
                    );
                }
                SpawnRequest::AsteroidShower {
                    direction,
                    count,
                    speed_range,
                    spread_angle,
                    lifetime,
                } => {
                    // ASTEROID SHOWER EVENT!
                    self.trigger_asteroid_shower(
                        direction,
                        count,
                        speed_range,
                        spread_angle,
                        lifetime,
                        physics_manager,
                        entity_manager,
                    );
                }
                SpawnRequest::BinaryPair {
                    body_type1,
                    body_type2,
                    center_position,
                    separation_distance,
                    lifetime,
                } => {
                    // BINARY PAIR EVENT!
                    let (id1, id2) = self.spawn_binary_pair(
                        body_type1,
                        body_type2,
                        center_position,
                        separation_distance,
                        lifetime,
                        physics_manager,
                        entity_manager,
                    );

                    println!(
                        "🌌 Spawner: Created binary pair {:?} + {:?} at {:?} (separation: {:.1}, lifetime: {:.1}s) (IDs: {:?}, {:?}, total: {})",
                        body_type1,
                        body_type2,
                        center_position,
                        separation_distance,
                        lifetime,
                        id1,
                        id2,
                        self.bodies.len()
                    );
                }
                SpawnRequest::RogueBody {
                    body_type,
                    position,
                    velocity,
                    lifetime,
                } => {
                    // ROGUE BODY EVENT!
                    let _entity_id = self.spawn_body_with_velocity_and_lifetime(
                        body_type,
                        position,
                        velocity,
                        lifetime,
                        physics_manager,
                        entity_manager,
                    );

                    println!(
                        "🌌 Spawner: Created rogue {:?} at {:?} with velocity {:?} (speed: {:.1}, lifetime: {:.1}s, total: {})",
                        body_type,
                        position,
                        velocity,
                        velocity.magnitude(),
                        lifetime,
                        self.bodies.len()
                    );
                }
            }
        }

        // Process all pending body spawn requests (from death sequences)
        // These have velocity and need special handling
        let body_spawns_to_process = std::mem::take(&mut self.pending_body_spawns);
        for spawn_request in body_spawns_to_process {
            if let Some(lifetime) = spawn_request.lifetime {
                // Spawn with lifetime
                let _entity_id = self.spawn_body_with_velocity_and_lifetime(
                    spawn_request.body_type,
                    spawn_request.position,
                    spawn_request.velocity,
                    lifetime,
                    physics_manager,
                    entity_manager,
                );
            } else {
                // Spawn eternal body with velocity
                let entity_id =
                    entity_manager.create_entity(crate::engine::entity::EntityType::LargeBody);
                let mut body =
                    LargeBody::new(entity_id, spawn_request.body_type, spawn_request.position);
                body.set_velocity(spawn_request.velocity);
                body.register_with_physics(physics_manager);
                self.bodies.push(body);
            }

            println!(
                "💫 Death-spawned: {:?} at {:?} (total bodies: {})",
                spawn_request.body_type,
                spawn_request.position,
                self.bodies.len()
            );
        }

        // Process all pending absorptions (BlackHoleLarge absorbing other bodies)
        let absorptions_to_process = std::mem::take(&mut self.pending_absorptions);
        for (absorber_id, victim_id) in absorptions_to_process {
            // Find both bodies
            let absorber_idx = self
                .bodies
                .iter()
                .position(|b| b.entity_id() == absorber_id);
            let victim_idx = self.bodies.iter().position(|b| b.entity_id() == victim_id);

            if let (Some(abs_idx), Some(vic_idx)) = (absorber_idx, victim_idx) {
                // Clone victim data for the absorption (we need it immutably)
                let victim_data = self.bodies[vic_idx].clone();

                // Perform absorption on the absorber
                let success = self.bodies[abs_idx].absorb_body(&victim_data);

                if success {
                    // Collect any events generated by the absorption (particles, etc.)
                    let absorption_events = self.bodies[abs_idx].drain_events();
                    self.pending_events.extend(absorption_events);

                    // Mark victim for removal by setting it to ReadyForRemoval state
                    self.bodies[vic_idx].set_death_state(DeathState::ReadyForRemoval);

                    // The victim will be cleaned up in the next update cycle's dead body removal
                }
            }
        }
    }

    /// Drain pending events (called by dispatcher)
    pub fn drain_events(&mut self) -> Vec<crate::engine::dispatcher::EventType> {
        std::mem::take(&mut self.pending_events)
    }

    /// Get render data for all bodies (including atmosphere spheres)
    pub fn get_render_data(&self) -> Vec<Primitive> {
        let mut primitives = Vec::new();

        for body in &self.bodies {
            // Add the main body wireframe
            primitives.push(body.get_render_data());

            // Add atmosphere sphere if this body type has one
            if let Some(atmo_multiplier) = body.body_type().atmosphere_radius_multiplier() {
                let atmosphere = Primitive::new(
                    PrimitiveType::Sphere,
                    body.position(),
                    body.body_type().atmosphere_color(),
                )
                .with_uniform_scale(body.radius() * atmo_multiplier * 2.0) // Sphere diameter
                .with_rotation(Vec3::new(0.0, body.rotation(), 0.0));

                primitives.push(atmosphere);
            }
        }

        primitives
    }

    /// Get reference to all bodies
    pub fn bodies(&self) -> &[LargeBody] {
        &self.bodies
    }

    /// Get body by entity ID
    pub fn get_body(&self, entity_id: EntityId) -> Option<&LargeBody> {
        self.bodies.iter().find(|b| b.entity_id() == entity_id)
    }

    /// Collect lighting data from all large bodies for rendering
    pub fn get_lighting_data(&self) -> LightingUniforms {
        let mut lighting = LightingUniforms::new();

        for body in &self.bodies {
            // Skip bodies that are dead or too far away
            if body.is_dead() {
                continue;
            }

            // Determine light properties based on body type
            let (intensity, color, radius) = match body.body_type() {
                // POSITIVE LIGHT SOURCES (Brightness emitters)
                LargeBodyType::Star => {
                    // Stars emit intense warm light
                    let intensity = 0.8 * (body.mass() / 200_000.0).min(2.0);
                    let color = Color {
                        r: 1.0,
                        g: 0.9,
                        b: 0.6,
                        a: 1.0,
                    };
                    let radius = body.radius() * 10.0; // Wide influence
                    (intensity, color, radius)
                }
                LargeBodyType::WhiteHole => {
                    // White holes emit intense repulsive white light
                    let intensity = 0.6 * (body.mass().abs() / 150_000.0).min(1.5);
                    let color = Color {
                        r: 0.9,
                        g: 0.95,
                        b: 1.0,
                        a: 1.0,
                    };
                    let radius = body.radius() * 100.0;
                    (intensity, color, radius)
                }
                LargeBodyType::NeutronStar => {
                    // Neutron stars emit harsh green-white light
                    let intensity = 0.5 * (body.mass() / 100_000.0).min(1.0);
                    let color = Color {
                        r: 0.8,
                        g: 1.0,
                        b: 0.9,
                        a: 1.0,
                    };
                    let radius = body.radius() * 100.0; // Compact but intense
                    (intensity, color, radius)
                }
                LargeBodyType::Planet | LargeBodyType::GasGiant => {
                    // Planets emit soft reflected light
                    let intensity = 0.15 * (body.radius() / 30.0).min(0.5);
                    let body_color = body.body_type().color();
                    let color = Color {
                        r: body_color.r * 0.7,
                        g: body_color.g * 0.7,
                        b: body_color.b * 0.7,
                        a: 1.0,
                    };
                    let radius = body.radius() * 100.0;
                    (intensity, color, radius)
                }

                // NEGATIVE LIGHT SOURCES (Darkness emitters)
                LargeBodyType::BlackHole => {
                    // Black holes emit darkness
                    let intensity = -0.6 * (body.mass() / 200_000.0).min(1.5);
                    let color = Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.1, // Subtle blue tint to the darkness
                        a: 1.0,
                    };
                    let radius = body.radius() * 100.0; // Wide darkness influence
                    (intensity, color, radius)
                }
                LargeBodyType::BlackHoleLarge => {
                    // Large black holes emit more intense darkness
                    let intensity = -1.0 * (body.mass() / 300_000.0).min(2.5);
                    let color = Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.05,
                        a: 1.0,
                    };
                    let radius = body.radius() * 10.0; // Massive darkness radius
                    (intensity, color, radius)
                }
                LargeBodyType::ExoticMatter => {
                    // Exotic matter creates weird flickering darkness
                    let time_factor = (body.age() * 2.0).sin() * 0.5 + 0.5;
                    let intensity = -0.4 * time_factor;
                    let color = Color {
                        r: 0.2,
                        g: 0.1,
                        b: 0.3, // Purple-tinted darkness
                        a: 1.0,
                    };
                    let radius = body.radius() * 100.0;
                    (intensity, color, radius)
                }

                // NON-EMITTING BODIES
                LargeBodyType::LauncherMass | LargeBodyType::Debug => {
                    // These don't emit light
                    continue;
                }
            };

            // Create light and add to uniforms
            let light = Light::new(body.position(), intensity, color, radius);
            if !lighting.add_light(light) {
                // Hit max lights, stop adding
                break;
            }
        }

        lighting
    }
}
