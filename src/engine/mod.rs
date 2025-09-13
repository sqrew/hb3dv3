pub mod collision;
pub mod dispatcher;
pub mod entity;
pub mod math;
pub mod scheduler;
pub mod time;
pub mod window;

pub use collision::*;
pub use dispatcher::*;
pub use entity::*;
pub use math::*;
pub use scheduler::*;
pub use time::*;
pub use window::*;

pub struct Engine {
    dispatcher: Dispatcher,
    scheduler: Scheduler,
    collision_manager: CollisionManager,
    time: TimeManager,
    fps_display_timer: f32,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            dispatcher: Dispatcher::new(),
            scheduler: Scheduler::new(),
            collision_manager: CollisionManager::new(),
            time: TimeManager::new(),
            fps_display_timer: 0.0,
        }
    }
    
    // Single update method that handles everything
    pub fn update(&mut self, delta_time: f32, input: &crate::input::InputManager, camera_forward: Vec3, camera_right: Vec3, camera_up: Vec3) -> (Vec<crate::graphics::Primitive>, Vec3, Vec<dispatcher::GraphicsEvent>) {
        self.update_with_gpu(delta_time, input, camera_forward, camera_right, camera_up, None, None)
    }
    
    // Update method with optional GPU context for physics
    pub fn update_with_gpu(&mut self, delta_time: f32, input: &crate::input::InputManager, camera_forward: Vec3, camera_right: Vec3, camera_up: Vec3, device: Option<&wgpu::Device>, queue: Option<&wgpu::Queue>) -> (Vec<crate::graphics::Primitive>, Vec3, Vec<dispatcher::GraphicsEvent>) {
        // Update time tracking
        self.time.update(delta_time);
        
        // Display FPS every second
        self.fps_display_timer += delta_time;
        if self.fps_display_timer >= 1.0 {
            println!("FPS: {:.1} | Frame time: {:.2}ms", self.time.fps(), delta_time * 1000.0);
            self.fps_display_timer = 0.0;
        }
        
        // Process queued events from previous frame
        let mut graphics_events = Vec::new();
        self.dispatcher.process_events(&mut self.scheduler, &mut graphics_events);
        
        // Pre-update
        self.scheduler.preupdate();
        
        // Main update with physics support
        self.scheduler.update_with_physics(delta_time, input, camera_forward, camera_right, camera_up, device, queue);
        
        // Post-update
        self.scheduler.postupdate();
        
        // Collect events from managers
        let player_events = self.scheduler.player_mut().drain_events(); // Collect player and weapon events
        let enemy_events = self.scheduler.enemies_mut().drain_events(); // Collect events from EnemyManager
        let bullet_events = self.scheduler.bullets_mut().drain_events(); // Collect events from BulletManager
        let collision_events = self.collision_manager.drain_events(); // Collect events from CollisionManager
        
        // Collect all events for next frame
        self.dispatcher.collect_events(player_events, enemy_events, bullet_events, collision_events);
        
        // Prepare events for next frame
        self.dispatcher.prepare_next_frame();
        
        // Pre-render
        self.scheduler.prerender();
        
        // Get render data and player position
        let primitives = self.scheduler.get_render_data();
        let player_pos = self.scheduler.get_player_position();
        
        // Post-render
        self.scheduler.postrender();
        
        (primitives, player_pos, graphics_events)
    }
    
    /// Get mutable access to the collision manager
    pub fn collision_manager_mut(&mut self) -> &mut CollisionManager {
        &mut self.collision_manager
    }
    
    /// Get access to the scheduler (for collision processing)
    pub fn scheduler(&self) -> &Scheduler {
        &self.scheduler
    }
    
    /// Process collision pairs using the collision manager
    pub fn process_collisions(&mut self, collision_pairs: &[(u32, u32)]) {
        self.collision_manager.process_collision_pairs(
            collision_pairs,
            &self.scheduler.bullets(),
            &self.scheduler.enemies(),
            &self.scheduler.entity_manager(),
        );
        
        // Process large body vs large body collisions (CPU-based)
        self.collision_manager.process_large_body_collisions(
            &self.scheduler.large_bodies(),
        );
        
        // Handle bullet removals from large body collisions
        let bullets_to_remove = self.collision_manager.drain_bullets_to_remove();
        for bullet_id in bullets_to_remove {
            self.scheduler.bullets_mut().mark_bullet_for_removal(bullet_id);
        }
        
        // Handle enemy removals from large body collisions
        let enemies_to_remove = self.collision_manager.drain_enemies_to_remove();
        for enemy_id in enemies_to_remove {
            self.scheduler.enemies_mut().mark_enemy_for_removal(enemy_id);
        }
        
        // Handle bullet velocity changes from bullet-bullet collisions
        let velocity_changes = self.collision_manager.drain_bullet_velocity_changes();
        for (bullet_id, new_velocity) in velocity_changes {
            self.scheduler.bullets_mut().set_bullet_velocity(bullet_id, new_velocity);
        }
    }
    
    /// Initialize physics GPU system
    pub fn initialize_physics_gpu(&mut self, device: &wgpu::Device) {
        self.scheduler.initialize_physics_gpu(device);
    }
}
