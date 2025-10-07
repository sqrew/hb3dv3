use super::effects::{ExplosionEffect, OnExpireEffect};
use super::pool::BulletPool;
use super::types::{Bullet, MetaBullet, ProjectileType};
use crate::engine::dispatcher::{CollisionEvent, EventType};
use crate::engine::{EntityId, Vec3};
use crate::graphics::vertex::LineInstance;
use crate::graphics::{Color, Primitive};
use crate::scene::GravityAffected;
// Note: FractalBullet and FractalSplitEvent no longer used - fractal data embedded in regular bullets

/// Manages all bullets in the game using a high-performance object pool
pub struct BulletManager {
    pool: BulletPool, // Object pool for bullets and metabullets
    event_queue: Vec<EventType>,
    next_entity_id: u32, // For generating entity IDs for fractal children

    // Performance optimization: Spread fractal splits across frames
    fractal_split_batch_size: usize, // Max bullets to process for splits per frame
    pending_split_indices: Vec<usize>, // Bullets waiting to split (indices into pool)
}

impl BulletManager {
    pub fn new() -> Self {
        // Initialize with high-capacity pools for performance
        let pool = BulletPool::new(128000, 1000); // 2000 bullets, 500 metabullets

        BulletManager {
            pool,
            event_queue: Vec::new(),
            next_entity_id: 1000000, // Start fractal child IDs from 1M to avoid conflicts

            // Performance tuning: Process more fractal splits per frame with pooling
            fractal_split_batch_size: 1000,
            pending_split_indices: Vec::new(),
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.update_with_targets(dt, &[]);
    }

    /// Update bullets with target positions for seeking behavior
    pub fn update_with_targets(
        &mut self,
        dt: f32,
        enemy_positions: &[Vec3],
    ) -> Vec<crate::engine::entity::EntityId> {
        // Update all active bullets using pool indices
        let bullet_indices = self.pool.active_bullet_indices().to_vec();
        for index in bullet_indices {
            if let Some(bullet) = self.pool.get_bullet_mut(index) {
                if bullet.active() {
                    bullet.update(dt);
                }
            }
        }

        // Update metabullets with seeking behavior using pool indices
        let metabullet_indices = self.pool.active_metabullet_indices().to_vec();
        for index in metabullet_indices {
            if let Some(metabullet) = self.pool.get_metabullet_mut(index) {
                if metabullet.active() {
                    // Apply seeking forces if this metabullet has seeking enabled
                    if metabullet.seeking() && !enemy_positions.is_empty() {
                        Self::apply_seeking_force(metabullet, enemy_positions, dt);
                    }
                    metabullet.update(dt);
                }
            }
        }

        // Spawn trail particles for seeking metabullets
        self.spawn_seeking_trail_particles(dt);

        // Handle fractal splitting for regular bullets and collect new entity IDs
        let new_fractal_entities = self.handle_fractal_splitting(dt);

        // Check for expired metabullets and trigger their OnExpireEffects
        // Need to collect indices first to avoid borrowing conflicts
        let metabullet_check_indices = self.pool.active_metabullet_indices().to_vec();
        for index in metabullet_check_indices {
            if let Some(metabullet) = self.pool.get_metabullet(index) {
                if !metabullet.is_alive() {
                    if let Some(ref on_expire_effects) = metabullet.on_expire() {
                        for effect in on_expire_effects.iter() {
                            let explosion_events = effect.on_expire(metabullet.position());
                            // Add explosion events to the event queue
                            for explosion_event in explosion_events {
                                self.event_queue.push(
                                    crate::engine::dispatcher::EventType::Explosion(
                                        explosion_event,
                                    ),
                                );
                            }
                        }
                    }
                }
            }
        }

        // Clean up dead bullets using pool cleanup (automatically returns them to pool)
        let dead_entity_ids = self.pool.cleanup_dead_bullets();

        // Return new fractal entity IDs for registration
        let mut all_new_entities = new_fractal_entities;
        all_new_entities.extend(dead_entity_ids);
        all_new_entities
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

    /// Spawn trail particles for seeking metabullets
    fn spawn_seeking_trail_particles(&mut self, dt: f32) {
        const TRAIL_SPAWN_INTERVAL: f32 = 0.01; // Spawn particles every 20ms (50 per second)
        const PARTICLE_COUNT: u32 = 1; // Spawn 3 particles per interval
        const PARTICLE_LIFETIME: f32 = 5.0; // Particles last 0.5 seconds

        let metabullet_indices = self.pool.active_metabullet_indices().to_vec();
        for index in metabullet_indices {
            if let Some(metabullet) = self.pool.get_metabullet_mut(index) {
                if metabullet.active() && metabullet.seeking() {
                    // Update trail spawn timer
                    metabullet.update_trail_timer(dt);

                    // Check if it's time to spawn particles
                    if metabullet.trail_spawn_timer() >= TRAIL_SPAWN_INTERVAL {
                        // Reset timer
                        metabullet.reset_trail_timer();

                        // Spawn trail particles at current position
                        let color = metabullet.visuals().color;

                        // Emit graphics event for particle spawning
                        self.event_queue
                            .push(crate::engine::dispatcher::EventType::Graphics(
                                crate::engine::dispatcher::GraphicsEvent::SpawnParticles {
                                    position: metabullet.position(),
                                    velocity: -metabullet.velocity() * 0.1, // Small backward velocity
                                    count: PARTICLE_COUNT,
                                    lifetime: PARTICLE_LIFETIME,
                                    color,
                                },
                            ));
                    }
                }
            }
        }
    }

    pub fn get_render_data(&self) -> Vec<Primitive> {
        let mut renderable: Vec<Primitive> = Vec::new();

        // Render active bullets with their custom visuals
        for &index in self.pool.active_bullet_indices() {
            if let Some(bullet) = self.pool.get_bullet(index) {
                if bullet.active() {
                    let visuals = bullet.visuals();
                    let primitive =
                        Primitive::new(visuals.primitive_type, bullet.position(), visuals.color)
                            .with_uniform_scale(visuals.scale);
                    renderable.push(primitive);
                }
            }
        }

        // Render active metabullets with their custom visuals
        for &index in self.pool.active_metabullet_indices() {
            if let Some(metabullet) = self.pool.get_metabullet(index) {
                if metabullet.active() {
                    let visuals = metabullet.visuals();
                    let primitive = Primitive::new(
                        visuals.primitive_type,
                        metabullet.position(),
                        visuals.color,
                    )
                    .with_uniform_scale(visuals.scale);
                    renderable.push(primitive);
                }
            }
        }

        renderable
    }

    pub fn bullets(&self) -> Vec<&Bullet> {
        self.pool.active_bullets()
    }

    pub fn bullets_mut(&mut self) -> Vec<&mut Bullet> {
        self.pool.active_bullets_mut()
    }

    pub fn metabullets(&self) -> Vec<&MetaBullet> {
        self.pool.active_metabullets()
    }

    /// Get bullets as slice for external systems (WARNING: includes inactive bullets)
    /// Use with caution - external systems must check bullet.active()
    pub fn bullets_slice(&self) -> &[Bullet] {
        self.pool.bullets_slice()
    }

    /// Get metabullets as slice for external systems (WARNING: includes inactive metabullets)
    /// Use with caution - external systems must check metabullet.active()
    pub fn metabullets_slice(&self) -> &[MetaBullet] {
        self.pool.metabullets_slice()
    }

    /// Get pool reference for direct access (avoid when possible)
    pub fn pool(&self) -> &BulletPool {
        &self.pool
    }

    /// Get mutable pool reference for direct access (avoid when possible)
    pub fn pool_mut(&mut self) -> &mut BulletPool {
        &mut self.pool
    }

    /// Collect active bullets for physics (UNSAFE but necessary for physics system)
    pub fn collect_active_bullets_mut_unsafe(&mut self) -> Vec<&mut Bullet> {
        unsafe { self.pool.collect_active_bullets_mut_unsafe() }
    }

    pub fn remove_bullet(&mut self, index: usize) -> bool {
        self.pool.release_bullet(index);
        true
    }

    pub fn remove_metabullet(&mut self, index: usize) -> bool {
        self.pool.release_metabullet(index);
        true
    }

    pub fn bullet_count(&self) -> usize {
        self.pool.active_bullet_indices().len()
    }

    pub fn metabullet_count(&self) -> usize {
        self.pool.active_metabullet_indices().len()
    }

    /// Clean up dead bullets and return their entity IDs for destruction
    pub fn cleanup_dead_bullets(&mut self) -> Vec<crate::engine::entity::EntityId> {
        // Pool handles cleanup automatically and returns entity IDs
        self.pool.cleanup_dead_bullets()
    }

    /// Find a bullet by entity ID and get its damage
    pub fn get_bullet_damage(&self, entity_id: crate::engine::entity::EntityId) -> Option<f32> {
        // Check active bullets in pool
        for &index in self.pool.active_bullet_indices() {
            if let Some(bullet) = self.pool.get_bullet(index) {
                if bullet.active() && bullet.entity_id() == entity_id {
                    return Some(bullet.damage());
                }
            }
        }

        // Check active metabullets in pool
        for &index in self.pool.active_metabullet_indices() {
            if let Some(metabullet) = self.pool.get_metabullet(index) {
                if metabullet.active() && metabullet.entity_id() == entity_id {
                    return Some(metabullet.damage());
                }
            }
        }

        None
    }

    pub fn get_bullet_velocity(&self, entity_id: crate::engine::entity::EntityId) -> Option<Vec3> {
        // Check active bullets in pool
        for &index in self.pool.active_bullet_indices() {
            if let Some(bullet) = self.pool.get_bullet(index) {
                if bullet.active() && bullet.entity_id() == entity_id {
                    return Some(bullet.velocity());
                }
            }
        }

        // Check active metabullets in pool
        for &index in self.pool.active_metabullet_indices() {
            if let Some(metabullet) = self.pool.get_metabullet(index) {
                if metabullet.active() && metabullet.entity_id() == entity_id {
                    return Some(metabullet.velocity());
                }
            }
        }

        None
    }

    pub fn set_bullet_velocity(
        &mut self,
        entity_id: crate::engine::entity::EntityId,
        velocity: Vec3,
    ) -> bool {
        // Check active bullets in pool
        for index in self.pool.active_bullet_indices().to_vec() {
            if let Some(bullet) = self.pool.get_bullet_mut(index) {
                if bullet.active() && bullet.entity_id() == entity_id {
                    bullet.set_velocity(velocity);
                    return true;
                }
            }
        }

        // Check active metabullets in pool
        for index in self.pool.active_metabullet_indices().to_vec() {
            if let Some(metabullet) = self.pool.get_metabullet_mut(index) {
                if metabullet.active() && metabullet.entity_id() == entity_id {
                    metabullet.set_velocity(velocity);
                    return true;
                }
            }
        }

        false
    }

    /// Remove a bullet by entity ID
    pub fn remove_bullet_by_entity_id(
        &mut self,
        entity_id: crate::engine::entity::EntityId,
    ) -> bool {
        // Check active bullets in pool
        for index in self.pool.active_bullet_indices().to_vec() {
            if let Some(bullet) = self.pool.get_bullet(index) {
                if bullet.active() && bullet.entity_id() == entity_id {
                    self.pool.release_bullet(index);
                    return true;
                }
            }
        }

        // Check active metabullets in pool
        for index in self.pool.active_metabullet_indices().to_vec() {
            if let Some(metabullet) = self.pool.get_metabullet(index) {
                if metabullet.active() && metabullet.entity_id() == entity_id {
                    self.pool.release_metabullet(index);
                    return true;
                }
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
        if self.metabullets().is_empty() {
            return;
        }

        // Find the metabullet and collect its effects to avoid borrowing conflicts
        let mut chain_events_to_add = Vec::new();
        let mut explosion_events_to_add = Vec::new();

        for metabullet in self.metabullets() {
            if metabullet.entity_id() == bullet_id {
                if let Some(ref on_hit_effects) = metabullet.on_hit() {
                    for effect in on_hit_effects.iter() {
                        // Try to downcast to ImplosionOnHitEffect first
                        if let Some(implosion_effect) = effect
                            .as_any()
                            .downcast_ref::<super::effects::ImplosionOnHitEffect>(
                        ) {
                            // This is an implosion effect - trigger explosion
                            let explosion_event =
                                implosion_effect.get_explosion_event(impact_point);
                            explosion_events_to_add.push(explosion_event);
                        } else {
                            // Regular on_hit effect (e.g., chain lightning)
                            let chain_events = effect.on_hit(impact_point, Some(target_id));
                            chain_events_to_add.extend(chain_events);
                        }
                    }
                }
                break;
            }
        }

        // Add collected chain lightning events to the event queue
        for chain_event in chain_events_to_add {
            self.event_queue
                .push(EventType::ChainLightning(chain_event));
        }

        // Add collected explosion events to the event queue
        for explosion_event in explosion_events_to_add {
            self.event_queue.push(EventType::Explosion(explosion_event));
        }
    }

    /// Mark a bullet for removal by entity ID (for collision processing)
    pub fn mark_bullet_for_removal(&mut self, entity_id: EntityId) {
        // Check active bullets in pool
        for index in self.pool.active_bullet_indices().to_vec() {
            if let Some(bullet) = self.pool.get_bullet_mut(index) {
                if bullet.active() && bullet.entity_id() == entity_id {
                    bullet.mark_for_removal();
                    return;
                }
            }
        }

        // Check active metabullets in pool
        for index in self.pool.active_metabullet_indices().to_vec() {
            if let Some(metabullet) = self.pool.get_metabullet_mut(index) {
                if metabullet.active() && metabullet.entity_id() == entity_id {
                    metabullet.mark_for_removal();
                    return;
                }
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
                // Acquire bullet from pool for maximum performance
                self.pool.acquire_bullet(
                    entity_id, position, velocity, lifetime, damage, mass, visuals,
                );
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

                // Acquire MetaBullet from pool for maximum performance
                self.pool.acquire_metabullet(
                    entity_id,
                    position,
                    velocity,
                    lifetime,
                    damage,
                    mass,
                    on_hit_effects,
                    on_expire_effects,
                    visuals,
                );
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
                    crate::scene::explosion::FalloffType::Linear,
                    damage,           // Add damage parameter
                    explosion_radius, // Use explosion_radius for damage_radius too
                    Color::ORANGE,
                    Color::ORANGE,
                    50,
                );

                let on_expire_effects: Option<Vec<Box<dyn OnExpireEffect>>> =
                    Some(vec![Box::new(explosion_effect)]);

                // Acquire seeking MetaBullet from pool with explosion effect
                self.pool.acquire_seeking_metabullet(
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
                );
            }
            ProjectileType::ImplosionExplosive {
                damage,
                velocity,
                lifetime,
                mass,
                explosion_radius,
                explosion_force,
                explosion_duration,
                visuals,
            } => {
                // Create implosion effect (negative force) for BOTH hit and expire
                let implosion_effect = ExplosionEffect::new(
                    explosion_radius,
                    explosion_force, // Negative value for implosion!
                    explosion_duration,
                    crate::scene::explosion::FalloffType::Linear,
                    damage,
                    explosion_radius,
                    Color::PURPLE,
                    Color::MAGENTA,
                    50,
                );

                // Clone the effect for both on_hit and on_expire
                let on_hit_effects: Option<Vec<Box<dyn super::effects::OnHitEffect>>> =
                    Some(vec![Box::new(super::effects::ImplosionOnHitEffect {
                        explosion_effect: implosion_effect.clone(),
                    })]);

                let on_expire_effects: Option<Vec<Box<dyn OnExpireEffect>>> =
                    Some(vec![Box::new(implosion_effect)]);

                // Acquire regular MetaBullet from pool with implosion effect on both hit and expire
                self.pool.acquire_metabullet(
                    entity_id,
                    position,
                    velocity,
                    lifetime,
                    damage,
                    mass,
                    on_hit_effects,
                    on_expire_effects,
                    visuals,
                );
            }
            ProjectileType::Fractal {
                damage,
                velocity,
                lifetime,
                mass,
                fractal_config,
                visuals,
            } => {
                // Acquire fractal bullet from pool for maximum performance
                self.pool.acquire_fractal_bullet(
                    entity_id,
                    position,
                    velocity,
                    lifetime,
                    damage,
                    mass,
                    visuals,
                    fractal_config,
                    0, // Generation 0 (parent)
                );
            }
            ProjectileType::Laser {
                damage,
                velocity,
                lifetime,
                mass,
                max_trail_length,
                trail_fade_rate,
                visuals,
            } => {
                // Acquire laser bullet from pool for maximum performance
                self.pool.acquire_laser_bullet(
                    entity_id,
                    position,
                    velocity,
                    lifetime,
                    damage,
                    mass,
                    visuals,
                    max_trail_length,
                    trail_fade_rate,
                );
            }
        }
    }

    /// Update fractal bullets and handle splitting with batch processing for performance
    fn handle_fractal_splitting(&mut self, dt: f32) -> Vec<crate::engine::entity::EntityId> {
        // STEP 1: Find all active bullets ready to split and add new ones to pending queue
        self.pending_split_indices.retain(|&index| {
            // Keep only valid indices that point to active bullets
            if let Some(bullet) = self.pool.get_bullet(index) {
                bullet.active()
            } else {
                false
            }
        });

        // Check all active bullets for split readiness
        for &index in self.pool.active_bullet_indices() {
            if let Some(bullet) = self.pool.get_bullet(index) {
                if bullet.active()
                    && bullet.should_split(dt)
                    && !self.pending_split_indices.contains(&index)
                {
                    self.pending_split_indices.push(index);
                }
            }
        }

        // STEP 2: Process only a batch of splits this frame for smooth performance
        let splits_to_process = self
            .fractal_split_batch_size
            .min(self.pending_split_indices.len());
        if splits_to_process == 0 {
            return Vec::new();
        }

        // Pre-calculate total children needed and batch acquire from pool
        let mut total_children_needed = 0;
        let split_indices: Vec<usize> = self
            .pending_split_indices
            .drain(..splits_to_process)
            .collect();

        // Count children needed for batch pool acquisition
        for &index in &split_indices {
            if let Some(bullet) = self.pool.get_bullet(index) {
                if bullet.should_split(dt) {
                    total_children_needed += bullet.get_split_directions().len();
                }
            }
        }

        // STEP 3: Batch acquire bullets from pool for maximum performance
        let available_bullet_indices = self
            .pool
            .batch_acquire_fractal_bullets(total_children_needed);
        let mut acquired_index = 0;
        let mut new_entity_ids = Vec::with_capacity(total_children_needed);

        // Process the batch of splits using pre-acquired bullets
        for &parent_index in &split_indices {
            if let Some(parent_bullet) = self.pool.get_bullet(parent_index) {
                if parent_bullet.should_split(dt) {
                    // Generate children based on fractal pattern
                    let split_directions = parent_bullet.get_split_directions();
                    let fractal_data = parent_bullet.fractal_data().cloned();

                    if let Some(fractal_data) = fractal_data {
                        // Pre-allocate entity IDs in batch for this bullet's children
                        let child_count = split_directions.len();
                        let entity_id_start = self.next_entity_id;
                        self.next_entity_id += child_count as u32;

                        // Create all children using pre-acquired pool bullets
                        for (i, direction) in split_directions.into_iter().enumerate() {
                            if acquired_index < available_bullet_indices.len() {
                                let child_index = available_bullet_indices[acquired_index];
                                let child_entity_id = EntityId(entity_id_start + i as u32);

                                // Use parent bullet data to create child
                                if let Some(parent) = self.pool.get_bullet(parent_index) {
                                    if let Some(child) =
                                        parent.create_fractal_child(child_entity_id, direction)
                                    {
                                        // Reset the pre-acquired bullet as a fractal child
                                        if let Some(pool_bullet) =
                                            self.pool.get_bullet_mut(child_index)
                                        {
                                            pool_bullet.reset_fractal(
                                                child.entity_id(),
                                                child.position(),
                                                child.velocity(),
                                                child.ttl(),
                                                child.damage(),
                                                child.mass(),
                                                child.visuals().clone(),
                                                fractal_data.config.clone(),
                                                fractal_data.generation + 1,
                                            );
                                        }

                                        // CRITICAL: Add the bullet to the active list
                                        if !self.pool.active_bullet_indices().contains(&child_index)
                                        {
                                            self.pool.add_to_active_bullets(child_index);
                                        }
                                        new_entity_ids.push(child_entity_id);
                                    }
                                }
                                acquired_index += 1;
                            }
                        }
                    }

                    // Mark parent as having split
                    if let Some(parent_bullet) = self.pool.get_bullet_mut(parent_index) {
                        parent_bullet.mark_split();
                    }
                }
            }
        }

        // Debug info for monitoring performance
        if !new_entity_ids.is_empty() {
            println!(
                "🌸 Fractal batch: {} bullets split into {} children ({} pending) - Pool Performance!",
                splits_to_process,
                new_entity_ids.len(),
                self.pending_split_indices.len()
            );
        }

        new_entity_ids
    }

    /// Get fractal bullet count for debugging
    pub fn fractal_bullet_count(&self) -> usize {
        let mut count = 0;
        for &index in self.pool.active_bullet_indices() {
            if let Some(bullet) = self.pool.get_bullet(index) {
                if bullet.active() && bullet.fractal_data().is_some() {
                    count += 1;
                }
            }
        }
        count
    }

    /// Get pending split count for performance monitoring
    pub fn pending_split_count(&self) -> usize {
        self.pending_split_indices.len()
    }

    /// Adjust fractal split batch size for performance tuning
    pub fn set_fractal_batch_size(&mut self, size: usize) {
        self.fractal_split_batch_size = size.max(1); // At least 1 to prevent deadlock
        println!(
            "🔧 Fractal batch size set to: {}",
            self.fractal_split_batch_size
        );
    }

    /// Get line instances for laser trail rendering
    pub fn get_laser_trail_lines(&self) -> Vec<LineInstance> {
        let mut line_instances = Vec::new();

        for &index in self.pool.active_bullet_indices() {
            if let Some(bullet) = self.pool.get_bullet(index) {
                if !bullet.active() {
                    continue;
                }
                if let Some(trail_data) = bullet.trail_data() {
                    let trail_points = &trail_data.trail_points;

                    // Need at least 2 points to make a line
                    if trail_points.len() < 2 {
                        continue;
                    }

                    // Create line segments between consecutive trail points
                    for i in 0..trail_points.len() - 1 {
                        let start_pos = trail_points[i];
                        let end_pos = trail_points[i + 1];

                        // Calculate fade factor based on position in trail (newer = brighter)
                        // i=0 is oldest (back), i=len-1 is newest (front)
                        // We want: newest=1.0 (bright), oldest=faded
                        let trail_progress = i as f32 / (trail_points.len() - 1) as f32;
                        // Reverse the progress so newer segments are brighter
                        let alpha = 1.0 - ((1.0 - trail_progress) * trail_data.trail_fade_rate);
                        let alpha = alpha.max(0.1); // Minimum visibility

                        // Use bullet's color with fading alpha
                        let bullet_color = bullet.visuals().color;
                        let line_color = [
                            bullet_color.r,
                            bullet_color.g,
                            bullet_color.b,
                            bullet_color.a * alpha,
                        ];

                        line_instances.push(LineInstance {
                            start_pos: [start_pos.x, start_pos.y, start_pos.z],
                            end_pos: [end_pos.x, end_pos.y, end_pos.z],
                            thickness: 0.02, // Thin laser beam
                            color: line_color,
                        });
                    }
                }
            }
        }

        line_instances
    }
}
