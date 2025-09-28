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
    /// Player collides with enemies and collectibles
    pub const PLAYER: CollisionMask = CollisionMask(
        (1 << EntityType::Enemy as u32) | (1 << EntityType::Collectible as u32)
    );

    /// Enemies collide with player and player bullets
    pub const ENEMY: CollisionMask =
        CollisionMask((1 << EntityType::Player as u32) | (1 << EntityType::PlayerBullet as u32));

    /// Player bullets collide with enemies and other player bullets
    pub const PLAYER_BULLET: CollisionMask =
        CollisionMask((1 << EntityType::Enemy as u32) | (1 << EntityType::PlayerBullet as u32));

    /// Enemy bullets collide with player only (when implemented)
    pub const ENEMY_BULLET: CollisionMask = CollisionMask(1 << EntityType::Player as u32);

    /// Large bodies collide with everything that can hit them
    pub const LARGE_BODY: CollisionMask = CollisionMask(
        (1 << EntityType::Player as u32)
            | (1 << EntityType::Enemy as u32)
            | (1 << EntityType::PlayerBullet as u32)
            | (1 << EntityType::EnemyBullet as u32),
    );

    /// Collectibles collide with player only
    pub const COLLECTIBLE: CollisionMask = CollisionMask(1 << EntityType::Player as u32);
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
            EntityType::LargeBody => CollisionMask::LARGE_BODY,
            EntityType::Collectible => CollisionMask::COLLECTIBLE,
        }
    }
}

/// Dedicated collision manager that processes GPU collision results and generates events
pub struct CollisionManager {
    /// Events generated from collision processing
    event_queue: Vec<crate::engine::dispatcher::EventType>,
    /// Bullets to remove due to large body collisions
    bullets_to_remove: Vec<crate::engine::entity::EntityId>,
    /// Enemies to remove due to large body collisions
    enemies_to_remove: Vec<crate::engine::entity::EntityId>,
    /// Bullet velocity changes to apply (entity_id, new_velocity)
    bullet_velocity_changes: Vec<(crate::engine::entity::EntityId, crate::engine::Vec3)>,
}

