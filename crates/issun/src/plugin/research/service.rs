//! Research service for domain logic
//!
//! Provides pure functions for research/development calculations.
//! All functions are stateless and can be used independently.

use super::types::{ResearchId, ResearchProject};

/// Research service providing pure research calculation logic
///
/// This service handles stateless calculations for research operations.
/// It follows Domain-Driven Design principles - research logic as a service.
///
/// # Design Philosophy
///
/// - **Stateless**: All functions are pure, taking inputs and returning outputs
/// - **Testable**: No dependencies on Registry or Resources
/// - **Reusable**: Can be called from Registry, Hook, or game code directly
///
/// # Example
///
/// ```ignore
/// use issun::plugin::research::{ResearchService, ResearchProject};
///
/// let project = ResearchProject::new("writing", "Writing", "Foundation of civilization");
///
/// // Calculate progress for this turn
/// let base_progress = 0.1; // 10% per turn
/// let speed_mult = 1.2; // +20% from bonuses
/// let difficulty_penalty = 1.0; // No penalty
/// let progress = ResearchService::calculate_progress(
///     base_progress,
///     speed_mult,
///     difficulty_penalty,
/// );
/// assert_eq!(progress, 0.12); // 0.1 * 1.2 * 1.0
///
/// // Calculate research cost
/// let base_cost = 1000;
/// let tier = 2;
/// let cost_mult = 1.5;
/// let cost = ResearchService::calculate_cost(base_cost, tier, cost_mult);
/// assert_eq!(cost, 6000); // 1000 * (2^2) * 1.5
/// ```
#[derive(Debug, Clone, Default)]
pub struct ResearchService;

impl ResearchService {
    /// Create a new research service
    pub fn new() -> Self {
        Self
    }

    /// Calculate research progress for a single tick/turn
    ///
    /// # Formula
    ///
    /// ```text
    /// progress = base_progress * speed_multiplier / difficulty_penalty
    /// ```
    ///
    /// # Arguments
    ///
    /// * `base_progress` - Base progress amount per turn (e.g., 0.1 = 10% per turn)
    /// * `speed_multiplier` - Multiplier from bonuses (e.g., 1.2 = +20% speed)
    /// * `difficulty_penalty` - Penalty from difficulty (e.g., 1.5 = 50% slower)
    ///
    /// # Returns
    ///
    /// Progress amount to add (0.0-1.0 range, but can exceed 1.0 for fast research)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Normal progress: 10% per turn
    /// let progress = ResearchService::calculate_progress(0.1, 1.0, 1.0);
    /// assert_eq!(progress, 0.1);
    ///
    /// // With +50% speed bonus
    /// let progress = ResearchService::calculate_progress(0.1, 1.5, 1.0);
    /// assert_eq!(progress, 0.15);
    ///
    /// // With difficulty penalty (50% slower)
    /// let progress = ResearchService::calculate_progress(0.1, 1.0, 1.5);
    /// assert!((progress - 0.0667).abs() < 0.001); // 0.1 / 1.5
    ///
    /// // Combined: +50% speed, +50% difficulty
    /// let progress = ResearchService::calculate_progress(0.1, 1.5, 1.5);
    /// assert_eq!(progress, 0.1); // 0.1 * 1.5 / 1.5
    /// ```
    pub fn calculate_progress(
        base_progress: f32,
        speed_multiplier: f32,
        difficulty_penalty: f32,
    ) -> f32 {
        if difficulty_penalty <= 0.0 {
            return base_progress * speed_multiplier; // No penalty
        }

        (base_progress * speed_multiplier / difficulty_penalty).max(0.0)
    }

