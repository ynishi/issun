//! Input mapping system for ISSUN

use crossterm::event::KeyCode;
use std::collections::HashMap;

/// Game action enum (extensible by users)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameAction {
    /// Move up
    MoveUp,
    /// Move down
    MoveDown,
    /// Move left
    MoveLeft,
    /// Move right
    MoveRight,
    /// Confirm/Select
    Confirm,
    /// Cancel/Back
    Cancel,
    /// Menu
    Menu,
    /// Quit
    Quit,
    /// Custom action (for user-defined actions)
    Custom(u32),
}

/// Input mapper for key bindings
pub struct InputMapper {
    bindings: HashMap<KeyCode, GameAction>,
}

impl InputMapper {
    /// Create a new input mapper with default bindings
    pub fn new() -> Self {
        let mut bindings = HashMap::new();

        // Default bindings
        bindings.insert(KeyCode::Up, GameAction::MoveUp);
        bindings.insert(KeyCode::Char('k'), GameAction::MoveUp);

        bindings.insert(KeyCode::Down, GameAction::MoveDown);
        bindings.insert(KeyCode::Char('j'), GameAction::MoveDown);

        bindings.insert(KeyCode::Left, GameAction::MoveLeft);
        bindings.insert(KeyCode::Char('h'), GameAction::MoveLeft);

        bindings.insert(KeyCode::Right, GameAction::MoveRight);
        bindings.insert(KeyCode::Char('l'), GameAction::MoveRight);

        bindings.insert(KeyCode::Enter, GameAction::Confirm);
        bindings.insert(KeyCode::Char(' '), GameAction::Confirm);

        bindings.insert(KeyCode::Esc, GameAction::Cancel);
        bindings.insert(KeyCode::Backspace, GameAction::Cancel);

        bindings.insert(KeyCode::Char('m'), GameAction::Menu);
        bindings.insert(KeyCode::Char('q'), GameAction::Quit);

        Self { bindings }
    }

    /// Map a key to an action
    pub fn map_key(&self, key: KeyCode) -> Option<GameAction> {
        self.bindings.get(&key).copied()
    }

    /// Rebind a key to an action
    pub fn rebind(&mut self, action: GameAction, key: KeyCode) {
        self.bindings.insert(key, action);
    }

    /// Remove a key binding
    pub fn unbind(&mut self, key: KeyCode) {
        self.bindings.remove(&key);
    }

    /// Get all bindings for an action
    pub fn get_bindings_for_action(&self, action: GameAction) -> Vec<KeyCode> {
        self.bindings
            .iter()
            .filter(|(_, a)| **a == action)
            .map(|(k, _)| *k)
            .collect()
    }
}

impl Default for InputMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_bindings() {
        let mapper = InputMapper::new();

        assert_eq!(mapper.map_key(KeyCode::Up), Some(GameAction::MoveUp));
        assert_eq!(mapper.map_key(KeyCode::Char('k')), Some(GameAction::MoveUp));
        assert_eq!(mapper.map_key(KeyCode::Enter), Some(GameAction::Confirm));
    }

    #[test]
    fn test_rebind() {
        let mut mapper = InputMapper::new();
        mapper.rebind(GameAction::Quit, KeyCode::Char('x'));

        assert_eq!(mapper.map_key(KeyCode::Char('x')), Some(GameAction::Quit));
    }
}