impl CollisionManager {
    pub fn new() -> Self {
        Self {
            event_queue: Vec::new(),
            bullets_to_remove: Vec::new(),
            enemies_to_remove: Vec::new(),
            bullet_velocity_changes: Vec::new(),
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
                }
                // PlayerBullet hits LargeBody
                (
                    Some(crate::engine::entity::EntityType::PlayerBullet),
                    Some(crate::engine::entity::EntityType::LargeBody),
                ) => {
                    // Queue bullet for removal (handle in post-processing)
                    self.bullets_to_remove.push(entity_a);

                    // Generate collision event with impact point for particle effects
                    if let Some(bullet) = bullet_manager
                        .bullets()
                        .iter()
                        .find(|b| b.entity_id() == entity_a)
                    {
                        self.event_queue
                            .push(crate::engine::dispatcher::EventType::Collision(
                                crate::engine::dispatcher::CollisionEvent::BulletHitLargeBody {
                                    bullet_id: entity_a,
                                    large_body_id: entity_b,
                                    impact_point: bullet.position(),
                                },
                            ));
                    }
                }
                // LargeBody hits PlayerBullet (reverse)
                (
                    Some(crate::engine::entity::EntityType::LargeBody),
                    Some(crate::engine::entity::EntityType::PlayerBullet),
                ) => {
                    // Queue bullet for removal (handle in post-processing)
                    self.bullets_to_remove.push(entity_b);

                    // Generate collision event with impact point for particle effects
                    if let Some(bullet) = bullet_manager
                        .bullets()
                        .iter()
                        .find(|b| b.entity_id() == entity_b)
                    {
                        self.event_queue
                            .push(crate::engine::dispatcher::EventType::Collision(
                                crate::engine::dispatcher::CollisionEvent::BulletHitLargeBody {
                                    bullet_id: entity_b,
                                    large_body_id: entity_a,
                                    impact_point: bullet.position(),
                                },
                            ));
                    }
                }
                // Enemy hits LargeBody
                (
                    Some(crate::engine::entity::EntityType::Enemy),
                    Some(crate::engine::entity::EntityType::LargeBody),
                ) => {
                    // Queue enemy for removal (instant death on large body collision)
                    self.enemies_to_remove.push(entity_a);

                    // Generate collision event with impact point for particle effects
                    if let Some(enemy_pos) = enemy_manager.get_enemy_position(entity_a) {
                        self.event_queue
                            .push(crate::engine::dispatcher::EventType::Collision(
                                crate::engine::dispatcher::CollisionEvent::EnemyHitLargeBody {
                                    enemy_id: entity_a,
                                    large_body_id: entity_b,
                                    impact_point: enemy_pos,
                                },
                            ));
                    }
                }
                // LargeBody hits Enemy (reverse)
                (
                    Some(crate::engine::entity::EntityType::LargeBody),
                    Some(crate::engine::entity::EntityType::Enemy),
                ) => {
                    // Queue enemy for removal (instant death on large body collision)
                    self.enemies_to_remove.push(entity_b);

                    // Generate collision event with impact point for particle effects
                    if let Some(enemy_pos) = enemy_manager.get_enemy_position(entity_b) {
                        self.event_queue
                            .push(crate::engine::dispatcher::EventType::Collision(
                                crate::engine::dispatcher::CollisionEvent::EnemyHitLargeBody {
                                    enemy_id: entity_b,
                                    large_body_id: entity_a,
                                    impact_point: enemy_pos,
                                },
                            ));
                    }
                }
                // Player hits LargeBody
                (
                    Some(crate::engine::entity::EntityType::Player),
                    Some(crate::engine::entity::EntityType::LargeBody),
                ) => {
                    // Player takes damage from hitting large body
                    // Could add impact damage event here
                }
                // LargeBody hits Player (reverse)
                (
                    Some(crate::engine::entity::EntityType::LargeBody),
                    Some(crate::engine::entity::EntityType::Player),
                ) => {
                    // Player takes damage from hitting large body
                    // Could add impact damage event here
                }
                // Player hits Collectible
                (
                    Some(crate::engine::entity::EntityType::Player),
                    Some(crate::engine::entity::EntityType::Collectible),
                ) => {
                    self.event_queue
                        .push(crate::engine::dispatcher::EventType::Collision(
                            crate::engine::dispatcher::CollisionEvent::PlayerHitCollectible {
                                player_id: entity_a,
                                collectible_id: entity_b,
                            },
                        ));
                }
                // Collectible hits Player (reverse)
                (
                    Some(crate::engine::entity::EntityType::Collectible),
                    Some(crate::engine::entity::EntityType::Player),
                ) => {
                    self.event_queue
                        .push(crate::engine::dispatcher::EventType::Collision(
                            crate::engine::dispatcher::CollisionEvent::PlayerHitCollectible {
                                player_id: entity_b,
                                collectible_id: entity_a,
                            },
                        ));
                }
                // PlayerBullet hits PlayerBullet (bullet-bullet collision)
                (
                    Some(crate::engine::entity::EntityType::PlayerBullet),
                    Some(crate::engine::entity::EntityType::PlayerBullet),
                ) => {
                    // Get both bullet velocities and positions for elastic collision
                    if let (Some(vel_a), Some(vel_b)) = (
                        bullet_manager.get_bullet_velocity(entity_a),
                        bullet_manager.get_bullet_velocity(entity_b),
                    ) {
                        // Calculate separation direction to prevent repeated collisions
                        if let (Some(bullet_a), Some(bullet_b)) = (
                            bullet_manager
                                .bullets()
                                .iter()
                                .find(|b| b.entity_id() == entity_a),
                            bullet_manager
                                .bullets()
                                .iter()
                                .find(|b| b.entity_id() == entity_b),
                        ) {
                            // Calculate separation vector (from bullet_b to bullet_a)
                            let separation = bullet_a.position() - bullet_b.position();
                            let separation_magnitude = separation.magnitude();

                            if separation_magnitude > 0.001 {
                                let separation_dir = separation / separation_magnitude;

                                // Add small separation impulse to prevent sticking
                                // This pushes bullets apart slightly after collision
                                const SEPARATION_IMPULSE: f32 = 2.0; // Adjust as needed
                                let separation_velocity = separation_dir * SEPARATION_IMPULSE;

                                // Apply elastic collision + separation impulse
                                self.bullet_velocity_changes
                                    .push((entity_a, vel_b + separation_velocity));
                                self.bullet_velocity_changes
                                    .push((entity_b, vel_a - separation_velocity));
                            } else {
                                // Fallback: if bullets are at exact same position, use random separation
                                let random_dir = crate::engine::Vec3::new(
                                    (entity_a.0 as f32 * 0.1).sin(),
                                    (entity_b.0 as f32 * 0.1).cos(),
                                    0.0,
                                )
                                .normalize();
                                const SEPARATION_IMPULSE: f32 = 2.0;
                                let separation_velocity = random_dir * SEPARATION_IMPULSE;

                                self.bullet_velocity_changes
                                    .push((entity_a, vel_b + separation_velocity));
                                self.bullet_velocity_changes
                                    .push((entity_b, vel_a - separation_velocity));
                            }
                        } else {
                            // Fallback to original behavior if bullets not found
                            self.bullet_velocity_changes.push((entity_a, vel_b));
                            self.bullet_velocity_changes.push((entity_b, vel_a));
                        }

                        // Generate collision event with impact point for particle effects
                        // Calculate approximate impact point as midpoint between bullets
                        if let (Some(bullet_a), Some(bullet_b)) = (
                            bullet_manager
                                .bullets()
                                .iter()
                                .find(|b| b.entity_id() == entity_a),
                            bullet_manager
                                .bullets()
                                .iter()
                                .find(|b| b.entity_id() == entity_b),
                        ) {
                            let impact_point = (bullet_a.position() + bullet_b.position()) * 0.5;
                            self.event_queue
                                .push(crate::engine::dispatcher::EventType::Collision(
                                    crate::engine::dispatcher::CollisionEvent::BulletHitBullet {
                                        bullet_a_id: entity_a,
                                        bullet_b_id: entity_b,
                                        impact_point,
                                    },
                                ));
                        }
                    }
                }
                _ => {
                    // Unhandled collision type or invalid entities
                    if type_a.is_none() || type_b.is_none() {
                        // Check if these are fractal bullet entities (IDs >= 1M) that are being cleaned up
                        // This happens due to timing between collision detection and entity cleanup
                        let is_fractal_cleanup = entity_a.0 >= 1000000 || entity_b.0 >= 1000000;

                        if !is_fractal_cleanup {
                            println!(
                                "⚠️ Warning: Collision between invalid entities {} and {}",
                                entity_a.0, entity_b.0
                            );
                        }
                        // Fractal bullet cleanup collisions are expected and can be silently ignored
                    }
                }
            }
        }
    }

    /// Process large body vs large body collisions (CPU-based collision detection)
    pub fn process_large_body_collisions(
        &mut self,
        large_body_manager: &crate::scene::LargeBodyManager,
    ) {
        let large_bodies = large_body_manager.bodies();

        // Check collisions between all pairs of large bodies
        for i in 0..large_bodies.len() {
            for j in (i + 1)..large_bodies.len() {
                let body_a = &large_bodies[i];
                let body_b = &large_bodies[j];

                // Calculate distance between bodies
                let distance_vec = body_a.position() - body_b.position();
                let distance = distance_vec.magnitude();

                // Check if they're colliding (using collision radii with reduced buffer)
                let collision_distance = body_a.collision_radius() + body_b.collision_radius();
                let detection_distance = collision_distance * 0.9; // Reduced buffer - trigger when closer to visual contact

                if distance < detection_distance && distance > 0.001 {
                    // Calculate impact point (surface contact point)
                    let direction = distance_vec.normalize();
                    let impact_point = body_b.position() + direction * body_b.collision_radius();

                    // Create collision event
                    self.event_queue
                        .push(crate::engine::dispatcher::EventType::Collision(
                            crate::engine::dispatcher::CollisionEvent::LargeBodyHitLargeBody {
                                large_body_a_id: body_a.entity_id(),
                                large_body_b_id: body_b.entity_id(),
                                impact_point,
                            },
                        ));
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

    /// Get and clear bullets that need to be removed due to large body collisions
    pub fn drain_bullets_to_remove(&mut self) -> Vec<crate::engine::entity::EntityId> {
        self.bullets_to_remove.drain(..).collect()
    }

    /// Get and clear enemies that need to be removed due to large body collisions
    pub fn drain_enemies_to_remove(&mut self) -> Vec<crate::engine::entity::EntityId> {
        self.enemies_to_remove.drain(..).collect()
    }

    /// Get and clear bullet velocity changes that need to be applied
    pub fn drain_bullet_velocity_changes(
        &mut self,
    ) -> Vec<(crate::engine::entity::EntityId, crate::engine::Vec3)> {
        self.bullet_velocity_changes.drain(..).collect()
    }
}

impl Default for CollisionManager {
    fn default() -> Self {
        Self::new()
    }
}
