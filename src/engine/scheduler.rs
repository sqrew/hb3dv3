use crate::scene::{BulletManager, EnemyManager, PlayerManager};
use crate::input::InputManager;
use crate::graphics::{Primitive, Vec3};
use crate::engine::entity::{EntityManager, EntityType, EntityLookup};

pub struct Scheduler {
    entity_manager: EntityManager,
    player: PlayerManager,
    enemies: EnemyManager,
    bullets: BulletManager,
}

impl Scheduler {
    pub fn new() -> Self {
        let mut entity_manager = EntityManager::new();
        
        // Create player entity
        let player_entity = entity_manager.create_entity(EntityType::Player);
        
        let mut enemies = EnemyManager::new();
        enemies.spawn_initial_enemies(&mut entity_manager);
        
        Scheduler {
            entity_manager,
            player: PlayerManager::new(player_entity),
            enemies,
            bullets: BulletManager::new(),
        }
    }

    pub fn preupdate(&mut self) {
        // Pre-update phase - prepare for frame
    }
    
    pub fn update(&mut self, delta_time: f32, input: &InputManager, camera_forward: Vec3, camera_right: Vec3, camera_up: Vec3) {
        // Update player movement
        if let Some(bullet_requests) = self.player.update(delta_time, input, camera_forward, camera_right, camera_up) {
            // Spawn bullets from player weapon
            for request in bullet_requests.iter() {
                let bullet_entity = self.entity_manager.create_entity(EntityType::PlayerBullet);
                self.bullets.spawn_bullet(
                    bullet_entity,
                    request.position,
                    request.speed,
                    request.direction,
                    request.lifetime,
                    request.damage,
                );
            }
        }
        
        self.enemies.update(delta_time);
        self.bullets.update(delta_time);
        
    }
    
    pub fn postupdate(&mut self) {
        // Post-update phase - clean up dead entities efficiently
        
        // Each manager handles its own cleanup and returns destroyed entity IDs
        let destroyed_bullets = self.bullets.cleanup_dead_bullets();
        let destroyed_enemies = self.enemies.cleanup_dead_enemies();
        
        // Destroy entities from the entity manager
        for entity_id in destroyed_bullets.into_iter().chain(destroyed_enemies.into_iter()) {
            self.entity_manager.destroy_entity(entity_id);
        }
    }
    
    pub fn prerender(&self) {
        // Pre-render phase - prepare render data
    }
    
    pub fn get_render_data(&self) -> Vec<Primitive> {
        let mut primitives = Vec::new();
        
        // Get render data from all managers
        primitives.extend(self.player.get_render_data());
        primitives.extend(self.enemies.get_render_data());
        primitives.extend(self.bullets.get_render_data());
        
        primitives
    }
    
    pub fn postrender(&self) {
        // Post-render phase - cleanup after rendering
    }
    
    pub fn get_player_position(&self) -> Vec3 {
        self.player.player().position()
    }
    
    /// Get access to the entity manager for queries and lookups
    pub fn entity_manager(&self) -> &EntityManager {
        &self.entity_manager
    }
    
    /// Create an EntityLookup instance for targeting and position queries
    pub fn create_entity_lookup(&self) -> EntityLookup {
        EntityLookup::new(
            &self.entity_manager,
            &self.player,
            &self.enemies,
            &self.bullets,
        )
    }
    
    /// Get access to the player manager
    pub fn player(&self) -> &PlayerManager {
        &self.player
    }
    
    /// Get access to the enemy manager
    pub fn enemies(&self) -> &EnemyManager {
        &self.enemies
    }
    
    /// Get access to the bullet manager
    pub fn bullets(&self) -> &BulletManager {
        &self.bullets
    }
    
    /// Get mutable access to the enemy manager
    pub fn enemies_mut(&mut self) -> &mut EnemyManager {
        &mut self.enemies
    }
    
    /// Get mutable access to the bullet manager
    pub fn bullets_mut(&mut self) -> &mut BulletManager {
        &mut self.bullets
    }
    
    /// Process collision pairs from GPU collision detection
    pub fn process_collision_pairs(&mut self, collision_pairs: &[(u32, u32)]) {
        for &(entity_a_id, entity_b_id) in collision_pairs {
            // Check if it's a bullet hitting an enemy
            if let Some(bullet_damage) = self.bullets.get_bullet_damage(crate::engine::entity::EntityId(entity_a_id)) {
                if self.enemies.damage_enemy(crate::engine::entity::EntityId(entity_b_id), bullet_damage) {
                    // Bullet hit enemy - remove bullet
                    self.bullets.remove_bullet_by_entity_id(crate::engine::entity::EntityId(entity_a_id));
                    println!("💥 Collision! Bullet {} hit Enemy {}", entity_a_id, entity_b_id);
                }
            }
            // Check reverse (enemy hit by bullet)
            else if let Some(bullet_damage) = self.bullets.get_bullet_damage(crate::engine::entity::EntityId(entity_b_id)) {
                if self.enemies.damage_enemy(crate::engine::entity::EntityId(entity_a_id), bullet_damage) {
                    // Bullet hit enemy - remove bullet
                    self.bullets.remove_bullet_by_entity_id(crate::engine::entity::EntityId(entity_b_id));
                    println!("💥 Collision! Bullet {} hit Enemy {}", entity_b_id, entity_a_id);
                }
            }
        }
    }
    
    /// Simple CPU-based collision detection (temporary fallback)
    pub fn check_collisions_cpu(&mut self) {
        let bullets = self.bullets.bullets();
        let enemies = self.enemies.enemies();
        
        let mut bullets_to_remove = Vec::new();
        let mut enemies_to_damage = Vec::new();
        
        // Check each bullet against each enemy
        for bullet in bullets {
            for enemy in enemies {
                let distance_sq = (bullet.position() - enemy.position()).magnitude_squared();
                let collision_distance = bullet.collision_radius() + enemy.collision_radius();
                
                if distance_sq <= collision_distance * collision_distance {
                    // Collision detected!
                    bullets_to_remove.push(bullet.entity_id());
                    enemies_to_damage.push((enemy.entity_id(), bullet.damage()));
                    println!("💥 CPU Collision! Bullet {} hit Enemy {}", bullet.entity_id().id(), enemy.entity_id().id());
                    break; // One bullet can only hit one enemy
                }
            }
        }
        
        // Apply damage and remove bullets
        for (enemy_id, damage) in enemies_to_damage {
            self.enemies.damage_enemy(enemy_id, damage);
        }
        
        for bullet_id in bullets_to_remove {
            self.bullets.remove_bullet_by_entity_id(bullet_id);
        }
    }
    
}
