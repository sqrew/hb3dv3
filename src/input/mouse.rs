use winit::{
    event::{Event, WindowEvent, MouseButton, ElementState},
    dpi::PhysicalPosition,
};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct MouseState {
    // Position
    position: PhysicalPosition<f64>,
    last_position: PhysicalPosition<f64>,
    delta: (f64, f64),
    
    // Buttons
    pressed_buttons: HashSet<MouseButton>,
    just_pressed_buttons: HashSet<MouseButton>,
    just_released_buttons: HashSet<MouseButton>,
    
    // Scroll
    scroll_delta: f32,
    
    // Window focus
    cursor_entered: bool,
    cursor_left: bool,
}

impl MouseState {
    pub fn new() -> Self {
        Self {
            position: PhysicalPosition::new(0.0, 0.0),
            last_position: PhysicalPosition::new(0.0, 0.0),
            delta: (0.0, 0.0),
            pressed_buttons: HashSet::new(),
            just_pressed_buttons: HashSet::new(),
            just_released_buttons: HashSet::new(),
            scroll_delta: 0.0,
            cursor_entered: false,
            cursor_left: false,
        }
    }

    pub fn handle_event(&mut self, event: &Event<()>) {
        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CursorMoved { position, .. } => {
                        self.last_position = self.position;
                        self.position = *position;
                        self.delta = (
                            position.x - self.last_position.x,
                            position.y - self.last_position.y,
                        );
                    }
                    
                    WindowEvent::MouseInput { state, button, .. } => {
                        match state {
                            ElementState::Pressed => {
                                if !self.pressed_buttons.contains(button) {
                                    self.just_pressed_buttons.insert(*button);
                                }
                                self.pressed_buttons.insert(*button);
                            }
                            ElementState::Released => {
                                self.pressed_buttons.remove(button);
                                self.just_released_buttons.insert(*button);
                            }
                        }
                    }
                    
                    WindowEvent::MouseWheel { delta, .. } => {
                        match delta {
                            winit::event::MouseScrollDelta::LineDelta(_x, y) => {
                                self.scroll_delta += y;
                            }
                            winit::event::MouseScrollDelta::PixelDelta(pos) => {
                                self.scroll_delta += pos.y as f32 * 0.01;
                            }
                        }
                    }
                    
                    WindowEvent::CursorEntered { .. } => {
                        self.cursor_entered = true;
                        self.cursor_left = false;
                    }
                    
                    WindowEvent::CursorLeft { .. } => {
                        self.cursor_left = true;
                        self.cursor_entered = false;
                        self.delta = (0.0, 0.0);
                    }
                    
                    _ => {}
                }
            }
            _ => {}
        }
    }

    pub fn update(&mut self) {
        // Clear per-frame state
        self.just_pressed_buttons.clear();
        self.just_released_buttons.clear();
        self.scroll_delta = 0.0;
        self.cursor_entered = false;
        self.cursor_left = false;
        
        // Reset delta if no movement this frame
        if self.position == self.last_position {
            self.delta = (0.0, 0.0);
        }
    }

    // Position queries
    pub fn position(&self) -> PhysicalPosition<f64> {
        self.position
    }

    pub fn delta(&self) -> (f64, f64) {
        self.delta
    }

    pub fn delta_x(&self) -> f64 {
        self.delta.0
    }

    pub fn delta_y(&self) -> f64 {
        self.delta.1
    }

    // Button queries
    pub fn is_button_pressed(&self, button: MouseButton) -> bool {
        self.pressed_buttons.contains(&button)
    }

    pub fn is_button_just_pressed(&self, button: MouseButton) -> bool {
        self.just_pressed_buttons.contains(&button)
    }

    pub fn is_button_just_released(&self, button: MouseButton) -> bool {
        self.just_released_buttons.contains(&button)
    }

    pub fn any_button_pressed(&self) -> bool {
        !self.pressed_buttons.is_empty()
    }

    pub fn any_button_just_pressed(&self) -> bool {
        !self.just_pressed_buttons.is_empty()
    }

    // Scroll queries
    pub fn scroll_delta(&self) -> f32 {
        self.scroll_delta
    }

    // Cursor state
    pub fn is_cursor_in_window(&self) -> bool {
        !self.cursor_left
    }

    pub fn cursor_entered_this_frame(&self) -> bool {
        self.cursor_entered
    }

    pub fn cursor_left_this_frame(&self) -> bool {
        self.cursor_left
    }
}