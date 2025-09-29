use super::types::{Bullet, MetaBullet};
use super::effects::{OnExpireEffect, OnHitEffect};
use super::visuals::BulletVisuals;
use crate::engine::{EntityId, Vec3};

/// Bullet pool for high-performance bullet management
pub struct BulletPool {
    // Regular bullet pool
    bullets: Vec<Bullet>,
    active_bullets: Vec<usize>,    // Indices of active bullets
    inactive_bullet_stack: Vec<usize>, // Stack of available bullet indices

    // MetaBullet pool
    metabullets: Vec<MetaBullet>,
    active_metabullets: Vec<usize>,    // Indices of active metabullets
    inactive_metabullet_stack: Vec<usize>, // Stack of available metabullet indices

    // Pool statistics
    max_bullets_used: usize,
    max_metabullets_used: usize,
    total_bullets_spawned: u64,
    total_metabullets_spawned: u64,
}

impl BulletPool {
    /// Create a new bullet pool with pre-allocated capacity
    pub fn new(bullet_capacity: usize, metabullet_capacity: usize) -> Self {
        let mut pool = BulletPool {
            bullets: Vec::with_capacity(bullet_capacity),
            active_bullets: Vec::with_capacity(bullet_capacity),
            inactive_bullet_stack: Vec::with_capacity(bullet_capacity),

            metabullets: Vec::with_capacity(metabullet_capacity),
            active_metabullets: Vec::with_capacity(metabullet_capacity),
            inactive_metabullet_stack: Vec::with_capacity(metabullet_capacity),

            max_bullets_used: 0,
            max_metabullets_used: 0,
            total_bullets_spawned: 0,
            total_metabullets_spawned: 0,
        };

        // Pre-allocate all bullets as inactive
        for i in 0..bullet_capacity {
            pool.bullets.push(Bullet::new(
                EntityId(0),
                Vec3::zeros(),
                Vec3::zeros(),
                0.0,
                0.0,
                0.0,
                BulletVisuals::basic_blaster(),
            ));
            pool.bullets[i].deactivate(); // Mark as inactive
            pool.inactive_bullet_stack.push(i);
        }

        // Pre-allocate all metabullets as inactive
        for i in 0..metabullet_capacity {
            pool.metabullets.push(MetaBullet::new(
                EntityId(0),
                Vec3::zeros(),
                Vec3::zeros(),
                0.0,
                0.0,
                0.0,
                None,
                None,
                BulletVisuals::basic_blaster(),
            ));
            pool.metabullets[i].deactivate(); // Mark as inactive
            pool.inactive_metabullet_stack.push(i);
        }

        println!(
            "🏊 Bullet pool initialized: {} bullets, {} metabullets",
            bullet_capacity, metabullet_capacity
        );

        pool
    }

    /// Get a bullet from the pool
    pub fn acquire_bullet(
        &mut self,
        entity_id: EntityId,
        pos: Vec3,
        vel: Vec3,
        ttl: f32,
        damage: f32,
        mass: f32,
        visuals: BulletVisuals,
    ) -> Option<usize> {
        if let Some(index) = self.inactive_bullet_stack.pop() {
            // Reuse existing bullet
            self.bullets[index].reset(entity_id, pos, vel, ttl, damage, mass, visuals);
            self.active_bullets.push(index);
            self.total_bullets_spawned += 1;
            self.max_bullets_used = self.max_bullets_used.max(self.active_bullets.len());
            Some(index)
        } else {
            // Pool exhausted - need to grow
            self.grow_bullet_pool();
            self.acquire_bullet(entity_id, pos, vel, ttl, damage, mass, visuals)
        }
    }

    /// Get a fractal bullet from the pool
    pub fn acquire_fractal_bullet(
        &mut self,
        entity_id: EntityId,
        pos: Vec3,
        vel: Vec3,
        ttl: f32,
        damage: f32,
        mass: f32,
        visuals: BulletVisuals,
        fractal_config: super::fractal::FractalConfig,
        generation: usize,
    ) -> Option<usize> {
        if let Some(index) = self.inactive_bullet_stack.pop() {
            // Reuse existing bullet
            self.bullets[index].reset_fractal(
                entity_id, pos, vel, ttl, damage, mass, visuals, fractal_config, generation
            );
            self.active_bullets.push(index);
            self.total_bullets_spawned += 1;
            self.max_bullets_used = self.max_bullets_used.max(self.active_bullets.len());
            Some(index)
        } else {
            // Pool exhausted - need to grow
            self.grow_bullet_pool();
            self.acquire_fractal_bullet(entity_id, pos, vel, ttl, damage, mass, visuals, fractal_config, generation)
        }
    }

