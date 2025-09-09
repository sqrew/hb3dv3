use crate::engine::entity::{EntityId, EntityType};
use crate::engine::{CollisionMask, Vec3};
use crate::graphics::{Color, Primitive, PrimitiveType};

pub struct Bullet {
    entity_id: EntityId,
    pos: Vec3,
    vel: Vec3,
    ttl: f32,
    damage: f32,
    collision_radius: f32,
    collision_mask: CollisionMask,
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
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.pos += self.vel * dt;
        self.ttl -= dt;
    }

    pub fn is_alive(&self) -> bool {
        self.ttl > 0.0
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
}
pub struct BulletManager {
    bullets: Vec<Bullet>,
    metabullets: Vec<MetaBullet>,
}

impl BulletManager {
    pub fn new() -> Self {
        BulletManager {
            bullets: Vec::new(),
            metabullets: Vec::new(),
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

    pub fn spawn_bullet(
        &mut self,
        entity_id: EntityId,
        pos: Vec3,
        speed: f32,
        direction: Vec3,
        ttl: f32,
        damage: f32,
    ) {
        let vel = direction.normalize() * speed;
        self.bullets
            .push(Bullet::new(entity_id, pos, vel, ttl, damage));
    }
    pub fn spawn_metabullet(
        &mut self,
        entity_id: EntityId,
        pos: Vec3,
        speed: f32,
        direction: Vec3,
        ttl: f32,
        damage: f32,
        on_hit: Option<Vec<Box<dyn OnHitEffect>>>,
        on_expire: Option<Vec<Box<dyn OnExpireEffect>>>,
    ) {
        let vel = direction.normalize() * speed;
        self.metabullets.push(MetaBullet::new(
            entity_id, pos, vel, ttl, damage, on_hit, on_expire,
        ));
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
}

pub struct MetaBullet {
    entity_id: EntityId,
    pos: Vec3,
    vel: Vec3,
    ttl: f32,
    damage: f32,
    collision_radius: f32,
    collision_mask: CollisionMask,
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
            on_hit,
            on_expire,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.pos += self.vel * dt;
        self.ttl -= dt;
    }

    pub fn is_alive(&self) -> bool {
        self.ttl > 0.0
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
}

pub trait OnHitEffect {}
pub trait OnExpireEffect {}
