use crate::engine::entity::{EntityLookup, EntityManager, EntityType};
use crate::graphics::{Primitive, Vec3};
use crate::input::InputManager;
use crate::scene::{
    BulletManager, EnemyManager, ExplosionManager, GravityAffected, LargeBodyManager,
    LargeBodyType, PhysicsManager, PlayerManager,
};

pub struct Scheduler {
    entity_manager: EntityManager,
    player: PlayerManager,
    enemies: EnemyManager,
    bullets: BulletManager,
    physics: PhysicsManager,
    large_bodies: LargeBodyManager,
    explosions: ExplosionManager,
}

impl Scheduler {
    pub fn new() -> Self {
        let mut entity_manager = EntityManager::new();

        // Create player entity
        let player_entity = entity_manager.create_entity(EntityType::Player);

        let mut enemies = EnemyManager::new();
        enemies.spawn_initial_enemies(&mut entity_manager);

        let mut physics = PhysicsManager::new();
        let mut large_bodies = LargeBodyManager::new();

        large_bodies.spawn_body(
            LargeBodyType::BlackHole,
            Vec3::new(50.0, 10.0, 50.0),
            &mut physics,
            &mut entity_manager,
        );

        // large_bodies.spawn_binary_pair(
        //     crate::scene::large_body::LargeBodyType::BlackHole,
        //     crate::scene::large_body::LargeBodyType::WhiteHole,
        //     crate::engine::Vec3::new(0.0, 0.0, 0.0),
        //     100.0, // Separation distance
        //     &mut physics,
        //     &mut entity_manager,
        // );
        // large_bodies.spawn_binary_pair(
        //     crate::scene::large_body::LargeBodyType::ExoticMatter,
        //     crate::scene::large_body::LargeBodyType::BlackHole,
        //     crate::engine::Vec3::new(0.0, 0.0, 0.0),
        //     100.0, // Separation distance
        //     &mut physics,
        //     &mut entity_manager,
        // );

        Scheduler {
            entity_manager,
            player: PlayerManager::new(player_entity),
            enemies,
            bullets: BulletManager::new(),
            physics,
            large_bodies,
            explosions: ExplosionManager::new(),
        }
    }

    pub fn preupdate(&mut self) {
        // Pre-update phase - prepare for frame
    }

    pub fn update(
        &mut self,
        delta_time: f32,
        input: &InputManager,
        camera_forward: Vec3,
        camera_right: Vec3,
        camera_up: Vec3,
    ) {
        self.update_with_physics(
            delta_time,
            input,
            camera_forward,
            camera_right,
            camera_up,
            None,
            None,
        );
    }

    pub fn update_with_physics(
        &mut self,
        delta_time: f32,
        input: &InputManager,
        camera_forward: Vec3,
        camera_right: Vec3,
        camera_up: Vec3,
        device: Option<&wgpu::Device>,
        queue: Option<&wgpu::Queue>,
    ) {
        // Update player movement
        if let Some(bullet_requests) =
            self.player
                .update(delta_time, input, camera_forward, camera_right, camera_up)
        {
            // Spawn bullets from player weapon using new projectile system
            for request in bullet_requests.iter() {
                let bullet_entity = self.entity_manager.create_entity(EntityType::PlayerBullet);

                // Create basic projectile type from request
                let velocity = request.direction * request.speed;
                let projectile_type = crate::scene::bullet::ProjectileType::Basic {
                    damage: request.damage,
                    velocity,
                    lifetime: request.lifetime,
                    mass: request.mass,
                };

                self.bullets
                    .spawn_projectile(bullet_entity, request.position, projectile_type);
            }
        }

        self.enemies.update(
            delta_time,
            self.player.player().position(),
            &mut self.entity_manager,
        );
        self.bullets.update(delta_time);
        self.explosions.update(delta_time);

        // Update large bodies (these get their physics from the PhysicsManager N-body simulation)
        self.large_bodies.update(delta_time, &self.physics);

        // Apply gravitational forces to all affected objects
        if let (Some(device), Some(queue)) = (device, queue) {
            // Update physics using the helper method that handles the complexity
            self.update_physics_forces(device, queue, delta_time);
        }
    }

    pub fn postupdate(&mut self) {
        // Post-update phase - clean up dead entities efficiently

        // Each manager handles its own cleanup and returns destroyed entity IDs
        let destroyed_bullets = self.bullets.cleanup_dead_bullets();
        let destroyed_enemies = self.enemies.cleanup_dead_enemies();

        // Destroy entities from the entity manager
        for entity_id in destroyed_bullets
            .into_iter()
            .chain(destroyed_enemies.into_iter())
        {
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
        primitives.extend(self.large_bodies.get_render_data());
        primitives.extend(self.explosions.get_render_data());

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
            &self.large_bodies,
        )
    }

    /// Get access to the player manager
    pub fn player(&self) -> &PlayerManager {
        &self.player
    }

    /// Get mutable access to the player manager
    pub fn player_mut(&mut self) -> &mut PlayerManager {
        &mut self.player
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

    /// Get access to the physics manager
    pub fn physics(&self) -> &PhysicsManager {
        &self.physics
    }

    /// Get mutable access to the physics manager
    pub fn physics_mut(&mut self) -> &mut PhysicsManager {
        &mut self.physics
    }

    /// Get access to the large body manager
    pub fn large_bodies(&self) -> &LargeBodyManager {
        &self.large_bodies
    }

    /// Get mutable access to the large body manager
    pub fn large_bodies_mut(&mut self) -> &mut LargeBodyManager {
        &mut self.large_bodies
    }

    /// Get access to the explosion manager
    pub fn explosions(&self) -> &ExplosionManager {
        &self.explosions
    }

    /// Get mutable access to the explosion manager
    pub fn explosions_mut(&mut self) -> &mut ExplosionManager {
        &mut self.explosions
    }

    /// Initialize GPU physics resources (call after graphics context is available)
    pub fn initialize_physics_gpu(&mut self, device: &wgpu::Device) {
        self.physics.initialize_gpu(device);
    }

    /// Update physics forces for all gravity-affected objects in one efficient GPU call
    fn update_physics_forces(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        delta_time: f32,
    ) {
        // Collect ALL affected objects into a single batch for efficient GPU processing
        let mut all_objects: Vec<&mut dyn GravityAffected> = Vec::new();

        // Add player
        all_objects.push(self.player.player_mut());

        // Add all enemies
        for enemy in self.enemies.enemies_mut().iter_mut() {
            all_objects.push(enemy);
        }

        // Add all bullets
        for bullet in self.bullets.bullets_mut().iter_mut() {
            all_objects.push(bullet);
        }

        // Single GPU call for all objects - much more efficient!
        if !all_objects.is_empty() {
            self.physics.update_gravity_batch(
                device,
                queue,
                &mut all_objects,
                self.explosions.explosions(),
                delta_time,
            );
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