    /// Calculate research cost
    ///
    /// Cost scales exponentially with tier/level to represent increasing complexity.
    ///
    /// # Formula
    ///
    /// ```text
    /// cost = base_cost * (tier ^ 2) * cost_multiplier
    /// ```
    ///
    /// # Arguments
    ///
    /// * `base_cost` - Base cost for tier 1
    /// * `tier` - Research tier/level (1, 2, 3, ...)
    /// * `cost_multiplier` - Multiplier from game state (e.g., inflation, difficulty)
    ///
    /// # Returns
    ///
    /// Final research cost
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Tier 1: Base cost only
    /// let cost = ResearchService::calculate_cost(1000, 1, 1.0);
    /// assert_eq!(cost, 1000); // 1000 * 1^2 * 1.0
    ///
    /// // Tier 2: 4x base cost
    /// let cost = ResearchService::calculate_cost(1000, 2, 1.0);
    /// assert_eq!(cost, 4000); // 1000 * 2^2
    ///
    /// // Tier 3: 9x base cost
    /// let cost = ResearchService::calculate_cost(1000, 3, 1.0);
    /// assert_eq!(cost, 9000); // 1000 * 3^2
    ///
    /// // With cost multiplier (e.g., 50% discount)
    /// let cost = ResearchService::calculate_cost(1000, 2, 0.5);
    /// assert_eq!(cost, 2000); // 1000 * 2^2 * 0.5
    /// ```
    pub fn calculate_cost(base_cost: i64, tier: u32, cost_multiplier: f32) -> i64 {
        let tier_scaling = (tier as f32).powi(2);
        let final_cost = base_cost as f32 * tier_scaling * cost_multiplier;
        final_cost.max(0.0).round() as i64
    }

