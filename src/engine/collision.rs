use crate::engine::entity::EntityType;

/// Collision mask system for efficient collision filtering
/// Each entity type has a mask defining what other entity types it can collide with
#[derive(Debug, Clone, Copy)]
pub struct CollisionMask(pub u32);

impl CollisionMask {
    pub const NONE: CollisionMask = CollisionMask(0);
    pub const ALL: CollisionMask = CollisionMask(0xFFFFFFFF);

    /// Check if this collision mask allows collision with the given entity type
    pub fn collides_with(&self, entity_type: EntityType) -> bool {
        let type_bit = 1 << (entity_type as u32);
        (self.0 & type_bit) != 0
    }

    /// Create a collision mask that collides with specific entity types
    pub fn from_types(types: &[EntityType]) -> Self {
        let mut mask = 0u32;
        for &entity_type in types {
            mask |= 1 << (entity_type as u32);
        }
        CollisionMask(mask)
    }

    /// Add collision with an entity type
    pub fn with_type(mut self, entity_type: EntityType) -> Self {
        let type_bit = 1 << (entity_type as u32);
        self.0 |= type_bit;
        self
    }

    /// Remove collision with an entity type
    pub fn without_type(mut self, entity_type: EntityType) -> Self {
        let type_bit = 1 << (entity_type as u32);
        self.0 &= !type_bit;
        self
    }
}

/// Default collision masks for each entity type
impl CollisionMask {
    /// Player collides with enemies only
    pub const PLAYER: CollisionMask = CollisionMask(1 << EntityType::Enemy as u32);

    /// Enemies collide with player and player bullets
    pub const ENEMY: CollisionMask =
        CollisionMask((1 << EntityType::Player as u32) | (1 << EntityType::PlayerBullet as u32));

    /// Player bullets collide with enemies only
    pub const PLAYER_BULLET: CollisionMask = CollisionMask(1 << EntityType::Enemy as u32);

    /// Enemy bullets collide with player only (when implemented)
    pub const ENEMY_BULLET: CollisionMask = CollisionMask(1 << EntityType::Player as u32);
}

impl Default for CollisionMask {
    fn default() -> Self {
        CollisionMask::NONE
    }
}

impl From<EntityType> for CollisionMask {
    fn from(entity_type: EntityType) -> Self {
        match entity_type {
            EntityType::Player => CollisionMask::PLAYER,
            EntityType::Enemy => CollisionMask::ENEMY,
            EntityType::PlayerBullet => CollisionMask::PLAYER_BULLET,
            EntityType::EnemyBullet => CollisionMask::ENEMY_BULLET,
        }
    }
}

/// Dedicated collision manager that processes GPU collision results and generates events
pub struct CollisionManager {
    /// Events generated from collision processing
    event_queue: Vec<crate::engine::dispatcher::EventType>,
}

impl CollisionManager {
    pub fn new() -> Self {
        Self {
            event_queue: Vec::new(),
        }
    }