    /// Get a laser bullet from the pool
    pub fn acquire_laser_bullet(
        &mut self,
        entity_id: EntityId,
        pos: Vec3,
        vel: Vec3,
        ttl: f32,
        damage: f32,
        mass: f32,
        visuals: BulletVisuals,
        max_trail_length: usize,
        trail_fade_rate: f32,
    ) -> Option<usize> {
        if let Some(index) = self.inactive_bullet_stack.pop() {
            // Reuse existing bullet
            self.bullets[index].reset_laser(
                entity_id, pos, vel, ttl, damage, mass, visuals, max_trail_length, trail_fade_rate
            );
            self.active_bullets.push(index);
            self.total_bullets_spawned += 1;
            self.max_bullets_used = self.max_bullets_used.max(self.active_bullets.len());
            Some(index)
        } else {
            // Pool exhausted - need to grow
            self.grow_bullet_pool();
            self.acquire_laser_bullet(entity_id, pos, vel, ttl, damage, mass, visuals, max_trail_length, trail_fade_rate)
        }
    }

    /// Get a metabullet from the pool
    pub fn acquire_metabullet(
        &mut self,
        entity_id: EntityId,
        pos: Vec3,
        vel: Vec3,
        ttl: f32,
        damage: f32,
        mass: f32,
        on_hit: Option<Vec<Box<dyn OnHitEffect>>>,
        on_expire: Option<Vec<Box<dyn OnExpireEffect>>>,
        visuals: BulletVisuals,
    ) -> Option<usize> {
        if let Some(index) = self.inactive_metabullet_stack.pop() {
            // Reuse existing metabullet
            self.metabullets[index].reset(entity_id, pos, vel, ttl, damage, mass, on_hit, on_expire, visuals);
            self.active_metabullets.push(index);
            self.total_metabullets_spawned += 1;
            self.max_metabullets_used = self.max_metabullets_used.max(self.active_metabullets.len());
            Some(index)
        } else {
            // Pool exhausted - need to grow
            self.grow_metabullet_pool();
            self.acquire_metabullet(entity_id, pos, vel, ttl, damage, mass, on_hit, on_expire, visuals)
        }
    }

    /// Get a seeking metabullet from the pool
    pub fn acquire_seeking_metabullet(
        &mut self,
        entity_id: EntityId,
        pos: Vec3,
        vel: Vec3,
        ttl: f32,
        damage: f32,
        mass: f32,
        on_hit: Option<Vec<Box<dyn OnHitEffect>>>,
        on_expire: Option<Vec<Box<dyn OnExpireEffect>>>,
        seeking_force: f32,
        max_seeking_range: f32,
        visuals: BulletVisuals,
    ) -> Option<usize> {
        if let Some(index) = self.inactive_metabullet_stack.pop() {
            // Reuse existing metabullet
            self.metabullets[index].reset_seeking(
                entity_id, pos, vel, ttl, damage, mass, on_hit, on_expire, seeking_force, max_seeking_range, visuals
            );
            self.active_metabullets.push(index);
            self.total_metabullets_spawned += 1;
            self.max_metabullets_used = self.max_metabullets_used.max(self.active_metabullets.len());
            Some(index)
        } else {
            // Pool exhausted - need to grow
            self.grow_metabullet_pool();
            self.acquire_seeking_metabullet(entity_id, pos, vel, ttl, damage, mass, on_hit, on_expire, seeking_force, max_seeking_range, visuals)
        }
    }

    /// Batch acquire multiple bullets for fractal splitting performance
    pub fn batch_acquire_fractal_bullets(
        &mut self,
        count: usize,
    ) -> Vec<usize> {
        let mut indices = Vec::with_capacity(count);

        // Ensure we have enough bullets available
        while self.inactive_bullet_stack.len() < count {
            self.grow_bullet_pool();
        }

        // Batch acquire all at once
        for _ in 0..count {
            if let Some(index) = self.inactive_bullet_stack.pop() {
                indices.push(index);
            }
        }

        indices
    }

    /// Return a bullet to the pool
    pub fn release_bullet(&mut self, index: usize) {
        if index < self.bullets.len() && self.bullets[index].active() {
            // Deactivate the bullet
            self.bullets[index].deactivate();

            // Remove from active list
            if let Some(pos) = self.active_bullets.iter().position(|&x| x == index) {
                self.active_bullets.swap_remove(pos);
            }

            // Return to inactive stack
            self.inactive_bullet_stack.push(index);
        }
    }

