use super::entity::Enemy;
use super::types::EnemyType;
use crate::engine::Vec3;
use crate::engine::dispatcher::{EnemyEvent, EventType, GraphicsEvent};
use crate::graphics::{Color, Primitive};

pub struct EnemyManager {
    enemies: Vec<Enemy>,
    event_queue: Vec<EventType>,
    spawn_timer: f32,
    spawn_interval: f32, // Seconds between spawns
}

impl EnemyManager {
    pub fn new() -> Self {
        Self {
            enemies: Vec::new(),
            event_queue: Vec::new(),
            spawn_timer: 0.0,
            // Spawn interval: how many enemies spawned per second
            spawn_interval: 0.5, // Spawn every second
        }
    }

    pub fn spawn_initial_enemies(
        &mut self,
        entity_manager: &mut crate::engine::entity::EntityManager,
    ) {
        use rand::Rng;

        // 1. Orbital Ring Formation (classic spiral around center)
        for i in 0..50 {
            let angle = (i as f32 / 50.0) * std::f32::consts::TAU;
            let radius = 8.0;
            let pos = Vec3::new(
                angle.cos() * radius,
                (i as f32) * 0.25 - 1.0,
                angle.sin() * radius,
            );
            let vel = Vec3::new(-angle.sin() * 2.0, 0.0, angle.cos() * 2.0);

            let enemy_entity =
                entity_manager.create_entity(crate::engine::entity::EntityType::Enemy);
            self.enemies
                .push(Enemy::new(enemy_entity, pos, vel, EnemyType::random()));
        }

        // 2. Spherical Shell Formation (hollow sphere of enemies)
        for _i in 0..100 {
            let theta = rand::rng().random_range(0.0..std::f32::consts::TAU);
            let phi = rand::rng().random_range(0.0..std::f32::consts::PI);
            let radius = 25.0;

            let sin_phi = phi.sin();
            let pos = Vec3::new(
                sin_phi * theta.cos() * radius,
                phi.cos() * radius,
                sin_phi * theta.sin() * radius,
            );
            let vel = Vec3::new(0.0, 0.0, 0.0); // Start stationary, let physics take over

            let enemy_entity =
                entity_manager.create_entity(crate::engine::entity::EntityType::Enemy);
            self.enemies
                .push(Enemy::new(enemy_entity, pos, vel, EnemyType::random()));
        }

        // 3. Linear Stream Formation (enemies in lines toward center)
        for direction in 0..8 {
            let base_angle = (direction as f32 / 8.0) * std::f32::consts::TAU;
            for distance in 1..20 {
                let radius = distance as f32 * 5.0;
                let pos = Vec3::new(
                    base_angle.cos() * radius,
                    rand::rng().random_range(-3.0..3.0),
                    base_angle.sin() * radius,
                );
                // Velocity toward center with slight randomness
                let vel = Vec3::new(
                    -base_angle.cos() * 1.5 + rand::rng().random_range(-0.5..0.5),
                    rand::rng().random_range(-0.2..0.2),
                    -base_angle.sin() * 1.5 + rand::rng().random_range(-0.5..0.5),
                );

                let enemy_entity =
                    entity_manager.create_entity(crate::engine::entity::EntityType::Enemy);
                self.enemies
                    .push(Enemy::new(enemy_entity, pos, vel, EnemyType::random()));
            }
        }

        // 4. Vertical Tower Formation (enemies stacked vertically)
        for layer in 0..15 {
            let y = (layer as f32 - 7.0) * 4.0;
            let enemies_in_layer = 8 + (layer % 3) * 2; // Varying density per layer

            for i in 0..enemies_in_layer {
                let angle = (i as f32 / enemies_in_layer as f32) * std::f32::consts::TAU;
                let radius = 15.0 + (layer as f32 * 0.5); // Slightly expanding radius
                let pos = Vec3::new(angle.cos() * radius, y, angle.sin() * radius);
                // Slow orbital motion around the tower
                let vel = Vec3::new(
                    -angle.sin() * 0.5,
                    rand::rng().random_range(-0.1..0.1),
                    angle.cos() * 0.5,
                );

                let enemy_entity =
                    entity_manager.create_entity(crate::engine::entity::EntityType::Enemy);
                self.enemies
                    .push(Enemy::new(enemy_entity, pos, vel, EnemyType::random()));
            }
        }

        // 5. Chaotic Cloud Formation (random cluster far away)
        let cloud_center = Vec3::new(50.0, 20.0, 50.0);
        for _i in 0..80 {
            let offset = Vec3::new(
                rand::rng().random_range(-10.0..10.0),
                rand::rng().random_range(-10.0..10.0),
                rand::rng().random_range(-10.0..10.0),
            );
            let pos = cloud_center + offset;
            let vel = Vec3::new(
                rand::rng().random_range(-1.0..1.0),
                rand::rng().random_range(-1.0..1.0),
                rand::rng().random_range(-1.0..1.0),
            );

            let enemy_entity =
                entity_manager.create_entity(crate::engine::entity::EntityType::Enemy);
            self.enemies
                .push(Enemy::new(enemy_entity, pos, vel, EnemyType::random()));
        }
    }

