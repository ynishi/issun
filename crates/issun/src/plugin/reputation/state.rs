//! Reputation runtime state (mutable)

use super::types::*;
use crate::state::State;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Reputation runtime state (Mutable)
///
/// Contains all reputation scores that change during gameplay.
/// This is a save/load target.
///
/// # Design
///
/// - **ReputationConfig**: Configuration (ReadOnly)
/// - **ReputationState**: Runtime scores (Mutable)
///
/// # Example
///
/// ```ignore
/// use issun::plugin::reputation::{ReputationState, SubjectId};
///
/// let mut state = ReputationState::new();
///
/// let id = SubjectId::new("player", "kingdom");
/// state.set(&id, 75.0);
///
/// assert_eq!(state.get(&id), Some(75.0));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationState {
    /// All reputation entries, keyed by (subject_id, category)
    ///
    /// Key format:
    /// - Single-dimensional: `observer->target`
    /// - Multi-dimensional: `observer->target:category`
    entries: HashMap<String, ReputationEntry>,
}

impl State for ReputationState {}

impl ReputationState {
    /// Create a new empty state
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Generate internal key for storage
    ///
    /// Key format: `observer->target` or `observer->target:category`
    fn make_key(subject_id: &SubjectId, category: Option<&str>) -> String {
        match category {
            Some(cat) => format!("{}->{}:{}", subject_id.observer, subject_id.target, cat),
            None => format!("{}->{}", subject_id.observer, subject_id.target),
        }
    }

    // ========================================
    // Single-dimensional (no category)
    // ========================================

    /// Get reputation score
    pub fn get(&self, subject_id: &SubjectId) -> Option<f32> {
        let key = Self::make_key(subject_id, None);
        self.entries.get(&key).map(|entry| entry.score)
    }

    /// Get entry (immutable)
    pub fn get_entry(&self, subject_id: &SubjectId) -> Option<&ReputationEntry> {
        let key = Self::make_key(subject_id, None);
        self.entries.get(&key)
    }

    /// Get entry (mutable)
    pub fn get_entry_mut(&mut self, subject_id: &SubjectId) -> Option<&mut ReputationEntry> {
        let key = Self::make_key(subject_id, None);
        self.entries.get_mut(&key)
    }

    /// Set reputation score (creates if doesn't exist)
    pub fn set(&mut self, subject_id: &SubjectId, score: f32) {
        let key = Self::make_key(subject_id, None);
        self.entries
            .entry(key)
            .and_modify(|e| e.score = score)
            .or_insert_with(|| ReputationEntry::new(subject_id.clone(), score));
    }

    /// Adjust reputation by delta (creates if doesn't exist)
    ///
    /// Returns the old and new scores.
    pub fn adjust(&mut self, subject_id: &SubjectId, delta: f32, default_score: f32) -> (f32, f32) {
        let key = Self::make_key(subject_id, None);
        let entry = self
            .entries
            .entry(key)
            .or_insert_with(|| ReputationEntry::new(subject_id.clone(), default_score));

        let old_score = entry.score;
        entry.adjust(delta);
        (old_score, entry.score)
    }

    /// Remove an entry
    pub fn remove(&mut self, subject_id: &SubjectId) -> Option<ReputationEntry> {
        let key = Self::make_key(subject_id, None);
        self.entries.remove(&key)
    }

    // ========================================
    // Multi-dimensional (with category)
    // ========================================

    /// Get reputation score with category
    pub fn get_category(&self, subject_id: &SubjectId, category: &str) -> Option<f32> {
        let key = Self::make_key(subject_id, Some(category));
        self.entries.get(&key).map(|entry| entry.score)
    }

    /// Get entry with category (immutable)
    pub fn get_category_entry(
        &self,
        subject_id: &SubjectId,
        category: &str,
    ) -> Option<&ReputationEntry> {
        let key = Self::make_key(subject_id, Some(category));
        self.entries.get(&key)
    }

    /// Get entry with category (mutable)
    pub fn get_category_entry_mut(
        &mut self,
        subject_id: &SubjectId,
        category: &str,
    ) -> Option<&mut ReputationEntry> {
        let key = Self::make_key(subject_id, Some(category));
        self.entries.get_mut(&key)
    }

    /// Set reputation score with category
    pub fn set_category(&mut self, subject_id: &SubjectId, category: String, score: f32) {
        let key = Self::make_key(subject_id, Some(&category));
        self.entries
            .entry(key)
            .and_modify(|e| e.score = score)
            .or_insert_with(|| {
                ReputationEntry::new(subject_id.clone(), score).with_category(category)
            });
    }

    /// Adjust reputation with category
    pub fn adjust_category(
        &mut self,
        subject_id: &SubjectId,
        category: String,
        delta: f32,
        default_score: f32,
    ) -> (f32, f32) {
        let key = Self::make_key(subject_id, Some(&category));
        let entry = self.entries.entry(key).or_insert_with(|| {
            ReputationEntry::new(subject_id.clone(), default_score).with_category(category)
        });

        let old_score = entry.score;
        entry.adjust(delta);
        (old_score, entry.score)
    }