    /// Check if prerequisites are satisfied
    ///
    /// # Arguments
    ///
    /// * `required` - List of required research IDs
    /// * `completed` - List of completed research IDs
    ///
    /// # Returns
    ///
    /// * `Ok(())` if all prerequisites are met
    /// * `Err(missing)` with list of missing prerequisite IDs
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let required = vec![
    ///     ResearchId::new("writing"),
    ///     ResearchId::new("philosophy"),
    /// ];
    ///
    /// let completed = vec![
    ///     ResearchId::new("writing"),
    ///     ResearchId::new("philosophy"),
    ///     ResearchId::new("mathematics"),
    /// ];
    ///
    /// // All prerequisites met
    /// let result = ResearchService::check_prerequisites(&required, &completed);
    /// assert!(result.is_ok());
    ///
    /// let incomplete = vec![
    ///     ResearchId::new("writing"),
    /// ];
    ///
    /// // Missing "philosophy"
    /// let result = ResearchService::check_prerequisites(&required, &incomplete);
    /// assert!(result.is_err());
    /// let missing = result.unwrap_err();
    /// assert_eq!(missing.len(), 1);
    /// assert_eq!(missing[0].as_str(), "philosophy");
    /// ```
    pub fn check_prerequisites(
        required: &[ResearchId],
        completed: &[ResearchId],
    ) -> Result<(), Vec<ResearchId>> {
        let missing: Vec<ResearchId> = required
            .iter()
            .filter(|req| !completed.contains(req))
            .cloned()
            .collect();

        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }

    /// Estimate turns to completion
    ///
    /// # Arguments
    ///
    /// * `current_progress` - Current progress (0.0-1.0)
    /// * `progress_per_turn` - Progress added each turn
    ///
    /// # Returns
    ///
    /// Estimated number of turns to complete (rounded up)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // 50% complete, 10% per turn = 5 turns
    /// let turns = ResearchService::estimate_completion(0.5, 0.1);
    /// assert_eq!(turns, 5);
    ///
    /// // 90% complete, 10% per turn = 1 turn
    /// let turns = ResearchService::estimate_completion(0.9, 0.1);
    /// assert_eq!(turns, 1);
    ///
    /// // Already complete
    /// let turns = ResearchService::estimate_completion(1.0, 0.1);
    /// assert_eq!(turns, 0);
    ///
    /// // Partial turn (0.75 complete, 0.3 per turn) = 1 turn
    /// let turns = ResearchService::estimate_completion(0.75, 0.3);
    /// assert_eq!(turns, 1);
    /// ```
    pub fn estimate_completion(current_progress: f32, progress_per_turn: f32) -> u32 {
        if current_progress >= 1.0 {
            return 0; // Already complete
        }

        if progress_per_turn <= 0.0 {
            return u32::MAX; // No progress = never complete
        }

        let remaining = (1.0 - current_progress).max(0.0);

        // Handle floating point precision: if remaining is very small, consider it complete
        const EPSILON: f32 = 1e-5;
        if remaining < EPSILON {
            return 0;
        }

        let turns_f = remaining / progress_per_turn;

        // Round to 6 decimal places to avoid floating point precision issues
        let turns_rounded = (turns_f * 1000000.0).round() / 1000000.0;

        turns_rounded.ceil() as u32
    }

    /// Calculate priority score for research projects
    ///
    /// Used for AI decision-making or auto-queue sorting.
    ///
    /// # Formula
    ///
    /// ```text
    /// priority = Σ(metric_value * weight)
    /// ```
    ///
    /// # Arguments
    ///
    /// * `project` - Research project to score
    /// * `weights` - Weights for each metric (e.g., `{"military_value": 2.0, "economic_value": 1.5}`)
    ///
    /// # Returns
    ///
    /// Priority score (higher = more important)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let project = ResearchProject::new("advanced_tactics", "Advanced Tactics", "...")
    ///     .add_metric("military_value", 10.0)
    ///     .add_metric("economic_value", 5.0);
    ///
    /// let mut weights = HashMap::new();
    /// weights.insert("military_value".into(), 2.0); // Military is 2x important
    /// weights.insert("economic_value".into(), 1.0);
    ///
    /// let priority = ResearchService::calculate_priority(&project, &weights);
    /// assert_eq!(priority, 25.0); // (10 * 2) + (5 * 1) = 25
    /// ```
    pub fn calculate_priority(
        project: &ResearchProject,
        weights: &std::collections::HashMap<String, f32>,
    ) -> f32 {
        project
            .metrics
            .iter()
            .filter_map(|(key, value)| {
                weights.get(key).map(|weight| value * weight)
            })
            .sum()
    }

    /// Calculate total progress including new amount
    ///
    /// Ensures progress is clamped to 0.0-1.0 range.
    ///
    /// # Arguments
    ///
    /// * `current_progress` - Current progress (0.0-1.0)
    /// * `progress_delta` - Amount to add
    ///
    /// # Returns
    ///
    /// New progress (clamped to 0.0-1.0)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Normal addition
    /// let progress = ResearchService::add_progress(0.5, 0.3);
    /// assert_eq!(progress, 0.8);
    ///
    /// // Clamped to 1.0
    /// let progress = ResearchService::add_progress(0.9, 0.3);
    /// assert_eq!(progress, 1.0);
    ///
    /// // Negative delta (edge case, should clamp to 0.0)
    /// let progress = ResearchService::add_progress(0.2, -0.5);
    /// assert_eq!(progress, 0.0);
    /// ```
    pub fn add_progress(current_progress: f32, progress_delta: f32) -> f32 {
        (current_progress + progress_delta).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_calculate_progress() {
        // Normal progress
        let progress = ResearchService::calculate_progress(0.1, 1.0, 1.0);
        assert!((progress - 0.1).abs() < 0.001);

        // With speed bonus
        let progress = ResearchService::calculate_progress(0.1, 1.5, 1.0);
        assert!((progress - 0.15).abs() < 0.001);

        // With difficulty penalty
        let progress = ResearchService::calculate_progress(0.1, 1.0, 1.5);
        assert!((progress - 0.0667).abs() < 0.001);

        // Combined
        let progress = ResearchService::calculate_progress(0.1, 1.5, 1.5);
        assert!((progress - 0.1).abs() < 0.001);

        // Zero difficulty (no penalty)
        let progress = ResearchService::calculate_progress(0.1, 1.5, 0.0);
        assert!((progress - 0.15).abs() < 0.001);
    }

    #[test]
    fn test_calculate_cost() {
        // Tier 1
        let cost = ResearchService::calculate_cost(1000, 1, 1.0);
        assert_eq!(cost, 1000);

        // Tier 2
        let cost = ResearchService::calculate_cost(1000, 2, 1.0);
        assert_eq!(cost, 4000);

        // Tier 3
        let cost = ResearchService::calculate_cost(1000, 3, 1.0);
        assert_eq!(cost, 9000);

        // With cost multiplier
        let cost = ResearchService::calculate_cost(1000, 2, 0.5);
        assert_eq!(cost, 2000);

        // With cost multiplier > 1
        let cost = ResearchService::calculate_cost(1000, 2, 1.5);
        assert_eq!(cost, 6000);

        // Zero tier (edge case)
        let cost = ResearchService::calculate_cost(1000, 0, 1.0);
        assert_eq!(cost, 0);
    }

    #[test]
    fn test_check_prerequisites() {
        let required = vec![
            ResearchId::new("writing"),
            ResearchId::new("philosophy"),
        ];

        // All prerequisites met
        let completed = vec![
            ResearchId::new("writing"),
            ResearchId::new("philosophy"),
            ResearchId::new("mathematics"),
        ];
        let result = ResearchService::check_prerequisites(&required, &completed);
        assert!(result.is_ok());

        // Missing one prerequisite
        let incomplete = vec![ResearchId::new("writing")];
        let result = ResearchService::check_prerequisites(&required, &incomplete);
        assert!(result.is_err());
        let missing = result.unwrap_err();
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0].as_str(), "philosophy");

        // Missing all prerequisites
        let none = vec![];
        let result = ResearchService::check_prerequisites(&required, &none);
        assert!(result.is_err());
        let missing = result.unwrap_err();
        assert_eq!(missing.len(), 2);

        // No prerequisites required
        let empty_req = vec![];
        let result = ResearchService::check_prerequisites(&empty_req, &none);
        assert!(result.is_ok());
    }