    pub fn update(
        &mut self,
        dt: f32,
        player_pos: Vec3,
        entity_manager: &mut crate::engine::entity::EntityManager,
    ) {
        // Tick eating cooldowns
        for enemy in self.enemies.iter_mut() {
            enemy.tick_eating_cooldown(dt);
        }

        // Handle Cannibal eating behavior (process before regular updates)
        let mut enemies_to_eat = Vec::new();

        for i in 0..self.enemies.len() {
            if self.enemies[i].is_cannibal() && self.enemies[i].can_eat() {
                // Find nearest basic enemy
                let cannibal_pos = self.enemies[i].position();
                let mut nearest_distance_sq = f32::MAX;
                let mut nearest_index = None;

                for j in 0..self.enemies.len() {
                    if i != j && self.enemies[j].is_basic_enemy() {
                        let prey_pos = self.enemies[j].position();
                        let diff = prey_pos - cannibal_pos;
                        let dist_sq = diff.magnitude_squared();

                        // Check if within eating range (2.0 units)
                        if dist_sq < 4.0 && dist_sq < nearest_distance_sq {
                            nearest_distance_sq = dist_sq;
                            nearest_index = Some(j);
                        }
                    }
                }

                // If found a prey within range, mark for eating
                if let Some(prey_idx) = nearest_index {
                    enemies_to_eat.push((i, prey_idx));
                }
            }
        }

        // Process eating (remove prey and grow cannibal)
        // Sort in reverse to handle index changes correctly
        enemies_to_eat.sort_by(|a, b| b.1.cmp(&a.1));

        for (cannibal_idx, prey_idx) in enemies_to_eat {
            // Get prey position for particles before removing
            let prey_pos = self.enemies[prey_idx].position();

            // Remove prey enemy
            self.enemies.remove(prey_idx);

            // Adjust cannibal index if prey was before it
            let adjusted_cannibal_idx = if prey_idx < cannibal_idx {
                cannibal_idx - 1
            } else {
                cannibal_idx
            };

            // Grow the cannibal
            if adjusted_cannibal_idx < self.enemies.len() {
                self.enemies[adjusted_cannibal_idx].consume_enemy();

                // Spawn red eating particles
                self.event_queue
                    .push(EventType::Graphics(GraphicsEvent::SpawnParticles {
                        position: prey_pos,
                        velocity: Vec3::new(0.0, 0.0, 0.0),
                        count: 30,
                        lifetime: 1.0,
                        color: Color::new(0.8, 0.0, 0.0, 1.0), // Dark red
                    }));
            }
        }

        // Update core vulnerability states based on active shield counts
        for i in 0..self.enemies.len() {
            if self.enemies[i].is_shield_orb_core() {
                let core_id = self.enemies[i].entity_id();

                // Count active shields for this core
                let active_shield_count = self
                    .enemies
                    .iter()
                    .filter(|e| {
                        if let Some(shield_core_id) = e.shield_core_id() {
                            shield_core_id == core_id && e.is_alive()
                        } else {
                            false
                        }
                    })
                    .count();

                self.enemies[i].update_vulnerability(active_shield_count);
            }
        }

        // Regular enemy updates with specialized behavior for different types
        for i in 0..self.enemies.len() {
            if self.enemies[i].is_shield() {
                // Shields orbit around their core
                if let Some(core_id) = self.enemies[i].shield_core_id() {
                    // Find the core position
                    if let Some(core) = self.enemies.iter().find(|e| e.entity_id() == core_id) {
                        let core_pos = core.position();
                        self.enemies[i].update_shield_orbit(dt, core_pos);
                    } else {
                        // Core is dead - shield should die too
                        self.enemies[i].take_damage(9999.0);
                    }
                }
            } else if self.enemies[i].is_cannibal() {
                // Cannibals seek nearest basic enemy instead of player
                let cannibal_pos = self.enemies[i].position();
                let mut nearest_distance_sq = f32::MAX;
                let mut target_pos = player_pos; // Default to player if no prey found

                for j in 0..self.enemies.len() {
                    if i != j && self.enemies[j].is_basic_enemy() {
                        let prey_pos = self.enemies[j].position();
                        let diff = prey_pos - cannibal_pos;
                        let dist_sq = diff.magnitude_squared();

                        if dist_sq < nearest_distance_sq {
                            nearest_distance_sq = dist_sq;
                            target_pos = prey_pos;
                        }
                    }
                }

                self.enemies[i].update_with_target(dt, target_pos);
            } else {
                // Regular enemies and cores seek player
                self.enemies[i].update(dt, player_pos);
            }
        }

        // Check for dead enemies and generate death events with position before removing them
        let mut enemies_to_remove = Vec::new();
        for enemy in &self.enemies {
            if !enemy.is_alive() {
                enemies_to_remove.push((
                    enemy.entity_id(),
                    enemy.position(),
                    enemy.enemy_type().clone(),
                ));
            }
        }

        // Generate death events and handle splitting for dead enemies
        for (enemy_id, position, enemy_type) in enemies_to_remove {
            self.event_queue
                .push(EventType::Enemy(EnemyEvent::Die { enemy_id }));

            // Also generate immediate death particles
            self.event_queue
                .push(EventType::Graphics(GraphicsEvent::SpawnParticles {
                    position,
                    velocity: crate::engine::Vec3::new(0.0, 0.0, 0.0),
                    count: 150,
                    lifetime: 2.0,
                    color: Color::GREEN,
                }));

            // Check if this is a Splitter that should split - do it here before removal
            if let EnemyType::Splitter {
                current_generation,
                max_generation,
            } = enemy_type
            {
                if current_generation < max_generation {
                    // Spawn 2 child splitters immediately
                    use rand::Rng;

                    let next_generation = current_generation + 1;

                    // Create 2 children offset from parent position with outward velocity
                    for i in 0..2 {
                        let angle = (i as f32 / 2.0) * std::f32::consts::TAU + rand::rng().random_range(0.0..0.5);
                        let offset_distance = 2.0; // Spawn slightly away from parent

                        let offset = Vec3::new(
                            angle.cos() * offset_distance,
                            rand::rng().random_range(-1.0..1.0), // Random vertical offset
                            angle.sin() * offset_distance,
                        );

                        // Velocity pointing outward from split point
                        let outward_speed = 5.0;
                        let vel = Vec3::new(
                            angle.cos() * outward_speed,
                            rand::rng().random_range(-2.0..2.0),
                            angle.sin() * outward_speed,
                        );

                        // Create entity ID using entity_manager
                        let enemy_entity = entity_manager.create_entity(crate::engine::entity::EntityType::Enemy);

                        self.enemies.push(Enemy::new(
                            enemy_entity,
                            position + offset,
                            vel,
                            EnemyType::Splitter {
                                current_generation: next_generation,
                                max_generation,
                            },
                        ));
                    }

                    // Spawn particles for the splitter split effect
                    self.event_queue
                        .push(EventType::Graphics(GraphicsEvent::SpawnParticles {
                            position,
                            velocity: Vec3::new(0.0, 0.0, 0.0),
                            count: 50, // Burst of particles at split point
                            lifetime: 1.5,
                            color: Color::ORANGE, // Bright split effect
                        }));
                }
            }

            // Check if this is a Shield that should split - fractal defense
            if let EnemyType::Shield {
                current_generation,
                max_generation,
                core_id,
                orbit_angle,
                orbit_inclination,
                orbit_radius,
            } = enemy_type
            {
                if current_generation < max_generation {
                    // Spawn 2 child shields immediately at same orbit
                    use rand::Rng;

                    let next_generation = current_generation + 1;

                    // Create 2 children at slightly different orbital positions
                    for i in 0..2 {
                        // Offset angle slightly so they don't spawn on top of each other
                        let angle_offset = (i as f32 - 0.5) * 0.5; // ±0.25 radians
                        let new_orbit_angle = orbit_angle + angle_offset;

                        // Create entity ID using entity_manager
                        let enemy_entity =
                            entity_manager.create_entity(crate::engine::entity::EntityType::Enemy);

                        self.enemies.push(Enemy::new(
                            enemy_entity,
                            position, // Start at parent position
                            Vec3::zeros(), // Shields have no velocity
                            EnemyType::Shield {
                                current_generation: next_generation,
                                max_generation,
                                core_id,
                                orbit_angle: new_orbit_angle,
                                orbit_inclination, // Keep same inclination (same latitude on sphere)
                                orbit_radius,      // Keep same orbit radius
                            },
                        ));
                    }

                    // Spawn blue particles for shield split
                    self.event_queue
                        .push(EventType::Graphics(GraphicsEvent::SpawnParticles {
                            position,
                            velocity: Vec3::new(0.0, 0.0, 0.0),
                            count: 30,
                            lifetime: 1.0,
                            color: Color::new(0.3, 0.6, 1.0, 1.0), // Blue shield particles
                        }));
                }
            }
        }

        // Now remove the dead enemies (after spawning children)
        self.enemies.retain(|e| e.is_alive());

        self.spawn_timer += dt;
        if self.spawn_timer >= self.spawn_interval {
            self.spawn_timer = 0.0;
            self.spawn_enemy_near_player(player_pos, entity_manager);
        }
    }

