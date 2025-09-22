pub mod collision;
pub mod dispatcher;
pub mod entity;
pub mod game_state;
pub mod math;
pub mod scheduler;
pub mod time;
pub mod window;

pub use collision::*;
pub use dispatcher::*;
pub use entity::*;
pub use game_state::*;
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
    game_state: GameState,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            dispatcher: Dispatcher::new(),
            scheduler: Scheduler::new(),
            collision_manager: CollisionManager::new(),
            time: TimeManager::new(),
            fps_display_timer: 0.0,
            game_state: GameState::default(),
        }
    }

    // Single update method that handles everything
    pub fn update(
        &mut self,
        delta_time: f32,
        input: &crate::input::InputManager,
        camera_forward: Vec3,
        camera_right: Vec3,
        camera_up: Vec3,
    ) -> (
        Vec<crate::graphics::Primitive>,
        Vec3,
        Vec<dispatcher::GraphicsEvent>,
    ) {
        self.update_with_gpu(
            delta_time,
            input,
            camera_forward,
            camera_right,
            camera_up,
            None,
            None,
        )
    }

    // Update method with optional GPU context for physics
    pub fn update_with_gpu(
        &mut self,
        delta_time: f32,
        input: &crate::input::InputManager,
        camera_forward: Vec3,
        camera_right: Vec3,
        camera_up: Vec3,
        device: Option<&wgpu::Device>,
        queue: Option<&wgpu::Queue>,
    ) -> (
        Vec<crate::graphics::Primitive>,
        Vec3,
        Vec<dispatcher::GraphicsEvent>,
    ) {
        // Handle state input
        self.handle_state_input(input);

        // Always update time tracking
        self.time.update(delta_time);

        // Display FPS every second
        self.fps_display_timer += delta_time;
        if self.fps_display_timer >= 1.0 {
            println!(
                "FPS: {:.1} | Frame time: {:.2}ms | State: {:?}",
                self.time.fps(),
                delta_time * 1000.0,
                self.game_state
            );
            self.fps_display_timer = 0.0;
        }

        // Process queued events from previous frame
        let mut graphics_events = Vec::new();
        self.dispatcher
            .process_events(&mut self.scheduler, &mut graphics_events);

        // Only update game systems if we should
        if self.game_state.should_update_game_systems() {
            // Pre-update
            self.scheduler.preupdate();

            // Main update with physics support
            self.scheduler.update_with_physics(
                delta_time,
                input,
                camera_forward,
                camera_right,
                camera_up,
                device,
                queue,
            );

            // Post-update
            self.scheduler.postupdate();

            // Let dispatcher collect events directly from managers
            self.dispatcher
                .collect_from_managers(&mut self.scheduler, &mut self.collision_manager);

            // Prepare events for next frame
            self.dispatcher.prepare_next_frame();
        }

        // Always do render preparation
        self.scheduler.prerender();

        // Get render data and player position
        let primitives = self.scheduler.get_render_data();
        let player_pos = self.scheduler.get_player_position();

        // Post-render
        self.scheduler.postrender();

        (primitives, player_pos, graphics_events)
    }

    /// Process GPU collision pairs
    pub fn process_gpu_collisions(&mut self, collision_pairs: &[(u32, u32)]) {
        self.collision_manager.process_collision_pairs(
            collision_pairs,
            &self.scheduler.bullets(),
            &self.scheduler.enemies(),
            &self.scheduler.entity_manager(),
        );
    }

    /// Process large body vs large body collisions (runs every frame)
    pub fn process_large_body_collisions(&mut self) {
        self.collision_manager
            .process_large_body_collisions(&self.scheduler.large_bodies());
    }

    /// Handle collision cleanup and event dispatch (runs every frame)
    pub fn process_collision_cleanup(&mut self) {
        // Handle bullet removals from collisions
        let bullets_to_remove = self.collision_manager.drain_bullets_to_remove();
        for bullet_id in bullets_to_remove {
            self.scheduler
                .bullets_mut()
                .mark_bullet_for_removal(bullet_id);
        }

        // Handle enemy removals from collisions
        let enemies_to_remove = self.collision_manager.drain_enemies_to_remove();
        for enemy_id in enemies_to_remove {
            self.scheduler
                .enemies_mut()
                .mark_enemy_for_removal(enemy_id);
        }

        // Handle bullet velocity changes from bullet-bullet collisions
        let velocity_changes = self.collision_manager.drain_bullet_velocity_changes();
        for (bullet_id, new_velocity) in velocity_changes {
            self.scheduler
                .bullets_mut()
                .set_bullet_velocity(bullet_id, new_velocity);
        }

        // Events will be drained and processed by the main update loop
    }

    /// Initialize physics GPU system
    pub fn initialize_physics_gpu(&mut self, device: &wgpu::Device) {
        self.scheduler.initialize_physics_gpu(device);
    }

    /// Handle state input (pause, restart, etc.)
    fn handle_state_input(&mut self, input: &crate::input::InputManager) {
        use crate::input::Action;

        if input.is_action_just_pressed(Action::Pause) {
            self.game_state.toggle_pause();
            println!("Game state changed to: {:?}", self.game_state);
        }

        if input.is_action_just_pressed(Action::Restart) {
            self.restart_game();
        }
    }

    /// Get current game state

    /// Restart the game
    pub fn restart_game(&mut self) {
        println!("Restarting game...");
        self.game_state.restart();

        // Reset all game systems through the scheduler
        self.scheduler.restart_game();

        // Clear collision manager state
        self.collision_manager = CollisionManager::new();

        // Reset time but keep FPS tracking
        self.time = TimeManager::new();

        println!("Game restarted successfully!");
    }
}
