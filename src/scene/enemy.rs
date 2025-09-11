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

        // Keep enemies within reasonable bounds (but allow gravity to pull them)
        self.pos.x = self.pos.x.clamp(-50.0, 50.0);
        self.pos.z = self.pos.z.clamp(-50.0, 50.0);
        
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
}

impl EnemyManager {
    pub fn new() -> Self {
        Self {
            enemies: Vec::new(),
            event_queue: Vec::new(),
        }
    }

    pub fn spawn_initial_enemies(
        &mut self,
        entity_manager: &mut crate::engine::entity::EntityManager,
    ) {
        // Spawn a few enemies in interesting positions
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
            self.enemies.push(Enemy::new(enemy_entity, pos, vel, 50.0));
        }
    }

    pub fn update(&mut self, dt: f32) {
        for enemy in self.enemies.iter_mut() {
            enemy.update(dt);
        }

        // Remove dead enemies
        self.enemies.retain(|e| e.is_alive());
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

    /// Get enemy position by entity ID
    pub fn get_enemy_position(&self, entity_id: crate::engine::entity::EntityId) -> Option<Vec3> {
        for enemy in &self.enemies {
            if enemy.entity_id() == entity_id {
                return Some(enemy.position());
            }
        }
        None
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