    pub fn get_render_data(&self) -> Vec<Primitive> {
        self.enemies
            .iter()
            .map(|enemy| {
                let config = enemy.config();
                Primitive::new(config.primitive_type, enemy.position(), config.color)
                    .with_uniform_scale(config.visual_scale)
            })
            .collect()
    }

    pub fn enemies(&self) -> &[Enemy] {
        &self.enemies
    }

    pub fn enemies_mut(&mut self) -> &mut [Enemy] {
        &mut self.enemies
    }

    pub fn remove_enemy(&mut self, index: usize) -> bool {
        if index < self.enemies.len() {
            self.enemies.remove(index);
            true
        } else {
            false
        }
    }

    pub fn enemy_count(&self) -> usize {
        self.enemies.len()
    }

    /// Clean up dead enemies and return their entity IDs for destruction
    pub fn cleanup_dead_enemies(&mut self) -> Vec<crate::engine::entity::EntityId> {
        let mut destroyed_entities = Vec::new();

        let mut i = 0;
        while i < self.enemies.len() {
            if !self.enemies[i].is_alive() {
                destroyed_entities.push(self.enemies[i].entity_id());
                self.enemies.remove(i);
            } else {
                i += 1;
            }
        }

        destroyed_entities
    }

    /// Find and damage an enemy by entity ID
    pub fn damage_enemy(
        &mut self,
        entity_id: crate::engine::entity::EntityId,
        damage: f32,
    ) -> bool {
        self.damage_enemy_direct(entity_id, damage, entity_id)
    }