    /// Remove an entry with category
    pub fn remove_category(
        &mut self,
        subject_id: &SubjectId,
        category: &str,
    ) -> Option<ReputationEntry> {
        let key = Self::make_key(subject_id, Some(category));
        self.entries.remove(&key)
    }

    // ========================================
    // Bulk operations
    // ========================================

    /// Apply clamping to a score
    pub fn clamp_score(
        &mut self,
        subject_id: &SubjectId,
        min: f32,
        max: f32,
    ) -> Option<(f32, f32)> {
        let key = Self::make_key(subject_id, None);
        self.entries.get_mut(&key).map(|entry| {
            let old = entry.score;
            entry.clamp(min, max);
            (old, entry.score)
        })
    }

    /// Apply clamping to a score with category
    pub fn clamp_score_category(
        &mut self,
        subject_id: &SubjectId,
        category: &str,
        min: f32,
        max: f32,
    ) -> Option<(f32, f32)> {
        let key = Self::make_key(subject_id, Some(category));
        self.entries.get_mut(&key).map(|entry| {
            let old = entry.score;
            entry.clamp(min, max);
            (old, entry.score)
        })
    }

    /// Apply decay to all entries
    ///
    /// Decay moves scores towards the default score by the given rate.
    pub fn apply_decay(&mut self, default_score: f32, decay_rate: f32) {
        for entry in self.entries.values_mut() {
            let diff = default_score - entry.score;
            entry.score += diff * decay_rate;
        }
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if state is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterate over all entries
    pub fn iter(&self) -> impl Iterator<Item = &ReputationEntry> {
        self.entries.values()
    }

    /// Iterate over all entries (mutable)
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut ReputationEntry> {
        self.entries.values_mut()
    }
}

impl Default for ReputationState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_new() {
        let state = ReputationState::new();
        assert!(state.is_empty());
        assert_eq!(state.len(), 0);
    }

    #[test]
    fn test_set_and_get() {
        let mut state = ReputationState::new();
        let id = SubjectId::new("player", "kingdom");

        assert!(state.get(&id).is_none());

        state.set(&id, 75.0);
        assert_eq!(state.get(&id), Some(75.0));

        state.set(&id, 90.0);
        assert_eq!(state.get(&id), Some(90.0));
    }

    #[test]
    fn test_adjust() {
        let mut state = ReputationState::new();
        let id = SubjectId::new("player", "kingdom");

        let (old, new) = state.adjust(&id, 10.0, 0.0);
        assert_eq!(old, 0.0);
        assert_eq!(new, 10.0);

        let (old, new) = state.adjust(&id, 15.0, 0.0);
        assert_eq!(old, 10.0);
        assert_eq!(new, 25.0);

        let (old, new) = state.adjust(&id, -5.0, 0.0);
        assert_eq!(old, 25.0);
        assert_eq!(new, 20.0);
    }

    #[test]
    fn test_category() {
        let mut state = ReputationState::new();
        let id = SubjectId::new("player", "npc_alice");

        state.set_category(&id, "romance".into(), 10.0);
        state.set_category(&id, "friendship".into(), 20.0);

        assert_eq!(state.get_category(&id, "romance"), Some(10.0));
        assert_eq!(state.get_category(&id, "friendship"), Some(20.0));

        // Different category should be independent
        assert!(state.get(&id).is_none());
    }

    #[test]
    fn test_clamp() {
        let mut state = ReputationState::new();
        let id = SubjectId::new("player", "kingdom");

        state.set(&id, 150.0);
        let result = state.clamp_score(&id, -100.0, 100.0);
        assert_eq!(result, Some((150.0, 100.0)));
        assert_eq!(state.get(&id), Some(100.0));

        state.set(&id, -150.0);
        let result = state.clamp_score(&id, -100.0, 100.0);
        assert_eq!(result, Some((-150.0, -100.0)));
        assert_eq!(state.get(&id), Some(-100.0));
    }

    #[test]
    fn test_decay() {
        let mut state = ReputationState::new();
        let id = SubjectId::new("player", "kingdom");

        state.set(&id, 100.0);
        state.apply_decay(0.0, 0.1);
        // 100 + (0 - 100) * 0.1 = 100 - 10 = 90
        assert_eq!(state.get(&id), Some(90.0));

        state.apply_decay(0.0, 0.1);
        // 90 + (0 - 90) * 0.1 = 90 - 9 = 81
        assert_eq!(state.get(&id), Some(81.0));
    }

    #[test]
    fn test_remove() {
        let mut state = ReputationState::new();
        let id = SubjectId::new("player", "kingdom");

        state.set(&id, 50.0);
        assert!(state.get(&id).is_some());

        let removed = state.remove(&id);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().score, 50.0);
        assert!(state.get(&id).is_none());
    }

    #[test]
    fn test_reverse_relationship() {
        let mut state = ReputationState::new();
        let player_to_kingdom = SubjectId::new("player", "kingdom");
        let kingdom_to_player = player_to_kingdom.reverse();

        state.set(&player_to_kingdom, 75.0);
        state.set(&kingdom_to_player, 50.0);

        assert_eq!(state.get(&player_to_kingdom), Some(75.0));
        assert_eq!(state.get(&kingdom_to_player), Some(50.0));
    }
}
