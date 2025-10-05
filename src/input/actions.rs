use super::{GamepadManager, KeyboardState, MouseState};
use gilrs::{Axis, Button};
use winit::keyboard::KeyCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    // Movement
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,

    // Camera
    CameraLookHorizontal, // Analog input
    CameraLookVertical,   // Analog input
    CameraZoomIn,
    CameraZoomOut,
    CameraReset,

    // Game actions
    Fire,
    Jump,
    Dash,
    Interact,
    PanicExplosion,
    PlasmaBurst,

    // Weapon switching
    NextWeapon,
    PrevWeapon,

    // UI/Menu
    Menu,
    Confirm,
    Cancel,

    // Game State
    Pause,
    Restart,

    // Debug
    ToggleWireframe,
    TestLightning,
    ResetSurface,
    SpawnSplitter,
    SpawnCannibal,
    SpawnShieldOrb,
    SpawnSnake,
    SpawnBlob,

    // Application
    Exit,
}

#[derive(Debug, Clone)]
pub struct InputBinding {
    pub keyboard: Option<KeyCode>,
    pub gamepad_button: Option<Button>,
    pub gamepad_axis: Option<(Axis, f32)>, // axis and threshold
}

impl InputBinding {
    pub fn keyboard(key: KeyCode) -> Self {
        Self {
            keyboard: Some(key),
            gamepad_button: None,
            gamepad_axis: None,
        }
    }

    pub fn gamepad_axis(axis: Axis, threshold: f32) -> Self {
        Self {
            keyboard: None,
            gamepad_button: None,
            gamepad_axis: Some((axis, threshold)),
        }
    }

    pub fn mouse_button(_button: winit::event::MouseButton) -> Self {
        Self {
            keyboard: None,
            gamepad_button: None,
            gamepad_axis: None,
        }
    }

    pub fn with_gamepad_button(mut self, button: Button) -> Self {
        self.gamepad_button = Some(button);
        self
    }
}

pub struct ActionBindings {
    bindings: std::collections::HashMap<Action, Vec<InputBinding>>,
}

