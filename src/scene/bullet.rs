use crate::engine::entity::{EntityId, EntityType};
use crate::engine::{CollisionMask, Vec3};
use crate::engine::dispatcher::{EventType, CollisionEvent};
use crate::graphics::{Color, Primitive, PrimitiveType};
use crate::scene::GravityAffected;

/// Unified projectile type system - defines what kind of projectile to spawn
#[derive(Debug, Clone)]
pub enum ProjectileType {
    /// Basic projectile - fast, lightweight, no special effects
    Basic { 
        damage: f32, 
        velocity: Vec3, 
        lifetime: f32 
    },
    /// Custom projectile with arbitrary effects
    Custom { 
        damage: f32, 
        velocity: Vec3, 
        lifetime: f32, 
        effects: ProjectileEffects 
    },
}

/// Container for complex projectile effects
pub struct ProjectileEffects {
    pub on_hit: Option<Vec<Box<dyn OnHitEffect>>>,
    pub on_expire: Option<Vec<Box<dyn OnExpireEffect>>>,
}

impl std::fmt::Debug for ProjectileEffects {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProjectileEffects")
            .field("on_hit", &self.on_hit.as_ref().map(|v| format!("{} effects", v.len())))
            .field("on_expire", &self.on_expire.as_ref().map(|v| format!("{} effects", v.len())))
            .finish()
    }
}

impl Clone for ProjectileEffects {
    fn clone(&self) -> Self {
        // Note: We can't clone trait objects, so we just create empty effects
        // This is a limitation for now - custom effects can't be cloned
        Self {
            on_hit: None,
            on_expire: None,
        }
    }
}

impl Default for ProjectileEffects {
    fn default() -> Self {
        Self {
            on_hit: None,
            on_expire: None,
        }
    }
}

pub struct Bullet {
    entity_id: EntityId,
    pos: Vec3,
    vel: Vec3,
    ttl: f32,
    damage: f32,
    collision_radius: f32,
    collision_mask: CollisionMask,
    marked_for_removal: bool,
    mass: f32,
    applied_force: Vec3,
}

impl Bullet {
    pub fn new(entity_id: EntityId, pos: Vec3, vel: Vec3, ttl: f32, damage: f32) -> Self {
        Bullet {
            entity_id,
            pos,
            vel,
            ttl,
            damage,
            collision_radius: 0.1,
            collision_mask: CollisionMask::from(EntityType::PlayerBullet),
            marked_for_removal: false,
            mass: 0.5, // Light bullet mass in kg
            applied_force: Vec3::zeros(),
        }
    }

    pub fn update(&mut self, dt: f32) {
        // Apply gravitational forces (F = ma, so a = F/m)
        let gravity_acceleration = self.applied_force / self.mass;
        
        // Update velocity with gravity effects (bullets are ballistic)
        self.vel += gravity_acceleration * dt;
        
        // Update position
        self.pos += self.vel * dt;
        self.ttl -= dt;
        
        // Reset applied force for next frame
        self.applied_force = Vec3::zeros();
    }

    pub fn is_alive(&self) -> bool {
        self.ttl > 0.0 && !self.marked_for_removal
    }

    pub fn position(&self) -> Vec3 {
        self.pos
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
    
    pub fn damage(&self) -> f32 {
        self.damage
    }

    pub fn mark_for_removal(&mut self) {
        self.marked_for_removal = true;
    }
    
    pub fn velocity(&self) -> Vec3 {
        self.vel
    }
    
    pub fn set_velocity(&mut self, velocity: Vec3) {
        self.vel = velocity;
    }
}

impl GravityAffected for Bullet {
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

pub struct BulletManager {
    bullets: Vec<Bullet>,
    metabullets: Vec<MetaBullet>,
    event_queue: Vec<EventType>,
}

impl BulletManager {
    pub fn new() -> Self {
        BulletManager {
            bullets: Vec::new(),
            metabullets: Vec::new(),
            event_queue: Vec::new(),
        }
    }

    pub fn update(&mut self, dt: f32) {
        // Update all bullets
        for bullet in self.bullets.iter_mut() {
            bullet.update(dt);
        }

        for metabullet in self.metabullets.iter_mut() {
            metabullet.update(dt)
        }

        // Remove expired bullets
        self.bullets.retain(|b| b.is_alive());
        self.metabullets.retain(|mb| mb.is_alive());
    }


