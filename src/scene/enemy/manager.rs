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
        for i in 0..500 {
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
        for enemy in self.enemies.iter_mut() {
            enemy.update(dt, player_pos);
        }

        // Check for dead enemies and generate death events with position before removing them
        let mut enemies_to_remove = Vec::new();
        for enemy in &self.enemies {
            if !enemy.is_alive() {
                enemies_to_remove.push((enemy.entity_id(), enemy.position()));
            }
        }

        // Generate death events with position for dead enemies
        for (enemy_id, position) in enemies_to_remove {
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
        }

        // Now remove the dead enemies
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
                    let enemy_type = enemy.enemy_type();
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
                        let _enemy_type = enemy.enemy_type();

                        // Spawn explosion particles at enemy death location
                        use crate::engine::dispatcher::{EventType, GraphicsEvent};
                        use crate::graphics::Color;

                        self.event_queue
                            .push(EventType::Graphics(GraphicsEvent::SpawnParticles {
                                position: enemy_pos,
                                velocity: crate::engine::Vec3::new(0.0, 0.0, 0.0), // Upward explosion
                                count: 100,          // Big explosion for enemy death
                                lifetime: 2.0,       // Longer lasting death particles
                                color: Color::GREEN, // Orange explosion color: use enemy.color eventually once its coded in
                            }));

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
        }
    }
}