impl Default for ActionBindings {
    fn default() -> Self {
        let mut bindings = std::collections::HashMap::new();

        // Movement bindings (WASD + gamepad)
        bindings.insert(
            Action::MoveForward,
            vec![
                InputBinding::keyboard(KeyCode::KeyW).with_gamepad_button(Button::DPadUp),
                InputBinding::gamepad_axis(Axis::LeftStickY, 0.01), // Low threshold for analog
            ],
        );

        bindings.insert(
            Action::MoveBackward,
            vec![
                InputBinding::keyboard(KeyCode::KeyS).with_gamepad_button(Button::DPadDown),
                InputBinding::gamepad_axis(Axis::LeftStickY, 0.01), // Use positive threshold, handle direction in get_action_value
            ],
        );

        bindings.insert(
            Action::MoveLeft,
            vec![
                InputBinding::keyboard(KeyCode::KeyA).with_gamepad_button(Button::DPadLeft),
                InputBinding::gamepad_axis(Axis::LeftStickX, 0.01), // Use positive threshold, handle direction in get_action_value
            ],
        );

        bindings.insert(
            Action::MoveRight,
            vec![
                InputBinding::keyboard(KeyCode::KeyD).with_gamepad_button(Button::DPadRight),
                InputBinding::gamepad_axis(Axis::LeftStickX, 0.01), // Low threshold for analog
            ],
        );

        bindings.insert(
            Action::MoveUp,
            vec![
                InputBinding::keyboard(KeyCode::Space).with_gamepad_button(Button::South), // Space + A/X button for up movement
            ],
        );

        bindings.insert(
            Action::MoveDown,
            vec![
                InputBinding::keyboard(KeyCode::ControlLeft).with_gamepad_button(Button::East), // Ctrl + B/Circle button for down movement
            ],
        );

        // Camera bindings - mouse + gamepad
        bindings.insert(
            Action::CameraLookHorizontal,
            vec![
                InputBinding::gamepad_axis(Axis::RightStickX, 0.01), // Very low threshold for maximum sensitivity
            ],
        );

        bindings.insert(
            Action::CameraLookVertical,
            vec![InputBinding::gamepad_axis(Axis::RightStickY, 0.01)],
        );

        bindings.insert(
            Action::CameraZoomIn,
            vec![
                InputBinding::keyboard(KeyCode::Equal), // + key only
            ],
        );

        bindings.insert(
            Action::CameraZoomOut,
            vec![
                InputBinding::keyboard(KeyCode::Minus), // Minus key only
            ],
        );

        bindings.insert(
            Action::CameraReset,
            vec![InputBinding::keyboard(KeyCode::KeyR).with_gamepad_button(Button::LeftThumb)],
        );

        // Game actions
        bindings.insert(
            Action::Fire,
            vec![
                InputBinding::keyboard(KeyCode::Space), // Space bar for shooting!
                InputBinding::mouse_button(winit::event::MouseButton::Left),
                InputBinding::gamepad_axis(Axis::RightZ, 0.1), // Right trigger axis (RZ) for shooting
            ],
        );

        bindings.insert(
            Action::Jump,
            vec![InputBinding::keyboard(KeyCode::Space).with_gamepad_button(Button::South)],
        );

        bindings.insert(
            Action::Dash,
            vec![
                InputBinding::keyboard(KeyCode::ShiftLeft).with_gamepad_button(Button::LeftTrigger), // Left Shift + Left Bumper for dash
            ],
        );

        bindings.insert(
            Action::Interact,
            vec![
                InputBinding::keyboard(KeyCode::KeyF).with_gamepad_button(Button::West), // F key + X button (West on Xbox)
            ],
        );

        bindings.insert(
            Action::PanicExplosion,
            vec![
                InputBinding::keyboard(KeyCode::KeyX).with_gamepad_button(Button::North), // X key + Y button (North on Xbox)
            ],
        );

        // Weapon switching
        bindings.insert(
            Action::NextWeapon,
            vec![
                InputBinding::keyboard(KeyCode::KeyQ).with_gamepad_button(Button::RightTrigger), // Q key + Right Bumper
            ],
        );

        bindings.insert(
            Action::PrevWeapon,
            vec![
                InputBinding::keyboard(KeyCode::KeyE), //.with_gamepad_button(Button::LeftTrigger2), // E key + Left Bumper
            ],
        );

        // UI/Menu
        bindings.insert(
            Action::Menu,
            vec![InputBinding::keyboard(KeyCode::Escape).with_gamepad_button(Button::Start)],
        );

        bindings.insert(
            Action::Confirm,
            vec![InputBinding::keyboard(KeyCode::Enter).with_gamepad_button(Button::South)],
        );

        bindings.insert(
            Action::Cancel,
            vec![InputBinding::keyboard(KeyCode::Escape).with_gamepad_button(Button::East)],
        );

        // Game State actions
        bindings.insert(
            Action::Pause,
            vec![InputBinding::keyboard(KeyCode::KeyP).with_gamepad_button(Button::Start)],
        );

        bindings.insert(
            Action::Restart,
            vec![InputBinding::keyboard(KeyCode::KeyR).with_gamepad_button(Button::Select)],
        );

        // Debug actions
        bindings.insert(
            Action::ToggleWireframe,
            vec![InputBinding::keyboard(KeyCode::KeyT)],
        );

        bindings.insert(
            Action::TestLightning,
            vec![InputBinding::keyboard(KeyCode::KeyK)],
        );

        bindings.insert(
            Action::ResetSurface,
            vec![InputBinding::keyboard(KeyCode::KeyR)],
        );

        bindings.insert(
            Action::SpawnSplitter,
            vec![InputBinding::keyboard(KeyCode::KeyB)], // B for Boss
        );

        bindings.insert(
            Action::SpawnCannibal,
            vec![InputBinding::keyboard(KeyCode::KeyN)], // N for eNemy cannibal
        );

        bindings.insert(
            Action::SpawnShieldOrb,
            vec![InputBinding::keyboard(KeyCode::KeyO)], // O for Orb boss
        );

        bindings.insert(
            Action::SpawnSnake,
            vec![InputBinding::keyboard(KeyCode::KeyI)], // I for snake
        );

        bindings.insert(
            Action::SpawnBlob,
            vec![InputBinding::keyboard(KeyCode::KeyU)], // U for blob
        );

        // Application
        bindings.insert(Action::Exit, vec![InputBinding::keyboard(KeyCode::Escape)]);

        Self { bindings }
    }
}

