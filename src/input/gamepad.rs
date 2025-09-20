use gilrs::{Axis, Button, EventType, GamepadId, Gilrs};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct GamepadState {
    // Buttons
    pressed_buttons: HashSet<Button>,
    just_pressed_buttons: HashSet<Button>,
    just_released_buttons: HashSet<Button>,

    // Axes
    axis_values: HashMap<Axis, f32>,

    // Connection
    connected: bool,
    name: String,
}

impl GamepadState {
    pub fn new() -> Self {
        Self {
            pressed_buttons: HashSet::new(),
            just_pressed_buttons: HashSet::new(),
            just_released_buttons: HashSet::new(),
            axis_values: HashMap::new(),
            connected: false,
            name: String::new(),
        }
    }

    pub fn handle_event(&mut self, event: &EventType) {
        match event {
            EventType::ButtonPressed(button, _code) => {
                if !self.pressed_buttons.contains(button) {
                    self.just_pressed_buttons.insert(*button);
                }
                self.pressed_buttons.insert(*button);
            }

            EventType::ButtonReleased(button, _code) => {
                self.pressed_buttons.remove(button);
                self.just_released_buttons.insert(*button);
            }

            EventType::AxisChanged(axis, value, _code) => {
                // Apply deadzone (reduced for better sensitivity)
                let deadzone = 0.2;
                let adjusted_value = if value.abs() < deadzone {
                    0.0
                } else {
                    // Scale to account for deadzone
                    if *value > 0.0 {
                        (*value - deadzone) / (1.0 - deadzone)
                    } else {
                        (*value + deadzone) / (1.0 - deadzone)
                    }
                };

                self.axis_values.insert(*axis, adjusted_value);
            }

            _ => {}
        }
    }

    pub fn update(&mut self) {
        self.just_pressed_buttons.clear();
        self.just_released_buttons.clear();
    }

    pub fn set_connected(&mut self, connected: bool, name: String) {
        self.connected = connected;
        self.name = name;

        if !connected {
            self.pressed_buttons.clear();
            self.just_pressed_buttons.clear();
            self.just_released_buttons.clear();
            self.axis_values.clear();
        }
    }

    // Button queries
    pub fn is_button_pressed(&self, button: Button) -> bool {
        self.connected && self.pressed_buttons.contains(&button)
    }

    pub fn is_button_just_pressed(&self, button: Button) -> bool {
        self.connected && self.just_pressed_buttons.contains(&button)
    }

    pub fn is_button_just_released(&self, button: Button) -> bool {
        self.connected && self.just_released_buttons.contains(&button)
    }

    // Axis queries
    pub fn get_axis_value(&self, axis: Axis) -> f32 {
        if self.connected {
            self.axis_values.get(&axis).copied().unwrap_or(0.0)
        } else {
            0.0
        }
    }

    // Connection info
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug)]
pub struct GamepadManager {
    gamepads: HashMap<GamepadId, GamepadState>,
    active_gamepad: Option<GamepadId>,
}

impl GamepadManager {
    pub fn new() -> Self {
        Self {
            gamepads: HashMap::new(),
            active_gamepad: None,
        }
    }

    pub fn handle_gilrs_event(&mut self, id: GamepadId, event: &EventType) {
        match event {
            EventType::Connected => {
                let mut gamepad_state = GamepadState::new();
                gamepad_state.set_connected(true, format!("Gamepad {:?}", id));
                self.gamepads.insert(id, gamepad_state);

                // Auto-select first gamepad
                if self.active_gamepad.is_none() {
                    self.active_gamepad = Some(id);
                }
            }

            EventType::Disconnected => {
                if let Some(gamepad) = self.gamepads.get_mut(&id) {
                    gamepad.set_connected(false, String::new());
                }

                // Clear active gamepad if it was this one
                if self.active_gamepad == Some(id) {
                    self.active_gamepad = None;
                }

                self.gamepads.remove(&id);
            }

            EventType::ButtonPressed(button, code) => {
                if let Some(gamepad) = self.gamepads.get_mut(&id) {
                    gamepad.handle_event(&EventType::ButtonPressed(*button, *code));
                }
            }

            EventType::ButtonReleased(button, code) => {
                if let Some(gamepad) = self.gamepads.get_mut(&id) {
                    gamepad.handle_event(&EventType::ButtonReleased(*button, *code));
                }
            }

            EventType::AxisChanged(axis, value, code) => {
                if let Some(gamepad) = self.gamepads.get_mut(&id) {
                    gamepad.handle_event(&EventType::AxisChanged(*axis, *value, *code));
                }
            }

            _ => {}
        }
    }

    pub fn update(&mut self, gilrs: &mut Gilrs, active_id: Option<GamepadId>) {
        self.active_gamepad = active_id;

        // Don't update gamepad names every frame - they're already set on connection
        // This was causing frame stalls due to expensive gamepad.name() calls
        // Names are set once in handle_gilrs_event() when gamepads connect
    }

    /// Clear the just_pressed/just_released state after game systems have processed input
    pub fn clear_transient_state(&mut self) {
        for gamepad in self.gamepads.values_mut() {
            gamepad.update(); // Now it's safe to clear just_pressed state
        }
    }

    // Active gamepad queries (these are the main interface)
    pub fn is_button_pressed(&self, button: Button) -> bool {
        if let Some(id) = self.active_gamepad {
            if let Some(gamepad) = self.gamepads.get(&id) {
                return gamepad.is_button_pressed(button);
            }
        }
        false
    }

    pub fn is_button_just_pressed(&self, button: Button) -> bool {
        if let Some(id) = self.active_gamepad {
            if let Some(gamepad) = self.gamepads.get(&id) {
                return gamepad.is_button_just_pressed(button);
            }
        }
        false
    }

    pub fn is_button_just_released(&self, button: Button) -> bool {
        if let Some(id) = self.active_gamepad {
            if let Some(gamepad) = self.gamepads.get(&id) {
                return gamepad.is_button_just_released(button);
            }
        }
        false
    }

    pub fn get_axis_value(&self, axis: Axis) -> f32 {
        if let Some(id) = self.active_gamepad {
            if let Some(gamepad) = self.gamepads.get(&id) {
                let value = gamepad.get_axis_value(axis);

                return value;
            }
        }
        0.0
    }

    // Multi-gamepad queries
    pub fn get_gamepad(&self, id: GamepadId) -> Option<&GamepadState> {
        self.gamepads.get(&id)
    }

    pub fn active_gamepad_id(&self) -> Option<GamepadId> {
        self.active_gamepad
    }

    pub fn has_active_gamepad(&self) -> bool {
        self.active_gamepad.is_some()
            && self
                .active_gamepad
                .and_then(|id| self.gamepads.get(&id))
                .map_or(false, |g| g.is_connected())
    }

    pub fn active_gamepad_name(&self) -> Option<String> {
        self.active_gamepad
            .and_then(|id| self.gamepads.get(&id))
            .map(|g| g.name().to_string())
    }

    pub fn connected_gamepad_count(&self) -> usize {
        self.gamepads.values().filter(|g| g.is_connected()).count()
    }
}
