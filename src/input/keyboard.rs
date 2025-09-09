use winit::{
    event::{Event, KeyEvent, WindowEvent, ElementState},
    keyboard::{KeyCode, PhysicalKey},
};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct KeyboardState {
    pressed_keys: HashSet<KeyCode>,
    just_pressed_keys: HashSet<KeyCode>,
    just_released_keys: HashSet<KeyCode>,
}

impl KeyboardState {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            just_pressed_keys: HashSet::new(),
            just_released_keys: HashSet::new(),
        }
    }

    pub fn handle_event(&mut self, event: &Event<()>) {
        match event {
            Event::WindowEvent { 
                event: WindowEvent::KeyboardInput { 
                    event: KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(key_code),
                        ..
                    },
                    ..
                },
                ..
            } => {
                match state {
                    ElementState::Pressed => {
                        if !self.pressed_keys.contains(key_code) {
                            self.just_pressed_keys.insert(*key_code);
                        }
                        self.pressed_keys.insert(*key_code);
                    }
                    ElementState::Released => {
                        self.pressed_keys.remove(key_code);
                        self.just_released_keys.insert(*key_code);
                    }
                }
            }
            _ => {}
        }
    }

    pub fn update(&mut self) {
        // Clear the "just pressed/released" states for the next frame
        self.just_pressed_keys.clear();
        self.just_released_keys.clear();
    }

    pub fn is_key_pressed(&self, key_code: KeyCode) -> bool {
        self.pressed_keys.contains(&key_code)
    }

    pub fn is_key_just_pressed(&self, key_code: KeyCode) -> bool {
        self.just_pressed_keys.contains(&key_code)
    }

    pub fn is_key_just_released(&self, key_code: KeyCode) -> bool {
        self.just_released_keys.contains(&key_code)
    }

    pub fn any_key_pressed(&self) -> bool {
        !self.pressed_keys.is_empty()
    }

    pub fn any_key_just_pressed(&self) -> bool {
        !self.just_pressed_keys.is_empty()
    }

    pub fn pressed_keys(&self) -> impl Iterator<Item = &KeyCode> {
        self.pressed_keys.iter()
    }

    pub fn just_pressed_keys(&self) -> impl Iterator<Item = &KeyCode> {
        self.just_pressed_keys.iter()
    }

    pub fn just_released_keys(&self) -> impl Iterator<Item = &KeyCode> {
        self.just_released_keys.iter()
    }
}