    /// Process raw collision pairs from GPU and generate appropriate events
    pub fn process_collision_pairs(
        &mut self,
        collision_pairs: &[(u32, u32)],
        bullet_manager: &crate::scene::bullet::BulletManager,
        enemy_manager: &crate::scene::enemy::EnemyManager,
        entity_manager: &crate::engine::entity::EntityManager,
    ) {
        // Optimization: Batch lookup all entity types at once instead of per-collision
        let mut entity_types = std::collections::HashMap::new();
        for &(entity_a_id, entity_b_id) in collision_pairs {
            let entity_a = crate::engine::entity::EntityId(entity_a_id);
            let entity_b = crate::engine::entity::EntityId(entity_b_id);

            // Only lookup if we haven't already cached this entity type
            if !entity_types.contains_key(&entity_a) {
                entity_types.insert(entity_a, entity_manager.get_type(entity_a));
            }
            if !entity_types.contains_key(&entity_b) {
                entity_types.insert(entity_b, entity_manager.get_type(entity_b));
            }
        }

        // Process collisions using cached entity types
        for &(entity_a_id, entity_b_id) in collision_pairs {
            let entity_a = crate::engine::entity::EntityId(entity_a_id);
            let entity_b = crate::engine::entity::EntityId(entity_b_id);

            // Fast lookup from cache
            let type_a = entity_types.get(&entity_a).copied().flatten();
            let type_b = entity_types.get(&entity_b).copied().flatten();

            match (type_a, type_b) {
                // PlayerBullet hits Enemy
                (
                    Some(crate::engine::entity::EntityType::PlayerBullet),
                    Some(crate::engine::entity::EntityType::Enemy),
                ) => {
                    if let Some(bullet_damage) = bullet_manager.get_bullet_damage(entity_a) {
                        if let Some(enemy_pos) = enemy_manager.get_enemy_position(entity_b) {
                            self.event_queue
                                .push(crate::engine::dispatcher::EventType::Collision(
                                    crate::engine::dispatcher::CollisionEvent::BulletHitEnemy {
                                        bullet_id: entity_a,
                                        enemy_id: entity_b,
                                        damage: bullet_damage,
                                        impact_point: enemy_pos,
                                    },
                                ));
                            println!("PlayerBullet {} hit Enemy {}", entity_a.0, entity_b.0);
                        }
                    }
                }
                // Enemy hits PlayerBullet (reverse)
                (
                    Some(crate::engine::entity::EntityType::Enemy),
                    Some(crate::engine::entity::EntityType::PlayerBullet),
                ) => {
                    if let Some(bullet_damage) = bullet_manager.get_bullet_damage(entity_b) {
                        if let Some(enemy_pos) = enemy_manager.get_enemy_position(entity_a) {
                            self.event_queue
                                .push(crate::engine::dispatcher::EventType::Collision(
                                    crate::engine::dispatcher::CollisionEvent::BulletHitEnemy {
                                        bullet_id: entity_b,
                                        enemy_id: entity_a,
                                        damage: bullet_damage,
                                        impact_point: enemy_pos,
                                    },
                                ));
                            println!("PlayerBullet {} hit Enemy {}", entity_b.0, entity_a.0);
                        }
                    }
                }
                // Enemy hits Player
                (
                    Some(crate::engine::entity::EntityType::Enemy),
                    Some(crate::engine::entity::EntityType::Player),
                ) => {
                    self.event_queue
                        .push(crate::engine::dispatcher::EventType::Collision(
                            crate::engine::dispatcher::CollisionEvent::EnemyHitPlayer {
                                enemy_id: entity_a,
                                player_id: entity_b,
                                damage: 10.0, // Default enemy contact damage
                            },
                        ));
                    println!("Enemy {} hit Player {}", entity_a.0, entity_b.0);
                }
                // Player hits Enemy (reverse)
                (
                    Some(crate::engine::entity::EntityType::Player),
                    Some(crate::engine::entity::EntityType::Enemy),
                ) => {
                    self.event_queue
                        .push(crate::engine::dispatcher::EventType::Collision(
                            crate::engine::dispatcher::CollisionEvent::EnemyHitPlayer {
                                enemy_id: entity_b,
                                player_id: entity_a,
                                damage: 10.0, // Default enemy contact damage
                            },
                        ));
                    println!("Enemy {} hit Player {}", entity_b.0, entity_a.0);
                }
                _ => {
                    // Unhandled collision type or invalid entities
                    if type_a.is_none() || type_b.is_none() {
                        println!(
                            "⚠️ Warning: Collision between invalid entities {} and {}",
                            entity_a.0, entity_b.0
                        );
                    }
                }
            }
        }
    }

    /// Get and clear all collision events
    pub fn drain_events(&mut self) -> Vec<crate::engine::dispatcher::EventType> {
        self.event_queue.drain(..).collect()
    }

    /// Check if there are pending events
    pub fn has_events(&self) -> bool {
        !self.event_queue.is_empty()
    }

    /// Get event count without draining
    pub fn event_count(&self) -> usize {
        self.event_queue.len()
    }
}

impl Default for CollisionManager {
    fn default() -> Self {
        Self::new()
    }
}
