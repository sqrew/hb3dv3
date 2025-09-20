pub mod actions;
pub mod gamepad;
pub mod keyboard;
pub mod mouse;

pub use actions::*;
pub use gamepad::*;
pub use keyboard::*;
pub use mouse::*;

use winit::event::Event;
use gilrs::{Gilrs, GamepadId};

pub struct InputManager {
    // Input subsystems
    pub keyboard: KeyboardState,
    pub mouse: MouseState,
    pub gamepad: GamepadManager,
    
    // Action mapping
    action_bindings: ActionBindings,
    
    // Gilrs context (optional - gamepad support may fail)
    gilrs: Option<Gilrs>,
    active_gamepad: Option<GamepadId>,
}

impl InputManager {
    pub fn new() -> Self {
        let gilrs = match Gilrs::new() {
            Ok(g) => Some(g),
            Err(e) => {
                eprintln!("Warning: Failed to initialize gamepad support: {}. Gamepad input disabled.", e);
                None
            }
        };
        
        let mut active_gamepad = None;
        let mut gamepad_manager = GamepadManager::new();
        
        if let Some(ref g) = gilrs {
            println!("Gamepad support initialized");
            if g.gamepads().count() > 0 {
                println!("Found {} connected gamepad(s)", g.gamepads().count());
                // Select the first connected gamepad
                if let Some((id, gamepad)) = g.gamepads().next() {
                    active_gamepad = Some(id);
                    println!("Auto-selected gamepad: {:?}", gamepad.name());
                }
            }
            
            // Initialize gamepad manager with already connected gamepads
            for (id, gamepad) in g.gamepads() {
                gamepad_manager.handle_gilrs_event(id, &gilrs::EventType::Connected);
                println!("Initialized connected gamepad: {} (ID: {:?})", gamepad.name(), id);
            }
        }
        
        Self {
            keyboard: KeyboardState::new(),
            mouse: MouseState::new(),
            gamepad: gamepad_manager,
            action_bindings: ActionBindings::default(),
            gilrs,
            active_gamepad,
        }
    }
    
    pub fn handle_event(&mut self, event: &Event<()>) {
        self.keyboard.handle_event(event);
        self.mouse.handle_event(event);
    }
    
    pub fn update(&mut self) {
        // Update gilrs and handle gamepad events if available
        if let Some(ref mut gilrs) = self.gilrs {
            // Limit gamepad event processing to prevent frame stalls
            let mut events_processed = 0;
            const MAX_EVENTS_PER_FRAME: usize = 32; // Safety limit to prevent input event storms

            while let Some(gilrs::Event { id, event, time: _, .. }) = gilrs.next_event() {
                self.gamepad.handle_gilrs_event(id, &event);

                // Auto-select the first connected gamepad
                if self.active_gamepad.is_none() {
                    if let gilrs::EventType::Connected = event {
                        self.active_gamepad = Some(id);
                        println!("Gamepad connected and selected: {:?}", gilrs.gamepad(id).name());
                    }
                }

                // Handle disconnection
                if Some(id) == self.active_gamepad {
                    if let gilrs::EventType::Disconnected = event {
                        self.active_gamepad = None;
                        println!("Active gamepad disconnected");
                    }
                }

                events_processed += 1;
                if events_processed >= MAX_EVENTS_PER_FRAME {
                    // If we hit the limit, log it and break to prevent frame stalls
                    if cfg!(debug_assertions) {
                        println!("Warning: Gamepad event processing limited to {} events per frame", MAX_EVENTS_PER_FRAME);
                    }
                    break;
                }
            }
            
            // Update gamepad with gilrs context (but don't clear just_pressed state yet)
            self.gamepad.update(gilrs, self.active_gamepad);
        }
        
        // Update input subsystems (but don't clear transient state yet)
        // Note: keyboard.update() is called in end_frame() to preserve just_pressed state
        self.mouse.update();
    }
    
    /// Call this after game systems have processed input to clear transient state
    pub fn end_frame(&mut self) {
        self.keyboard.update(); // Clear just_pressed state after game systems read it
        self.gamepad.clear_transient_state();
    }
    
    // Action-based input queries
    pub fn is_action_pressed(&self, action: Action) -> bool {
        self.action_bindings.is_action_pressed(action, &self.keyboard, &self.mouse, &self.gamepad)
    }
    
    pub fn is_action_just_pressed(&self, action: Action) -> bool {
        self.action_bindings.is_action_just_pressed(action, &self.keyboard, &self.mouse, &self.gamepad)
    }
    
    pub fn get_action_value(&self, action: Action) -> f32 {
        self.action_bindings.get_action_value(action, &self.keyboard, &self.mouse, &self.gamepad)
    }
    
    pub fn get_action_vector2(&self, action: Action) -> (f32, f32) {
        self.action_bindings.get_action_vector2(action, &self.keyboard, &self.mouse, &self.gamepad)
    }
}