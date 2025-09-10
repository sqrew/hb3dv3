use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(pub u32);

impl EntityId {
    pub fn id(&self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityType {
    Player,
    Enemy,
    PlayerBullet,
    EnemyBullet,
}

pub struct EntityManager {
    next_id: u32,
    active_entities: HashSet<EntityId>,
    entity_types: std::collections::HashMap<EntityId, EntityType>,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            next_id: 1, // Start at 1, reserve 0 for invalid/null entity
            active_entities: HashSet::new(),
            entity_types: std::collections::HashMap::new(),
        }
    }

    /// Creates a new entity ID and registers it as active
    pub fn create_entity(&mut self, entity_type: EntityType) -> EntityId {
        // FIXED: Prevent ID collision after overflow by finding next available ID
        loop {
            let id = EntityId(self.next_id);

            // Handle overflow gracefully
            self.next_id = self.next_id.wrapping_add(1);
            if self.next_id == 0 {
                // If we've wrapped around, skip 0 as it might be used as a null ID
                self.next_id = 1;
            }

            // Ensure ID is not already in use (extremely rare after wraparound)
            if !self.active_entities.contains(&id) {
                self.active_entities.insert(id);
                self.entity_types.insert(id, entity_type);
                return id;
            }
            // If ID collision (very rare), try next ID
        }
    }

    /// Marks an entity as destroyed and removes it from active tracking
    pub fn destroy_entity(&mut self, id: EntityId) -> bool {
        self.entity_types.remove(&id);
        self.active_entities.remove(&id)
    }

    /// Checks if an entity is currently active
    pub fn is_active(&self, id: EntityId) -> bool {
        self.active_entities.contains(&id)
    }

    /// Alias for is_active - checks if an entity exists and is active
    pub fn exists(&self, id: EntityId) -> bool {
        self.is_active(id)
    }

    /// Gets the type of an entity
    pub fn get_type(&self, id: EntityId) -> Option<EntityType> {
        self.entity_types.get(&id).copied()
    }

    /// Gets all active entities of a specific type
    pub fn get_entities_of_type(&self, entity_type: EntityType) -> Vec<EntityId> {
        self.entity_types
            .iter()
            .filter(|&(_, &t)| t == entity_type)
            .map(|(&id, _)| id)
            .collect()
    }

    /// Gets the total number of active entities
    pub fn active_count(&self) -> usize {
        self.active_entities.len()
    }

    /// Gets the total number of entities ever created (including destroyed ones)
    pub fn total_created(&self) -> u32 {
        self.next_id - 1
    }

    /// Returns an iterator over all active entity IDs
    pub fn active_entities(&self) -> impl Iterator<Item = &EntityId> {
        self.active_entities.iter()
    }

    /// Gets statistics about entity types
    pub fn get_type_counts(&self) -> std::collections::HashMap<EntityType, usize> {
        let mut counts = std::collections::HashMap::new();
        for &entity_type in self.entity_types.values() {
            *counts.entry(entity_type).or_insert(0) += 1;
        }
        counts
    }
}

/// EntityManager extension for targeting and position lookups
/// This requires access to the actual game managers, so it's implemented separately
pub struct EntityLookup<'a> {
    entity_manager: &'a EntityManager,
    player_manager: &'a crate::scene::PlayerManager,
    enemy_manager: &'a crate::scene::EnemyManager,
    bullet_manager: &'a crate::scene::BulletManager,
}

impl<'a> EntityLookup<'a> {
    pub fn new(
        entity_manager: &'a EntityManager,
        player_manager: &'a crate::scene::PlayerManager,
        enemy_manager: &'a crate::scene::EnemyManager,
        bullet_manager: &'a crate::scene::BulletManager,
    ) -> Self {
        Self {
            entity_manager,
            player_manager,
            enemy_manager,
            bullet_manager,
        }
    }

    /// Get the player's entity ID
    pub fn get_player_entity_id(&self) -> EntityId {
        self.player_manager.player().entity_id()
    }

    /// Get all enemy entity IDs
    pub fn get_enemy_entity_ids(&self) -> Vec<EntityId> {
        self.enemy_manager
            .enemies()
            .iter()
            .map(|enemy| enemy.entity_id())
            .collect()
    }

    /// Get all bullet entity IDs of a specific type
    pub fn get_bullet_entity_ids_by_type(&self, bullet_type: EntityType) -> Vec<EntityId> {
        let mut entity_ids = Vec::new();

        match bullet_type {
            EntityType::PlayerBullet => {
                entity_ids.extend(
                    self.bullet_manager
                        .bullets()
                        .iter()
                        .map(|bullet| bullet.entity_id()),
                );
                entity_ids.extend(
                    self.bullet_manager
                        .metabullets()
                        .iter()
                        .map(|bullet| bullet.entity_id()),
                );
            }
            EntityType::EnemyBullet => {
                // When we add enemy bullets, they'll go here
            }
            _ => {
                // Invalid bullet type, return empty
            }
        }

        entity_ids
    }

    /// Get all entity IDs of a specific type
    pub fn get_entities_by_type(&self, entity_type: EntityType) -> Vec<EntityId> {
        match entity_type {
            EntityType::Player => vec![self.get_player_entity_id()],
            EntityType::Enemy => self.get_enemy_entity_ids(),
            EntityType::PlayerBullet => {
                self.get_bullet_entity_ids_by_type(EntityType::PlayerBullet)
            }
            EntityType::EnemyBullet => self.get_bullet_entity_ids_by_type(EntityType::EnemyBullet),
        }
    }

    /// Get the position of any entity by its ID
    pub fn get_entity_position(&self, entity_id: EntityId) -> Option<crate::engine::Vec3> {
        // Check player
        if self.player_manager.player().entity_id() == entity_id {
            return Some(self.player_manager.player().position());
        }

        // Check enemies
        for enemy in self.enemy_manager.enemies() {
            if enemy.entity_id() == entity_id {
                return Some(enemy.position());
            }
        }

        // Check bullets
        for bullet in self.bullet_manager.bullets() {
            if bullet.entity_id() == entity_id {
                return Some(bullet.position());
            }
        }

        // Check metabullets
        for bullet in self.bullet_manager.metabullets() {
            if bullet.entity_id() == entity_id {
                return Some(bullet.position());
            }
        }

        None
    }

    /// Get positions of multiple entities at once
    pub fn get_entity_positions(
        &self,
        entity_ids: &[EntityId],
    ) -> Vec<(EntityId, crate::engine::Vec3)> {
        let mut positions = Vec::new();

        for &entity_id in entity_ids {
            if let Some(position) = self.get_entity_position(entity_id) {
                positions.push((entity_id, position));
            }
        }

        positions
    }
}