    /// Damage enemy directly without generating damage events (used by collision system)
    pub fn damage_enemy_direct(
        &mut self,
        entity_id: crate::engine::entity::EntityId,
        damage: f32,
        _source: crate::engine::entity::EntityId,
    ) -> bool {
        for enemy in &mut self.enemies {
            if enemy.entity_id() == entity_id {
                let old_health = enemy.health();
                enemy.take_damage(damage);

                // Check if enemy died
                if enemy.health() <= 0.0 && old_health > 0.0 {
                    // Generate score event immediately while we still have access to the enemy
                    let enemy_type = enemy.enemy_type().clone();
                    let enemy_pos = enemy.position();

                    self.event_queue.push(EventType::Score(
                        crate::engine::dispatcher::ScoreEvent::EnemyKilled {
                            enemy_id: entity_id,
                            enemy_type,
                            position: enemy_pos,
                        },
                    ));

                    // Generate death event
                    self.event_queue.push(EventType::Enemy(EnemyEvent::Die {
                        enemy_id: entity_id,
                    }));
                }

                return true;
            }
        }
        false
    }

    /// Damage enemy and generate damage event (used by event system)
    pub fn damage_enemy_with_event(
        &mut self,
        entity_id: crate::engine::entity::EntityId,
        damage: f32,
        source: crate::engine::entity::EntityId,
    ) -> bool {
        for enemy in &mut self.enemies {
            if enemy.entity_id() == entity_id {
                let old_health = enemy.health();
                enemy.take_damage(damage);

                // Generate damage event
                self.event_queue
                    .push(EventType::Enemy(EnemyEvent::TakeDamage {
                        enemy_id: entity_id,
                        amount: damage,
                        source,
                    }));

                // Check if enemy died
                if enemy.health() <= 0.0 && old_health > 0.0 {
                    // Generate death event
                    self.event_queue.push(EventType::Enemy(EnemyEvent::Die {
                        enemy_id: entity_id,
                    }));
                }

                return true;
            }
        }
        false
    }