    /// Return a metabullet to the pool
    pub fn release_metabullet(&mut self, index: usize) {
        if index < self.metabullets.len() && self.metabullets[index].active() {
            // Deactivate the metabullet
            self.metabullets[index].deactivate();

            // Remove from active list
            if let Some(pos) = self.active_metabullets.iter().position(|&x| x == index) {
                self.active_metabullets.swap_remove(pos);
            }

            // Return to inactive stack
            self.inactive_metabullet_stack.push(index);
        }
    }

    /// Get active bullets - WARNING: Returns only active bullets via Vec (expensive)
    /// Consider using active_bullet_indices() and get_bullet() instead for better performance
    pub fn active_bullets(&self) -> Vec<&Bullet> {
        self.active_bullets.iter()
            .filter_map(|&index| self.bullets.get(index))
            .filter(|bullet| bullet.active())
            .collect()
    }

    /// Get active bullets mutably - WARNING: Cannot safely return mutable references
    /// Use active_bullet_indices() and get_bullet_mut() instead
    pub fn active_bullets_mut(&mut self) -> Vec<&mut Bullet> {
        // This is fundamentally unsafe with the pool design
        // Use active_bullet_indices() and get_bullet_mut() instead
        Vec::new()
    }

    /// Get active metabullets - WARNING: Returns only active metabullets via Vec (expensive)
    /// Consider using active_metabullet_indices() and get_metabullet() instead for better performance
    pub fn active_metabullets(&self) -> Vec<&MetaBullet> {
        self.active_metabullets.iter()
            .filter_map(|&index| self.metabullets.get(index))
            .filter(|metabullet| metabullet.active())
            .collect()
    }

    /// Get active metabullets mutably - WARNING: Cannot safely return mutable references
    /// Use active_metabullet_indices() and get_metabullet_mut() instead
    pub fn active_metabullets_mut(&mut self) -> Vec<&mut MetaBullet> {
        // This is fundamentally unsafe with the pool design
        // Use active_metabullet_indices() and get_metabullet_mut() instead
        Vec::new()
    }

    /// Get active bullet indices
    pub fn active_bullet_indices(&self) -> &[usize] {
        &self.active_bullets
    }

    /// Get active metabullet indices
    pub fn active_metabullet_indices(&self) -> &[usize] {
        &self.active_metabullets
    }

    /// Get bullets as slice for graphics/collision systems (ONLY ACTIVE BULLETS)
    /// WARNING: This is expensive and returns all bullets, filtering done by caller
    /// For performance, use active_bullet_indices() and get_bullet() instead
    pub fn bullets_slice(&self) -> &[Bullet] {
        &self.bullets
    }

    /// Get metabullets as slice for graphics/collision systems (ONLY ACTIVE METABULLETS)
    /// WARNING: This is expensive and returns all metabullets, filtering done by caller
    /// For performance, use active_metabullet_indices() and get_metabullet() instead
    pub fn metabullets_slice(&self) -> &[MetaBullet] {
        &self.metabullets
    }

    /// Get bullet by index
    pub fn get_bullet(&self, index: usize) -> Option<&Bullet> {
        self.bullets.get(index)
    }

    /// Get bullet by index mutably
    pub fn get_bullet_mut(&mut self, index: usize) -> Option<&mut Bullet> {
        self.bullets.get_mut(index)
    }

    /// Get metabullet by index
    pub fn get_metabullet(&self, index: usize) -> Option<&MetaBullet> {
        self.metabullets.get(index)
    }

    /// Get metabullet by index mutably
    pub fn get_metabullet_mut(&mut self, index: usize) -> Option<&mut MetaBullet> {
        self.metabullets.get_mut(index)
    }

    /// Manually add a bullet to active list (for batch processing)
    pub fn add_to_active_bullets(&mut self, index: usize) {
        if index < self.bullets.len() && !self.active_bullets.contains(&index) {
            self.active_bullets.push(index);
            self.max_bullets_used = self.max_bullets_used.max(self.active_bullets.len());
        }
    }

    /// Manually add a metabullet to active list (for batch processing)
    pub fn add_to_active_metabullets(&mut self, index: usize) {
        if index < self.metabullets.len() && !self.active_metabullets.contains(&index) {
            self.active_metabullets.push(index);
            self.max_metabullets_used = self.max_metabullets_used.max(self.active_metabullets.len());
        }
    }

