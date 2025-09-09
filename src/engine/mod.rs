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
    time: TimeManager,
    fps_display_timer: f32,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            dispatcher: Dispatcher::new(),
            scheduler: Scheduler::new(),
            time: TimeManager::new(),
            fps_display_timer: 0.0,
        }
    }
    
    // Single update method that handles everything
    pub fn update(&mut self, delta_time: f32, input: &crate::input::InputManager, camera_forward: Vec3, camera_right: Vec3, camera_up: Vec3) -> (Vec<crate::graphics::Primitive>, Vec3) {
        // Update time tracking
        self.time.update(delta_time);
        
        // Display FPS every second
        self.fps_display_timer += delta_time;
        if self.fps_display_timer >= 1.0 {
            println!("FPS: {:.1} | Frame time: {:.2}ms", self.time.fps(), delta_time * 1000.0);
            self.fps_display_timer = 0.0;
        }
        
        // Pre-update
        self.scheduler.preupdate();
        
        // Main update
        self.scheduler.update(delta_time, input, camera_forward, camera_right, camera_up);
        
        // Post-update
        self.scheduler.postupdate();
        
        // Pre-render
        self.scheduler.prerender();
        
        // Get render data and player position
        let primitives = self.scheduler.get_render_data();
        let player_pos = self.scheduler.get_player_position();
        
        // Post-render
        self.scheduler.postrender();
        
        (primitives, player_pos)
    }
}
