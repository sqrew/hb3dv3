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
    pub const ENEMY: CollisionMask = CollisionMask(
        (1 << EntityType::Player as u32) | (1 << EntityType::PlayerBullet as u32)
    );
    
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