    /// Mark an enemy for removal by setting health to 0 (used by collision system for instant kills)
    pub fn mark_enemy_for_removal(&mut self, entity_id: crate::engine::entity::EntityId) -> bool {
        for enemy in &mut self.enemies {
            if enemy.entity_id() == entity_id {
                enemy.take_damage(9999.0); // Mark as dead for cleanup
                return true;
            }
        }
        false
    }

    /// Get enemy position by entity ID
    pub fn get_enemy_position(&self, entity_id: crate::engine::entity::EntityId) -> Option<Vec3> {
        for enemy in &self.enemies {
            if enemy.entity_id() == entity_id {
                return Some(enemy.position());
            }
        }
        None
    }

    /// Get all enemy positions (for seeking weapons)
    pub fn get_all_enemy_positions(&self) -> Vec<Vec3> {
        self.enemies.iter().map(|enemy| enemy.position()).collect()
    }

    /// Spawn a new enemy near the player position (in a 3D sphere around player)
    pub fn spawn_enemy_near_player(
        &mut self,
        player_pos: Vec3,
        entity_manager: &mut crate::engine::entity::EntityManager,
    ) {
        use rand::Rng;

        // Random distance from player (5-100 units away)
        let distance = rand::rng().random_range(5.0..=100.0);

        // Generate random point on sphere using spherical coordinates
        // Uniform distribution on sphere surface
        let theta = rand::rng().random_range(0.0..std::f32::consts::TAU); // Azimuth angle (0 to 2π)
        let phi = rand::rng().random_range(0.0..std::f32::consts::PI); // Polar angle (0 to π)

        // Convert spherical to cartesian coordinates
        let sin_phi = phi.sin();
        let offset_x = sin_phi * theta.cos() * distance;
        let offset_y = phi.cos() * distance; // Y is "up" in our coordinate system
        let offset_z = sin_phi * theta.sin() * distance;

        let spawn_pos = Vec3::new(
            player_pos.x + offset_x,
            player_pos.y + offset_y,
            player_pos.z + offset_z,
        );

        // Initial velocity toward player with some randomness
        let to_player = player_pos - spawn_pos;
        let to_player_normalized = if to_player.magnitude() > 0.1 {
            to_player.normalize()
        } else {
            Vec3::new(1.0, 0.0, 0.0) // Default direction if spawned too close
        };

        // Base velocity toward player
        let base_speed = rand::rng().random_range(2.0..=5.0);
        let mut spawn_vel = to_player_normalized * base_speed;

        // Add some randomness to prevent perfectly straight-line movement
        spawn_vel.x += rand::rng().random_range(-1.0..1.0);
        spawn_vel.y += rand::rng().random_range(-1.0..1.0);
        spawn_vel.z += rand::rng().random_range(-1.0..1.0);

        // Create the enemy entity and add to manager
        let enemy_entity = entity_manager.create_entity(crate::engine::entity::EntityType::Enemy);
        self.enemies.push(Enemy::new(
            enemy_entity,
            spawn_pos,
            spawn_vel,
            EnemyType::random(),
        ));
    }

