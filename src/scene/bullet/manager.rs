use crate::engine::{EntityId, Vec3};
use crate::engine::dispatcher::{CollisionEvent, EventType};
use crate::graphics::Primitive;
use crate::scene::GravityAffected;
use super::types::{Bullet, MetaBullet, ProjectileType};
use super::effects::{OnExpireEffect, ExplosionEffect};

/// Manages all bullets in the game
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
        self.update_with_targets(dt, &[]);
    }

    /// Update bullets with target positions for seeking behavior
    pub fn update_with_targets(&mut self, dt: f32, enemy_positions: &[Vec3]) {
        // Update all bullets
        for bullet in self.bullets.iter_mut() {
            bullet.update(dt);
        }

        // Update metabullets with seeking behavior
        for metabullet in self.metabullets.iter_mut() {
            // Apply seeking forces if this metabullet has seeking enabled
            if metabullet.seeking() && !enemy_positions.is_empty() {
                Self::apply_seeking_force(metabullet, enemy_positions, dt);
            }
            metabullet.update(dt);
        }

        // Check for expired metabullets and trigger their OnExpireEffects
        let mut expired_metabullets = Vec::new();
        for (index, metabullet) in self.metabullets.iter().enumerate() {
            if !metabullet.is_alive() {
                if let Some(ref on_expire_effects) = metabullet.on_expire() {
                    for effect in on_expire_effects.iter() {
                        let explosion_events = effect.on_expire(metabullet.position());
                        // Add explosion events to the event queue
                        for explosion_event in explosion_events {
                            self.event_queue
                                .push(crate::engine::dispatcher::EventType::Explosion(
                                    explosion_event,
                                ));
                        }
                    }
                }
                expired_metabullets.push(index);
            }
        }

        // Remove expired bullets (regular bullets first, then metabullets)
        self.bullets.retain(|b| b.is_alive());

        // Remove expired metabullets (in reverse order to maintain indices)
        for &index in expired_metabullets.iter().rev() {
            self.metabullets.remove(index);
        }
    }

    /// Apply seeking force to a metabullet towards the nearest enemy
    fn apply_seeking_force(metabullet: &mut MetaBullet, enemy_positions: &[Vec3], dt: f32) {
        let mut nearest_enemy: Option<Vec3> = None;
        let mut nearest_distance = metabullet.max_seeking_range();

        // Find the nearest enemy within seeking range
        for &enemy_pos in enemy_positions {
            let distance = (enemy_pos - metabullet.position()).magnitude();
            if distance < nearest_distance {
                nearest_distance = distance;
                nearest_enemy = Some(enemy_pos);
            }
        }

        // Apply steering force towards the nearest enemy
        if let Some(target_pos) = nearest_enemy {
            let to_target = (target_pos - metabullet.position()).normalize();
            let current_direction = metabullet.velocity().normalize();

            // Calculate steering force (desired direction - current direction)
            let steering_force = (to_target - current_direction) * metabullet.seeking_force();

            // Apply seeking force as acceleration (F = ma, so add F/m to velocity)
            let seeking_acceleration = steering_force / metabullet.mass();
            let new_velocity = metabullet.velocity() + seeking_acceleration * dt;
            metabullet.set_velocity(new_velocity);
        }
    }

    pub fn get_render_data(&self) -> Vec<Primitive> {
        let mut renderable: Vec<Primitive> = Vec::new();

        // Render regular bullets with their custom visuals
        let bullets: Vec<Primitive> = self
            .bullets
            .iter()
            .map(|bullet| {
                let visuals = bullet.visuals();
                Primitive::new(visuals.primitive_type, bullet.position(), visuals.color)
                    .with_uniform_scale(visuals.scale)
            })
            .collect();

        // Render metabullets with their custom visuals
        let metabullets: Vec<Primitive> = self
            .metabullets
            .iter()
            .map(|bullet| {
                let visuals = bullet.visuals();
                Primitive::new(visuals.primitive_type, bullet.position(), visuals.color)
                    .with_uniform_scale(visuals.scale)
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

    pub fn set_bullet_velocity(
        &mut self,
        entity_id: crate::engine::entity::EntityId,
        velocity: Vec3,
    ) -> bool {
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
    pub fn remove_bullet_by_entity_id(
        &mut self,
        entity_id: crate::engine::entity::EntityId,
    ) -> bool {
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
            self.event_queue
                .push(EventType::Collision(CollisionEvent::BulletHitEnemy {
                    bullet_id,
                    enemy_id: target_id,
                    damage,
                    impact_point,
                }));
        }

        // Check if this is a MetaBullet with OnHitEffects and trigger them
        self.trigger_metabullet_on_hit_effects(bullet_id, target_id, impact_point);
    }

    /// Trigger OnHitEffects for MetaBullets
    fn trigger_metabullet_on_hit_effects(
        &mut self,
        bullet_id: EntityId,
        target_id: EntityId,
        impact_point: Vec3,
    ) {
        // Early exit if no metabullets exist (optimization for regular bullets)
        if self.metabullets.is_empty() {
            return;
        }

        // Find the metabullet and trigger its effects
        for metabullet in &self.metabullets {
            if metabullet.entity_id() == bullet_id {
                if let Some(ref on_hit_effects) = metabullet.on_hit() {
                    for effect in on_hit_effects.iter() {
                        let chain_events = effect.on_hit(impact_point, Some(target_id));
                        // Add chain lightning events to the event queue
                        for chain_event in chain_events {
                            self.event_queue
                                .push(EventType::ChainLightning(chain_event));
                        }
                    }
                }
                break;
            }
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
        projectile_type: ProjectileType,
    ) {
        match projectile_type {
            ProjectileType::Basic {
                damage,
                velocity,
                lifetime,
                mass,
                visuals,
            } => {
                // Create lightweight Bullet (same performance as before)
                self.bullets.push(Bullet::new(
                    entity_id, position, velocity, lifetime, damage, mass, visuals,
                ));
            }
            ProjectileType::Custom {
                damage,
                velocity,
                lifetime,
                mass,
                effects,
                visuals,
            } => {
                // Store the on_hit effects before passing to MetaBullet
                let on_hit_effects = effects.on_hit;
                let on_expire_effects = effects.on_expire;

                // Create MetaBullet with custom effects
                self.metabullets.push(MetaBullet::new(
                    entity_id,
                    position,
                    velocity,
                    lifetime,
                    damage,
                    mass,
                    on_hit_effects,
                    on_expire_effects,
                    visuals,
                ));
            }
            ProjectileType::SeekingExplosive {
                damage,
                velocity,
                lifetime,
                mass,
                seeking_force,
                seeking_range,
                explosion_radius,
                explosion_force,
                explosion_duration,
                visuals,
            } => {
                // Create explosion effect for when the seeking missile expires
                let explosion_effect = ExplosionEffect::new(
                    explosion_radius,
                    explosion_force,
                    explosion_duration,
                    crate::scene::explosion::FalloffType::Quadratic,
                );

                let on_expire_effects: Option<Vec<Box<dyn OnExpireEffect>>> =
                    Some(vec![Box::new(explosion_effect)]);

                // Create seeking MetaBullet with explosion effect
                self.metabullets.push(MetaBullet::new_seeking(
                    entity_id,
                    position,
                    velocity,
                    lifetime,
                    damage,
                    mass,
                    None, // No on_hit effects for seeking explosive
                    on_expire_effects,
                    seeking_force,
                    seeking_range,
                    visuals,
                ));
            }
        }
    }
}