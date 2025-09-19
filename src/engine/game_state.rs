#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Playing,
    Paused,
    MainMenu,
}

impl Default for GameState {
    fn default() -> Self {
        GameState::Playing
    }
}

impl GameState {
    pub fn is_playing(&self) -> bool {
        matches!(self, GameState::Playing)
    }

    pub fn is_paused(&self) -> bool {
        matches!(self, GameState::Paused)
    }

    pub fn is_main_menu(&self) -> bool {
        matches!(self, GameState::MainMenu)
    }

    pub fn should_update_game_systems(&self) -> bool {
        matches!(self, GameState::Playing)
    }

    pub fn should_update_physics(&self) -> bool {
        matches!(self, GameState::Playing)
    }

    pub fn toggle_pause(&mut self) {
        match self {
            GameState::Playing => *self = GameState::Paused,
            GameState::Paused => *self = GameState::Playing,
            GameState::MainMenu => *self = GameState::Playing,
        }
    }

    pub fn restart(&mut self) {
        *self = GameState::Playing;
    }

    pub fn to_main_menu(&mut self) {
        *self = GameState::MainMenu;
    }
}