    /// Get and clear enemy events
    pub fn drain_events(&mut self) -> Vec<EventType> {
        self.event_queue.drain(..).collect()
    }

    /// Spawn a fractal splitter boss at a specific position
    pub fn spawn_splitter(
        &mut self,
        position: Vec3,
        max_generation: u8,
        entity_manager: &mut crate::engine::entity::EntityManager,
    ) {
        let enemy_entity = entity_manager.create_entity(crate::engine::entity::EntityType::Enemy);
        self.enemies.push(Enemy::new(
            enemy_entity,
            position,
            Vec3::zeros(), // Start stationary
            EnemyType::Splitter {
                current_generation: 0,
                max_generation,
            },
        ));
    }

    /// Spawn a cannibal enemy at a specific position
    pub fn spawn_cannibal(
        &mut self,
        position: Vec3,
        entity_manager: &mut crate::engine::entity::EntityManager,
    ) {
        let enemy_entity = entity_manager.create_entity(crate::engine::entity::EntityType::Enemy);
        self.enemies.push(Enemy::new(
            enemy_entity,
            position,
            Vec3::zeros(), // Start stationary
            EnemyType::Cannibal {
                meals_consumed: 0,
                eating_cooldown: 0.0,
            },
        ));
    }

    /// Spawn a ShieldOrb boss (core with orbiting fractal shields)
    pub fn spawn_shield_orb_boss(
        &mut self,
        position: Vec3,
        initial_shield_count: usize,
        max_shield_generation: u8,
        orbit_radius: f32,
        entity_manager: &mut crate::engine::entity::EntityManager,
    ) {
        // Create the core
        let core_entity = entity_manager.create_entity(crate::engine::entity::EntityType::Enemy);
        let mut shield_ids = Vec::new();

        // Distribute shields evenly on a sphere using Fibonacci sphere algorithm
        // This gives a more uniform distribution than just dividing angles
        let golden_ratio = (1.0 + 5.0_f32.sqrt()) / 2.0;

        for i in 0..initial_shield_count {
            // Fibonacci sphere distribution
            let idx = i as f32;
            let count = initial_shield_count as f32;

            // Azimuth angle (theta) - horizontal rotation
            let orbit_angle = std::f32::consts::TAU * idx / golden_ratio;

            // Polar angle (phi) - vertical angle from pole
            // Maps from 0 to PI (top to bottom of sphere)
            // z ranges from ~1 (top) to ~-1 (bottom)
            let z = 1.0 - (2.0 * (idx + 0.5)) / count;
            let orbit_inclination = z.acos();

            let shield_entity =
                entity_manager.create_entity(crate::engine::entity::EntityType::Enemy);
            shield_ids.push(shield_entity);

            self.enemies.push(Enemy::new(
                shield_entity,
                position, // Start at core position, will be updated by orbital movement
                Vec3::zeros(),
                EnemyType::Shield {
                    current_generation: 0,
                    max_generation: max_shield_generation,
                    core_id: core_entity,
                    orbit_angle,
                    orbit_inclination,
                    orbit_radius,
                },
            ));
        }

        // Now create the core with the shield IDs
        self.enemies.push(Enemy::new(
            core_entity,
            position,
            Vec3::zeros(),
            EnemyType::ShieldOrbCore {
                shield_ids,
                is_vulnerable: false, // Start invulnerable (protected by shields)
            },
        ));
    }