impl ActionBindings {
    pub fn is_action_pressed(
        &self,
        action: Action,
        keyboard: &KeyboardState,
        _mouse: &MouseState,
        gamepad: &GamepadManager,
    ) -> bool {
        if let Some(bindings) = self.bindings.get(&action) {
            for binding in bindings {
                if let Some(key) = binding.keyboard {
                    if keyboard.is_key_pressed(key) {
                        return true;
                    }
                }

                if let Some(button) = binding.gamepad_button {
                    if gamepad.is_button_pressed(button) {
                        return true;
                    }
                }

                if let Some((axis, threshold)) = binding.gamepad_axis {
                    let value = gamepad.get_axis_value(axis);
                    if threshold > 0.0 && value >= threshold {
                        return true;
                    } else if threshold < 0.0 && value <= threshold {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn is_action_just_pressed(
        &self,
        action: Action,
        keyboard: &KeyboardState,
        _mouse: &MouseState,
        gamepad: &GamepadManager,
    ) -> bool {
        if let Some(bindings) = self.bindings.get(&action) {
            for binding in bindings {
                if let Some(key) = binding.keyboard {
                    if keyboard.is_key_just_pressed(key) {
                        return true;
                    }
                }

                if let Some(button) = binding.gamepad_button {
                    if gamepad.is_button_just_pressed(button) {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn get_action_value(
        &self,
        action: Action,
        keyboard: &KeyboardState,
        _mouse: &MouseState,
        gamepad: &GamepadManager,
    ) -> f32 {
        if let Some(bindings) = self.bindings.get(&action) {
            for binding in bindings {
                // Check digital inputs first (keyboard/buttons)
                if let Some(key) = binding.keyboard {
                    if keyboard.is_key_pressed(key) {
                        return 1.0;
                    }
                }

                if let Some(button) = binding.gamepad_button {
                    if gamepad.is_button_pressed(button) {
                        return 1.0;
                    }
                }

                // Check analog inputs (axes)
                if let Some((axis, _threshold)) = binding.gamepad_axis {
                    let raw_value = gamepad.get_axis_value(axis);

                    // Handle directional mapping for movement
                    match action {
                        Action::MoveForward => {
                            if axis == Axis::LeftStickY {
                                return raw_value.max(0.0); // Forward is positive Y
                            }
                        }
                        Action::MoveBackward => {
                            if axis == Axis::LeftStickY {
                                return (-raw_value).max(0.0); // Backward is negative Y (inverted)
                            }
                        }
                        Action::MoveLeft => {
                            if axis == Axis::LeftStickX {
                                return (-raw_value).max(0.0); // Left is negative X (inverted)
                            }
                        }
                        Action::MoveRight => {
                            if axis == Axis::LeftStickX {
                                return raw_value.max(0.0); // Right is positive X
                            }
                        }
                        _ => {
                            // For camera and other actions, return raw value
                            return raw_value;
                        }
                    }
                }
            }
        }
        0.0
    }

    pub fn get_action_vector2(
        &self,
        action: Action,
        keyboard: &KeyboardState,
        mouse: &MouseState,
        gamepad: &GamepadManager,
    ) -> (f32, f32) {
        match action {
            Action::MoveForward => {
                // For movement, we want to get both X and Z for left stick
                let forward = self.get_action_value(Action::MoveForward, keyboard, mouse, gamepad)
                    - self.get_action_value(Action::MoveBackward, keyboard, mouse, gamepad);
                let right = self.get_action_value(Action::MoveRight, keyboard, mouse, gamepad)
                    - self.get_action_value(Action::MoveLeft, keyboard, mouse, gamepad);

                (right, forward)
            }
            _ => (0.0, 0.0),
        }
    }
}