    pub fn get_render_data(&self) -> Vec<Primitive> {
        let mut renderable: Vec<Primitive> = Vec::new();
        let bullets: Vec<Primitive> = self
            .bullets
            .iter()
            .map(|bullet| {
                Primitive::new(
                    PrimitiveType::Tetrahedron,
                    bullet.pos,
                    Color::new(1.0, 1.0, 0.0, 1.0), // Yellow bullets
                )
            })
            .collect();

        let metabullets: Vec<Primitive> = self
            .metabullets
            .iter()
            .map(|bullet| {
                Primitive::new(
                    PrimitiveType::Tetrahedron,
                    bullet.pos,
                    Color::new(1.0, 0.0, 0.0, 1.0), // Red bullets
                )
            })
            .collect();
        renderable.extend(bullets);
        renderable.extend(metabullets);
        renderable
    }

    pub fn bullets(&self) -> &[Bullet] {
        &self.bullets
    }
    
    pub fn bullets_mut(&mut self) -> &mut [Bullet] {
        &mut self.bullets
    }

    pub fn metabullets(&self) -> &[MetaBullet] {
        &self.metabullets
    }

    pub fn remove_bullet(&mut self, index: usize) -> bool {
        if index < self.bullets.len() {
            self.bullets.remove(index);
            true
        } else {
            false
        }
    }

    pub fn remove_metabullet(&mut self, index: usize) -> bool {
        if index < self.metabullets.len() {
            self.metabullets.remove(index);
            true
        } else {
            false
        }
    }

    pub fn bullet_count(&self) -> usize {
        self.bullets.len()
    }

    pub fn metabullet_count(&self) -> usize {
        self.metabullets.len()
    }

    /// Clean up dead bullets and return their entity IDs for destruction
    pub fn cleanup_dead_bullets(&mut self) -> Vec<crate::engine::entity::EntityId> {
        let mut destroyed_entities = Vec::new();

        // Clean up regular bullets
        let mut i = 0;
        while i < self.bullets.len() {
            if !self.bullets[i].is_alive() {
                destroyed_entities.push(self.bullets[i].entity_id());
                self.bullets.remove(i);
            } else {
                i += 1;
            }
        }

        // Clean up metabullets
        let mut i = 0;
        while i < self.metabullets.len() {
            if !self.metabullets[i].is_alive() {
                destroyed_entities.push(self.metabullets[i].entity_id());
                self.metabullets.remove(i);
            } else {
                i += 1;
            }
        }

        destroyed_entities
    }
    
    /// Find a bullet by entity ID and get its damage
    pub fn get_bullet_damage(&self, entity_id: crate::engine::entity::EntityId) -> Option<f32> {
        // Check regular bullets
        for bullet in &self.bullets {
            if bullet.entity_id() == entity_id {
                return Some(bullet.damage());
            }
        }
        // Check metabullets
        for metabullet in &self.metabullets {
            if metabullet.entity_id() == entity_id {
                return Some(metabullet.damage());
            }
        }
        None
    }
    
    pub fn get_bullet_velocity(&self, entity_id: crate::engine::entity::EntityId) -> Option<Vec3> {
        // Check regular bullets
        for bullet in &self.bullets {
            if bullet.entity_id() == entity_id {
                return Some(bullet.velocity());
            }
        }
        // Check metabullets  
        for metabullet in &self.metabullets {
            if metabullet.entity_id() == entity_id {
                return Some(metabullet.velocity());
            }
        }
        None
    }
    
    pub fn set_bullet_velocity(&mut self, entity_id: crate::engine::entity::EntityId, velocity: Vec3) -> bool {
        // Check regular bullets
        for bullet in &mut self.bullets {
            if bullet.entity_id() == entity_id {
                bullet.set_velocity(velocity);
                return true;
            }
        }
        // Check metabullets
        for metabullet in &mut self.metabullets {
            if metabullet.entity_id() == entity_id {
                metabullet.set_velocity(velocity);
                return true;
            }
        }
        false
    }
    