    /// Collect active bullets for physics (UNSAFE - must not have overlapping access)
    pub unsafe fn collect_active_bullets_mut_unsafe(&mut self) -> Vec<&mut Bullet> {
        let active_indices = self.active_bullets.clone();
        let bullets_ptr = self.bullets.as_mut_ptr();
        let mut result = Vec::new();

        for index in active_indices {
            if index < self.bullets.len() {
                unsafe {
                    let bullet = &mut *bullets_ptr.add(index);
                    if bullet.active() {
                        result.push(bullet);
                    }
                }
            }
        }

        result
    }

    /// Cleanup dead bullets and return them to pool
    pub fn cleanup_dead_bullets(&mut self) -> Vec<EntityId> {
        let mut destroyed_entities = Vec::new();

        // Process bullets - iterate backwards to avoid index shifting
        let mut i = 0;
        while i < self.active_bullets.len() {
            let index = self.active_bullets[i];
            if !self.bullets[index].is_alive() {
                destroyed_entities.push(self.bullets[index].entity_id());
                self.release_bullet(index);
                // Don't increment i since we removed an element
            } else {
                i += 1;
            }
        }

        // Process metabullets - iterate backwards to avoid index shifting
        let mut i = 0;
        while i < self.active_metabullets.len() {
            let index = self.active_metabullets[i];
            if !self.metabullets[index].is_alive() {
                destroyed_entities.push(self.metabullets[index].entity_id());
                self.release_metabullet(index);
                // Don't increment i since we removed an element
            } else {
                i += 1;
            }
        }

        destroyed_entities
    }

    /// Grow the bullet pool when exhausted
    fn grow_bullet_pool(&mut self) {
        let current_size = self.bullets.len();
        let new_size = (current_size * 2).max(current_size + 1000); // Double or add 1000, whichever is larger

        println!(
            "🚀 Growing bullet pool from {} to {} (+{})",
            current_size, new_size, new_size - current_size
        );

        for i in current_size..new_size {
            self.bullets.push(Bullet::new(
                EntityId(0),
                Vec3::zeros(),
                Vec3::zeros(),
                0.0,
                0.0,
                0.0,
                BulletVisuals::basic_blaster(),
            ));
            self.bullets[i].deactivate();
            self.inactive_bullet_stack.push(i);
        }
    }

    /// Grow the metabullet pool when exhausted
    fn grow_metabullet_pool(&mut self) {
        let current_size = self.metabullets.len();
        let new_size = (current_size * 2).max(current_size + 500); // Double or add 500, whichever is larger

        println!(
            "🚀 Growing metabullet pool from {} to {} (+{})",
            current_size, new_size, new_size - current_size
        );

        for i in current_size..new_size {
            self.metabullets.push(MetaBullet::new(
                EntityId(0),
                Vec3::zeros(),
                Vec3::zeros(),
                0.0,
                0.0,
                0.0,
                None,
                None,
                BulletVisuals::basic_blaster(),
            ));
            self.metabullets[i].deactivate();
            self.inactive_metabullet_stack.push(i);
        }
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            active_bullets: self.active_bullets.len(),
            total_bullets: self.bullets.len(),
            active_metabullets: self.active_metabullets.len(),
            total_metabullets: self.metabullets.len(),
            max_bullets_used: self.max_bullets_used,
            max_metabullets_used: self.max_metabullets_used,
            total_bullets_spawned: self.total_bullets_spawned,
            total_metabullets_spawned: self.total_metabullets_spawned,
            bullet_pool_utilization: self.active_bullets.len() as f32 / self.bullets.len() as f32,
            metabullet_pool_utilization: self.active_metabullets.len() as f32 / self.metabullets.len() as f32,
        }
    }

    /// Print pool statistics
    pub fn print_stats(&self) {
        let stats = self.stats();
        println!(
            "🏊 Pool Stats: Bullets {}/{} ({:.1}%), MetaBullets {}/{} ({:.1}%), Max Used: {}/{}, Total Spawned: {}/{}",
            stats.active_bullets,
            stats.total_bullets,
            stats.bullet_pool_utilization * 100.0,
            stats.active_metabullets,
            stats.total_metabullets,
            stats.metabullet_pool_utilization * 100.0,
            stats.max_bullets_used,
            stats.max_metabullets_used,
            stats.total_bullets_spawned,
            stats.total_metabullets_spawned
        );
    }
}

/// Pool statistics for monitoring performance
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub active_bullets: usize,
    pub total_bullets: usize,
    pub active_metabullets: usize,
    pub total_metabullets: usize,
    pub max_bullets_used: usize,
    pub max_metabullets_used: usize,
    pub total_bullets_spawned: u64,
    pub total_metabullets_spawned: u64,
    pub bullet_pool_utilization: f32,
    pub metabullet_pool_utilization: f32,
}