    /// Handle enemy events from dispatcher
    pub fn handle_event(&mut self, event: crate::engine::dispatcher::EnemyEvent) {
        use crate::engine::dispatcher::EnemyEvent;
        match event {
            EnemyEvent::TakeDamage {
                enemy_id,
                amount,
                source,
            } => {
                self.damage_enemy_with_event(enemy_id, amount, source);
            }
            EnemyEvent::Die { enemy_id } => {
                // Find enemy and spawn death particles before marking as dead
                for enemy in &mut self.enemies {
                    if enemy.entity_id() == enemy_id {
                        let enemy_pos = enemy.position();
                        let enemy_type = enemy.enemy_type();

                        // Spawn explosion particles at enemy death location
                        use crate::engine::dispatcher::{EventType, GraphicsEvent};
                        use crate::graphics::Color;

                        self.event_queue
                            .push(EventType::Graphics(GraphicsEvent::SpawnParticles {
                                position: enemy_pos,
                                velocity: crate::engine::Vec3::new(0.0, 0.0, 0.0),
                                count: 100,
                                lifetime: 2.0,
                                color: Color::GREEN,
                            }));

                        // Check if this is a Splitter that should split
                        if let EnemyType::Splitter {
                            current_generation,
                            max_generation,
                        } = enemy_type
                        {
                            if *current_generation < *max_generation {
                                // Queue split event to spawn 2 child splitters
                                self.event_queue.push(EventType::Enemy(EnemyEvent::Split {
                                    parent_id: enemy_id,
                                    position: enemy_pos,
                                    current_generation: *current_generation,
                                    max_generation: *max_generation,
                                }));
                            }
                        }

                        // Score event is now generated immediately upon death in damage_enemy_direct

                        enemy.take_damage(9999.0); // Ensure death
                        break;
                    }
                }
            }
            EnemyEvent::Spawn {
                position,
                enemy_type: _,
            } => {
                // Note: Runtime enemy spawning requires access to EntityManager
                // For now, we create a temporary ID that won't collide with existing systems
                // This should be properly integrated with EntityManager in the future
                let temp_id = (self.enemies.len() as u32).wrapping_add(50000); // Large offset to avoid collisions
                let enemy_entity = crate::engine::entity::EntityId(temp_id);
                self.enemies.push(Enemy::new(
                    enemy_entity,
                    position,
                    Vec3::zeros(),
                    EnemyType::Drone,
                ));
            }
            EnemyEvent::Split {
                parent_id: _,
                position,
                current_generation,
                max_generation,
            } => {
                // Spawn 2 child splitters at the next generation
                use rand::Rng;

                let next_generation = current_generation + 1;

                // Create 2 children offset from parent position with outward velocity
                for i in 0..2 {
                    let angle = (i as f32 / 2.0) * std::f32::consts::TAU + rand::rng().random_range(0.0..0.5);
                    let offset_distance = 2.0; // Spawn slightly away from parent

                    let offset = Vec3::new(
                        angle.cos() * offset_distance,
                        rand::rng().random_range(-1.0..1.0), // Random vertical offset
                        angle.sin() * offset_distance,
                    );

                    // Velocity pointing outward from split point
                    let outward_speed = 5.0;
                    let vel = Vec3::new(
                        angle.cos() * outward_speed,
                        rand::rng().random_range(-2.0..2.0),
                        angle.sin() * outward_speed,
                    );

                    // Create temporary entity ID (should use EntityManager in production)
                    let temp_id = (self.enemies.len() as u32).wrapping_add(50000);
                    let enemy_entity = crate::engine::entity::EntityId(temp_id);

                    self.enemies.push(Enemy::new(
                        enemy_entity,
                        position + offset,
                        vel,
                        EnemyType::Splitter {
                            current_generation: next_generation,
                            max_generation,
                        },
                    ));
                }

                // Spawn particles for the split effect
                self.event_queue
                    .push(EventType::Graphics(GraphicsEvent::SpawnParticles {
                        position,
                        velocity: Vec3::new(0.0, 0.0, 0.0),
                        count: 50, // Burst of particles at split point
                        lifetime: 1.5,
                        color: Color::ORANGE, // Bright split effect
                    }));
            }
        }
    }
}
