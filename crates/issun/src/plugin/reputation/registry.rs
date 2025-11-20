//! Reputation registry for managing all reputation scores

use super::types::*;
use crate::resources::Resource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for reputation system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationConfig {
    /// Default score for new entries
    pub default_score: f32,

    /// Optional score range (min, max)
    pub score_range: Option<(f32, f32)>,

    /// Enable automatic score clamping
    pub auto_clamp: bool,

    /// Enable score decay over time
    pub enable_decay: bool,

    /// Decay rate per time unit (e.g., per day/turn)
    pub decay_rate: f32,
}

impl Default for ReputationConfig {
    fn default() -> Self {
        Self {
            default_score: 0.0,
            score_range: None,
            auto_clamp: false,
            enable_decay: false,
            decay_rate: 0.0,
        }
    }
}

/// Registry of all reputation scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationRegistry {
    /// All reputation entries, keyed by (subject_id, category)
    ///
    /// Key format:
    /// - Single-dimensional: `observer->target`
    /// - Multi-dimensional: `observer->target:category`
    entries: HashMap<String, ReputationEntry>,

    /// Optional thresholds for semantic levels
    thresholds: Vec<ReputationThreshold>,

    /// Configuration
    config: ReputationConfig,
}

impl Default for ReputationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Resource for ReputationRegistry {}

