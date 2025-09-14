use crate::engine::dispatcher::{EnemyEvent, EventType};
use crate::engine::entity::{EntityId, EntityType};
use crate::engine::{CollisionMask, Vec3};
use crate::graphics::{Color, Primitive, PrimitiveType};
use crate::scene::GravityAffected;

pub struct Enemy {
    entity_id: EntityId,
    pos: Vec3,
    vel: Vec3,
    health: f32,
    collision_radius: f32,
    collision_mask: CollisionMask,
    mass: f32,
    applied_force: Vec3,
}

impl Enemy {
    pub fn new(entity_id: EntityId, pos: Vec3, vel: Vec3, health: f32) -> Self {
        Enemy {
            entity_id,
            pos,
            vel,
            health,
            collision_radius: 0.6,
            collision_mask: CollisionMask::from(EntityType::Enemy),
            mass: 25.0, // Enemy mass in kg
            applied_force: Vec3::zeros(),
        }
    }

    pub fn update(&mut self, dt: f32) {
        // Apply gravitational forces (F = ma, so a = F/m)
        let gravity_acceleration = self.applied_force / self.mass;

        // Update velocity with both AI movement and gravity
        self.vel += gravity_acceleration * dt;

        // Update position
        self.pos += self.vel * dt;

        // Simple AI: bounce off boundaries (but allow gravity to override)
        if self.pos.x.abs() > 15.0 && self.applied_force.magnitude() < 10.0 {
            self.vel.x = -self.vel.x;
        }
        if self.pos.z.abs() > 15.0 && self.applied_force.magnitude() < 10.0 {
            self.vel.z = -self.vel.z;
        }

        // Reset applied force for next frame
        self.applied_force = Vec3::zeros();
    }

    pub fn position(&self) -> Vec3 {
        self.pos
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0.0
    }

    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    pub fn collision_radius(&self) -> f32 {
        self.collision_radius
    }

    pub fn collision_mask(&self) -> CollisionMask {
        self.collision_mask
    }

    pub fn take_damage(&mut self, damage: f32) {
        self.health -= damage;
    }
}

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
            spawn_interval: 10.0, // Spawn every 2 seconds
        }
    }

    pub fn spawn_initial_enemies(
        &mut self,
        entity_manager: &mut crate::engine::entity::EntityManager,
    ) {
        use rand::Rng;

        // 1. Orbital Ring Formation (classic spiral around center)
        for i in 0..200 {
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
            self.enemies.push(Enemy::new(enemy_entity, pos, vel, 50.0));
        }

        // 2. Spherical Shell Formation (hollow sphere of enemies)
        for i in 0..100 {
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
            self.enemies.push(Enemy::new(enemy_entity, pos, vel, 50.0));
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
                self.enemies.push(Enemy::new(enemy_entity, pos, vel, 50.0));
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
                self.enemies.push(Enemy::new(enemy_entity, pos, vel, 50.0));
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
            self.enemies.push(Enemy::new(enemy_entity, pos, vel, 50.0));
        }
    }

    pub fn update(
        &mut self,
        dt: f32,
        player_pos: Vec3,
        entity_manager: &mut crate::engine::entity::EntityManager,
    ) {
        // Update existing enemies
        for enemy in self.enemies.iter_mut() {
            enemy.update(dt);
        }

        // Remove dead enemies
        self.enemies.retain(|e| e.is_alive());

        // Timer-based enemy spawning
        self.spawn_timer += dt;
        if self.spawn_timer >= self.spawn_interval {
            self.spawn_timer = 0.0; // Reset timer
            self.spawn_enemy_near_player(player_pos, entity_manager);
        }
    }

    pub fn get_render_data(&self) -> Vec<Primitive> {
        self.enemies
            .iter()
            .map(|enemy| {
                Primitive::new(
                    PrimitiveType::Cube,
                    enemy.pos,
                    Color::new(0.8, 0.1, 0.1, 1.0), // Red enemies
                )
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
        source: crate::engine::entity::EntityId,
    ) -> bool {
        for enemy in &mut self.enemies {
            if enemy.entity_id() == entity_id {
                let old_health = enemy.health;
                enemy.take_damage(damage);

                // Check if enemy died
                if enemy.health <= 0.0 && old_health > 0.0 {
                    // Generate death event
                    self.event_queue.push(EventType::Enemy(EnemyEvent::Die {
                        enemy_id: entity_id,
                    }));
                    println!(
                        "💀 Enemy {} died from {} damage (source: {})!",
                        entity_id.0, damage, source.0
                    );
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
                let old_health = enemy.health;
                enemy.take_damage(damage);

                // Generate damage event
                self.event_queue
                    .push(EventType::Enemy(EnemyEvent::TakeDamage {
                        enemy_id: entity_id,
                        amount: damage,
                        source,
                    }));

                // Check if enemy died
                if enemy.health <= 0.0 && old_health > 0.0 {
                    // Generate death event
                    self.event_queue.push(EventType::Enemy(EnemyEvent::Die {
                        enemy_id: entity_id,
                    }));
                    println!(
                        "💀 Enemy {} died from {} damage (source: {})!",
                        entity_id.0, damage, source.0
                    );
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
                enemy.health = 0.0; // Mark as dead for cleanup
                println!("☠️ Enemy {} marked for removal", entity_id.0);
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

    /// Spawn a new enemy near the player position (in a 3D sphere around player)
    pub fn spawn_enemy_near_player(
        &mut self,
        player_pos: Vec3,
        entity_manager: &mut crate::engine::entity::EntityManager,
    ) {
        use rand::Rng;

        // Random distance from player (5-15 units away)
        let distance = rand::rng().random_range(5.0..=15.0);

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

        // Random initial velocity (slight movement in a random 3D direction)
        let vel_magnitude = rand::rng().random_range(0.5..=2.0);
        let vel_theta = rand::rng().random_range(0.0..std::f32::consts::TAU);
        let vel_phi = rand::rng().random_range(0.0..std::f32::consts::PI);
        let vel_sin_phi = vel_phi.sin();

        let spawn_vel = Vec3::new(
            vel_sin_phi * vel_theta.cos() * vel_magnitude,
            vel_phi.cos() * vel_magnitude,
            vel_sin_phi * vel_theta.sin() * vel_magnitude,
        );

        // Create the enemy entity and add to manager
        let enemy_entity = entity_manager.create_entity(crate::engine::entity::EntityType::Enemy);
        self.enemies
            .push(Enemy::new(enemy_entity, spawn_pos, spawn_vel, 50.0));
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
                // Mark enemy as dead (damage_enemy already handles this)
                for enemy in &mut self.enemies {
                    if enemy.entity_id() == enemy_id {
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
                self.enemies
                    .push(Enemy::new(enemy_entity, position, Vec3::zeros(), 50.0));
                println!(
                    "🔴 Spawned enemy {} at {:?} (temp ID - needs EntityManager integration)",
                    enemy_entity.0, position
                );
            }
        }
    }
}

impl GravityAffected for Enemy {
    fn position(&self) -> Vec3 {
        self.pos
    }

    fn mass(&self) -> f32 {
        self.mass
    }

    fn apply_force(&mut self, force: Vec3) {
        self.applied_force += force;
    }
}