    #[test]
    fn test_estimate_completion() {
        // 50% complete, 10% per turn = 5 turns
        let turns = ResearchService::estimate_completion(0.5, 0.1);
        assert_eq!(turns, 5);

        // 90% complete, 10% per turn = 1 turn
        let turns = ResearchService::estimate_completion(0.9, 0.1);
        assert_eq!(turns, 1);

        // Already complete
        let turns = ResearchService::estimate_completion(1.0, 0.1);
        assert_eq!(turns, 0);

        // Over 100% (edge case)
        let turns = ResearchService::estimate_completion(1.5, 0.1);
        assert_eq!(turns, 0);

        // Partial turn (0.75 complete, 0.3 per turn)
        let turns = ResearchService::estimate_completion(0.75, 0.3);
        assert_eq!(turns, 1);

        // Zero progress per turn
        let turns = ResearchService::estimate_completion(0.5, 0.0);
        assert_eq!(turns, u32::MAX);

        // Negative progress per turn (edge case)
        let turns = ResearchService::estimate_completion(0.5, -0.1);
        assert_eq!(turns, u32::MAX);
    }

    #[test]
    fn test_calculate_priority() {
        let project = ResearchProject::new("advanced_tactics", "Advanced Tactics", "...")
            .add_metric("military_value", 10.0)
            .add_metric("economic_value", 5.0);

        let mut weights = HashMap::new();
        weights.insert("military_value".into(), 2.0);
        weights.insert("economic_value".into(), 1.0);

        let priority = ResearchService::calculate_priority(&project, &weights);
        assert!((priority - 25.0).abs() < 0.001); // (10 * 2) + (5 * 1)

        // No matching weights
        let empty_weights = HashMap::new();
        let priority = ResearchService::calculate_priority(&project, &empty_weights);
        assert!((priority - 0.0).abs() < 0.001);

        // Partial matching weights
        let mut partial_weights = HashMap::new();
        partial_weights.insert("military_value".into(), 1.5);
        let priority = ResearchService::calculate_priority(&project, &partial_weights);
        assert!((priority - 15.0).abs() < 0.001); // 10 * 1.5
    }

    #[test]
    fn test_add_progress() {
        // Normal addition
        let progress = ResearchService::add_progress(0.5, 0.3);
        assert!((progress - 0.8).abs() < 0.001);

        // Clamped to 1.0
        let progress = ResearchService::add_progress(0.9, 0.3);
        assert_eq!(progress, 1.0);

        // Exact completion
        let progress = ResearchService::add_progress(0.7, 0.3);
        assert_eq!(progress, 1.0);

        // Negative delta (edge case)
        let progress = ResearchService::add_progress(0.2, -0.5);
        assert_eq!(progress, 0.0);

        // Zero delta
        let progress = ResearchService::add_progress(0.5, 0.0);
        assert_eq!(progress, 0.5);
    }
}