impl ReputationRegistry {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            thresholds: Vec::new(),
            config: ReputationConfig::default(),
        }
    }

    /// Create with custom config
    pub fn with_config(mut self, config: ReputationConfig) -> Self {
        self.config = config;
        self
    }

    /// Add thresholds
    pub fn add_threshold(&mut self, threshold: ReputationThreshold) {
        self.thresholds.push(threshold);
    }

    /// Add multiple thresholds at once
    pub fn add_thresholds(&mut self, thresholds: Vec<ReputationThreshold>) {
        self.thresholds.extend(thresholds);
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

    /// Get reputation score for a subject (single-dimensional)
    pub fn get(&self, subject_id: &SubjectId) -> Option<&ReputationEntry> {
        let key = Self::make_key(subject_id, None);
        self.entries.get(&key)
    }

    /// Get mutable reputation score for a subject
    pub fn get_mut(&mut self, subject_id: &SubjectId) -> Option<&mut ReputationEntry> {
        let key = Self::make_key(subject_id, None);
        self.entries.get_mut(&key)
    }

    /// Get reputation score for a subject with category (multi-dimensional)
    pub fn get_category(&self, subject_id: &SubjectId, category: &str) -> Option<&ReputationEntry> {
        let key = Self::make_key(subject_id, Some(category));
        self.entries.get(&key)
    }

    /// Get mutable reputation score for a subject with category
    pub fn get_category_mut(
        &mut self,
        subject_id: &SubjectId,
        category: &str,
    ) -> Option<&mut ReputationEntry> {
        let key = Self::make_key(subject_id, Some(category));
        self.entries.get_mut(&key)
    }

    /// Get or create entry with default score
    pub fn get_or_create(&mut self, subject_id: SubjectId) -> &mut ReputationEntry {
        let key = Self::make_key(&subject_id, None);
        self.entries.entry(key).or_insert_with(|| {
            ReputationEntry::new(subject_id, self.config.default_score)
        })
    }

    /// Get or create entry with category
    pub fn get_or_create_category(
        &mut self,
        subject_id: SubjectId,
        category: String,
    ) -> &mut ReputationEntry {
        let key = Self::make_key(&subject_id, Some(&category));
        self.entries.entry(key).or_insert_with(|| {
            ReputationEntry::new(subject_id, self.config.default_score).with_category(category)
        })
    }

    /// Set reputation score (creates if doesn't exist)
    pub fn set(&mut self, subject_id: SubjectId, score: f32) {
        let auto_clamp = self.config.auto_clamp;
        let score_range = self.config.score_range;

        let entry = self.get_or_create(subject_id);
        entry.set_score(score);
        if auto_clamp {
            if let Some((min, max)) = score_range {
                entry.clamp(min, max);
            }
        }
    }

    /// Set reputation score with category
    pub fn set_category(&mut self, subject_id: SubjectId, category: String, score: f32) {
        let auto_clamp = self.config.auto_clamp;
        let score_range = self.config.score_range;

        let entry = self.get_or_create_category(subject_id, category);
        entry.set_score(score);
        if auto_clamp {
            if let Some((min, max)) = score_range {
                entry.clamp(min, max);
            }
        }
    }

    /// Adjust reputation by delta (creates if doesn't exist)
    pub fn adjust(&mut self, subject_id: SubjectId, delta: f32) {
        let auto_clamp = self.config.auto_clamp;
        let score_range = self.config.score_range;

        let entry = self.get_or_create(subject_id);
        entry.adjust(delta);

        if auto_clamp {
            if let Some((min, max)) = score_range {
                entry.clamp(min, max);
            }
        }
    }

    /// Adjust reputation with category
    pub fn adjust_category(&mut self, subject_id: SubjectId, category: String, delta: f32) {
        let auto_clamp = self.config.auto_clamp;
        let score_range = self.config.score_range;

        let entry = self.get_or_create_category(subject_id, category);
        entry.adjust(delta);

        if auto_clamp {
            if let Some((min, max)) = score_range {
                entry.clamp(min, max);
            }
        }
    }

    /// Get current threshold for a score
    pub fn get_threshold(&self, score: f32) -> Option<&ReputationThreshold> {
        self.thresholds.iter().find(|t| t.contains(score))
    }

    /// Get all thresholds
    pub fn thresholds(&self) -> &[ReputationThreshold] {
        &self.thresholds
    }

    /// Get configuration
    pub fn config(&self) -> &ReputationConfig {
        &self.config
    }

    /// Get all entries
    pub fn iter(&self) -> impl Iterator<Item = &ReputationEntry> {
        self.entries.values()
    }

    /// Get all entries (mutable)
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut ReputationEntry> {
        self.entries.values_mut()
    }

    /// Apply decay to all entries
    pub fn apply_decay(&mut self) {
        if !self.config.enable_decay {
            return;
        }

        for entry in self.entries.values_mut() {
            // Decay towards default score
            let diff = self.config.default_score - entry.score;
            entry.score += diff * self.config.decay_rate;
        }
    }

    /// Remove an entry
    pub fn remove(&mut self, subject_id: &SubjectId) -> Option<ReputationEntry> {
        let key = Self::make_key(subject_id, None);
        self.entries.remove(&key)
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

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_new() {
        let registry = ReputationRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.config().default_score, 0.0);
    }

    #[test]
    fn test_registry_with_config() {
        let config = ReputationConfig {
            default_score: 50.0,
            score_range: Some((-100.0, 100.0)),
            auto_clamp: true,
            enable_decay: false,
            decay_rate: 0.0,
        };
        let registry = ReputationRegistry::new().with_config(config.clone());
        assert_eq!(registry.config().default_score, 50.0);
        assert_eq!(registry.config().score_range, Some((-100.0, 100.0)));
    }

    #[test]
    fn test_get_or_create() {
        let mut registry = ReputationRegistry::new();
        let id = SubjectId::new("player", "kingdom");

        assert!(registry.get(&id).is_none());

        let entry = registry.get_or_create(id.clone());
        assert_eq!(entry.score, 0.0);

        // Should return existing entry
        let entry2 = registry.get_or_create(id.clone());
        assert_eq!(entry2.score, 0.0);
    }

    #[test]
    fn test_set_and_get() {
        let mut registry = ReputationRegistry::new();
        let id = SubjectId::new("player", "kingdom");

        registry.set(id.clone(), 75.0);
        let entry = registry.get(&id).unwrap();
        assert_eq!(entry.score, 75.0);
    }

    #[test]
    fn test_adjust() {
        let mut registry = ReputationRegistry::new();
        let id = SubjectId::new("player", "kingdom");

        registry.adjust(id.clone(), 10.0);
        assert_eq!(registry.get(&id).unwrap().score, 10.0);

        registry.adjust(id.clone(), 15.0);
        assert_eq!(registry.get(&id).unwrap().score, 25.0);

        registry.adjust(id.clone(), -5.0);
        assert_eq!(registry.get(&id).unwrap().score, 20.0);
    }

    #[test]
    fn test_auto_clamp() {
        let config = ReputationConfig {
            default_score: 0.0,
            score_range: Some((-100.0, 100.0)),
            auto_clamp: true,
            ..Default::default()
        };
        let mut registry = ReputationRegistry::new().with_config(config);
        let id = SubjectId::new("player", "kingdom");

        registry.set(id.clone(), 150.0);
        assert_eq!(registry.get(&id).unwrap().score, 100.0);

        registry.set(id.clone(), -150.0);
        assert_eq!(registry.get(&id).unwrap().score, -100.0);
    }

    #[test]
    fn test_category() {
        let mut registry = ReputationRegistry::new();
        let id = SubjectId::new("player", "npc_alice");

        registry.adjust_category(id.clone(), "romance".into(), 10.0);
        registry.adjust_category(id.clone(), "friendship".into(), 20.0);

        assert_eq!(
            registry.get_category(&id, "romance").unwrap().score,
            10.0
        );
        assert_eq!(
            registry.get_category(&id, "friendship").unwrap().score,
            20.0
        );

        // Different relationship
        let reverse_id = id.reverse();
        registry.adjust_category(reverse_id.clone(), "romance".into(), 5.0);
        assert_eq!(
            registry.get_category(&reverse_id, "romance").unwrap().score,
            5.0
        );
    }

    #[test]
    fn test_thresholds() {
        let mut registry = ReputationRegistry::new();
        registry.add_threshold(ReputationThreshold::new("Hostile", -100.0, -50.0));
        registry.add_threshold(ReputationThreshold::new("Neutral", -50.0, 50.0));
        registry.add_threshold(ReputationThreshold::new("Friendly", 50.0, 100.0));

        assert_eq!(registry.get_threshold(-75.0).unwrap().name, "Hostile");
        assert_eq!(registry.get_threshold(0.0).unwrap().name, "Neutral");
        assert_eq!(registry.get_threshold(75.0).unwrap().name, "Friendly");
        assert!(registry.get_threshold(150.0).is_none());
    }

    #[test]
    fn test_decay() {
        let config = ReputationConfig {
            default_score: 0.0,
            score_range: None,
            auto_clamp: false,
            enable_decay: true,
            decay_rate: 0.1,
        };
        let mut registry = ReputationRegistry::new().with_config(config);
        let id = SubjectId::new("player", "kingdom");

        registry.set(id.clone(), 100.0);
        assert_eq!(registry.get(&id).unwrap().score, 100.0);

        registry.apply_decay();
        // 100 + (0 - 100) * 0.1 = 100 - 10 = 90
        assert_eq!(registry.get(&id).unwrap().score, 90.0);

        registry.apply_decay();
        // 90 + (0 - 90) * 0.1 = 90 - 9 = 81
        assert_eq!(registry.get(&id).unwrap().score, 81.0);
    }

    #[test]
    fn test_remove() {
        let mut registry = ReputationRegistry::new();
        let id = SubjectId::new("player", "kingdom");

        registry.set(id.clone(), 50.0);
        assert!(registry.get(&id).is_some());

        let removed = registry.remove(&id);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().score, 50.0);
        assert!(registry.get(&id).is_none());
    }

    #[test]
    fn test_reverse_relationship() {
        let mut registry = ReputationRegistry::new();
        let player_to_kingdom = SubjectId::new("player", "kingdom");
        let kingdom_to_player = player_to_kingdom.reverse();

        registry.set(player_to_kingdom.clone(), 75.0);
        registry.set(kingdom_to_player.clone(), 50.0);

        assert_eq!(registry.get(&player_to_kingdom).unwrap().score, 75.0);
        assert_eq!(registry.get(&kingdom_to_player).unwrap().score, 50.0);
    }
}