    /// Remove a bullet by entity ID
    pub fn remove_bullet_by_entity_id(&mut self, entity_id: crate::engine::entity::EntityId) -> bool {
        // Check regular bullets
        for i in 0..self.bullets.len() {
            if self.bullets[i].entity_id() == entity_id {
                self.bullets.remove(i);
                return true;
            }
        }
        // Check metabullets
        for i in 0..self.metabullets.len() {
            if self.metabullets[i].entity_id() == entity_id {
                self.metabullets.remove(i);
                return true;
            }
        }
        false
    }
    
    /// Get and clear the event queue
    pub fn drain_events(&mut self) -> Vec<EventType> {
        self.event_queue.drain(..).collect()
    }
    
    /// Queue an event
    pub fn queue_event(&mut self, event: EventType) {
        self.event_queue.push(event);
    }
    
    /// Queue a collision event when bullet hits something
    pub fn register_hit(&mut self, bullet_id: EntityId, target_id: EntityId, impact_point: Vec3) {
        if let Some(damage) = self.get_bullet_damage(bullet_id) {
            self.event_queue.push(EventType::Collision(CollisionEvent::BulletHitEnemy {
                bullet_id,
                enemy_id: target_id,
                damage,
                impact_point,
            }));
        }
    }
    
    /// Mark a bullet for removal by entity ID (for collision processing)
    pub fn mark_bullet_for_removal(&mut self, entity_id: EntityId) {
        // Check regular bullets
        for bullet in &mut self.bullets {
            if bullet.entity_id() == entity_id {
                bullet.mark_for_removal();
                return;
            }
        }
        // Check metabullets
        for metabullet in &mut self.metabullets {
            if metabullet.entity_id() == entity_id {
                metabullet.mark_for_removal();
                return;
            }
        }
    }
    
    /// Unified projectile spawning method
    pub fn spawn_projectile(
        &mut self, 
        entity_id: EntityId, 
        position: Vec3, 
        projectile_type: ProjectileType
    ) {
        match projectile_type {
            ProjectileType::Basic { damage, velocity, lifetime } => {
                // Create lightweight Bullet (same performance as before)
                self.bullets.push(Bullet::new(entity_id, position, velocity, lifetime, damage));
            },
            ProjectileType::Custom { damage, velocity, lifetime, effects } => {
                // Create MetaBullet with custom effects
                self.metabullets.push(MetaBullet::new(
                    entity_id, 
                    position, 
                    velocity, 
                    lifetime, 
                    damage,
                    effects.on_hit,
                    effects.on_expire
                ));
            },
        }
    }
}

pub struct MetaBullet {
    entity_id: EntityId,
    pos: Vec3,
    vel: Vec3,
    ttl: f32,
    damage: f32,
    collision_radius: f32,
    collision_mask: CollisionMask,
    marked_for_removal: bool,
    on_hit: Option<Vec<Box<dyn OnHitEffect>>>,
    on_expire: Option<Vec<Box<dyn OnExpireEffect>>>,
}

impl MetaBullet {
    pub fn new(
        entity_id: EntityId,
        pos: Vec3,
        vel: Vec3,
        ttl: f32,
        damage: f32,
        on_hit: Option<Vec<Box<dyn OnHitEffect>>>,
        on_expire: Option<Vec<Box<dyn OnExpireEffect>>>,
    ) -> Self {
        MetaBullet {
            entity_id,
            pos,
            vel,
            ttl,
            damage,
            collision_radius: 0.15,
            collision_mask: CollisionMask::from(EntityType::PlayerBullet),
            marked_for_removal: false,
            on_hit,
            on_expire,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.pos += self.vel * dt;
        self.ttl -= dt;
    }

    pub fn is_alive(&self) -> bool {
        self.ttl > 0.0 && !self.marked_for_removal
    }

    pub fn position(&self) -> Vec3 {
        self.pos
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

    pub fn damage(&self) -> f32 {
        self.damage
    }

    pub fn mark_for_removal(&mut self) {
        self.marked_for_removal = true;
    }
    
    pub fn velocity(&self) -> Vec3 {
        self.vel
    }
    
    pub fn set_velocity(&mut self, velocity: Vec3) {
        self.vel = velocity;
    }
}

pub trait OnHitEffect {}
pub trait OnExpireEffect {}
