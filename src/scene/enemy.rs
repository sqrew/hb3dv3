use crate::engine::entity::{EntityId, EntityType};
use crate::engine::{CollisionMask, Vec3};
use crate::graphics::{Color, Primitive, PrimitiveType};

pub struct Enemy {
    entity_id: EntityId,
    pos: Vec3,
    vel: Vec3,
    health: f32,
    collision_radius: f32,
    collision_mask: CollisionMask,
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
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.pos += self.vel * dt;

        // Simple AI: bounce off boundaries
        if self.pos.x.abs() > 15.0 {
            self.vel.x = -self.vel.x;
        }
        if self.pos.z.abs() > 15.0 {
            self.vel.z = -self.vel.z;
        }

        // Keep enemies within bounds
        self.pos.x = self.pos.x.clamp(-20.0, 20.0);
        self.pos.z = self.pos.z.clamp(-20.0, 20.0);
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
        if self.health <= 0.0 {
            println!("💀 Enemy {} destroyed by damage!", self.entity_id.id());
        }
    }
}

pub struct EnemyManager {
    enemies: Vec<Enemy>,
}

impl EnemyManager {
    pub fn new() -> Self {
        Self {
            enemies: Vec::new(),
        }
    }

    pub fn spawn_initial_enemies(
        &mut self,
        entity_manager: &mut crate::engine::entity::EntityManager,
    ) {
        // Spawn a few enemies in interesting positions
        for i in 0..500 {
            let angle = (i as f32 / 5.0) * std::f32::consts::TAU;
            let radius = 8.0;
            let pos = Vec3::new(
                angle.cos() * radius,
                (i as f32) * 0.5 - 1.0,
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

    pub fn remove_enemy(&mut self, index: usize) -> bool {
        if index < self.enemies.len() {
            self.enemies.remove(index);
            println!("💀 Enemy {} destroyed!", index);
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
        for enemy in &mut self.enemies {
            if enemy.entity_id() == entity_id {
                enemy.take_damage(damage);
                return true;
            }
        }
        false
    }
}
