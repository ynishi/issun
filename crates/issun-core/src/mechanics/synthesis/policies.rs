//! Policy traits for synthesis mechanic.

use super::types::{
    CatalystId, IngredientInput, InheritanceSource, InheritedTrait, Prerequisite,
    PrerequisiteResult, QualityLevel, Recipe, SynthesisConfig, SynthesisContext, SynthesisInput,
    SynthesisOutcome, SynthesizerStats, UnexpectedTrigger, UnlockedPrerequisites,
};

/// Policy for synthesis behavior and outcome calculation
///
/// This trait defines how synthesis dynamics are calculated including
/// success rates, quality determination, and outcome resolution.
/// Different implementations can model different synthesis systems
/// (crafting, research, fusion, etc.).
pub trait SynthesisPolicy {
    /// Calculate success rate for synthesis attempt
    ///
    /// # Arguments
    ///
    /// * `config` - Synthesis configuration
    /// * `recipe` - Recipe being attempted
    /// * `synthesizer` - Synthesizer stats
    /// * `ingredients` - Provided ingredients
    ///
    /// # Returns
    ///
    /// Success rate (0.0-1.0)
    fn calculate_success_rate(
        config: &SynthesisConfig,
        recipe: &Recipe,
        synthesizer: &SynthesizerStats,
        ingredients: &[IngredientInput],
    ) -> f32;

    /// Determine output quality
    ///
    /// # Arguments
    ///
    /// * `config` - Synthesis configuration
    /// * `recipe` - Recipe being attempted
    /// * `synthesizer` - Synthesizer stats
    /// * `ingredients` - Provided ingredients
    /// * `success_roll` - Random value for quality variation (0.0-1.0)
    ///
    /// # Returns
    ///
    /// Determined quality level
    fn determine_quality(
        config: &SynthesisConfig,
        recipe: &Recipe,
        synthesizer: &SynthesizerStats,
        ingredients: &[IngredientInput],
        success_roll: f32,
    ) -> QualityLevel;

    /// Check if prerequisites are satisfied
    ///
    /// # Arguments
    ///
    /// * `prerequisites` - List of prerequisites to check
    /// * `unlocked` - Current unlocked state
    ///
    /// # Returns
    ///
    /// Prerequisite result (satisfied or list of missing)
    fn check_prerequisites(
        prerequisites: &[Prerequisite],
        unlocked: &UnlockedPrerequisites,
    ) -> PrerequisiteResult;

    /// Determine synthesis outcome
    ///
    /// This is the main outcome resolution function that considers:
    /// - Success rate and roll
    /// - Critical success possibility
    /// - Partial success possibility
    /// - Unexpected outcomes
    /// - Transmutation
    ///
    /// # Arguments
    ///
    /// * `config` - Synthesis configuration
    /// * `input` - Full synthesis input
    /// * `success_rate` - Pre-calculated success rate
    /// * `roll` - Random value for outcome determination (0.0-1.0)
    ///
    /// # Returns
    ///
    /// Determined synthesis outcome
    fn determine_outcome(
        config: &SynthesisConfig,
        input: &SynthesisInput,
        success_rate: f32,
        roll: f32,
    ) -> SynthesisOutcome;

    /// Check for unexpected outcome possibility
    ///
    /// # Arguments
    ///
    /// * `recipe` - Recipe being attempted
    /// * `ingredients` - Provided ingredients
    /// * `context` - Synthesis context (time, catalysts, etc.)
    ///
    /// # Returns
    ///
    /// Optional unexpected trigger if conditions are met
    fn check_unexpected(
        recipe: &Recipe,
        ingredients: &[IngredientInput],
        context: &SynthesisContext,
    ) -> Option<UnexpectedTrigger>;

    /// Check for transmutation possibility
    ///
    /// # Arguments
    ///
    /// * `recipe` - Recipe being attempted
    /// * `ingredients` - Provided ingredients
    /// * `catalysts` - Available catalysts
    ///
    /// # Returns
    ///
    /// Optional (target recipe ID, catalyst) if transmutation is possible
    fn check_transmutation(
        recipe: &Recipe,
        ingredients: &[IngredientInput],
        catalysts: &[CatalystId],
    ) -> Option<(super::types::RecipeId, Option<CatalystId>)>;

    /// Determine trait inheritance for fusion
    ///
    /// # Arguments
    ///
    /// * `sources` - Inheritance sources (entities being fused)
    /// * `slots` - Number of trait slots available
    /// * `affinity` - Overall inheritance affinity
    /// * `rng` - Random value for inheritance selection
    ///
    /// # Returns
    ///
    /// List of inherited traits
    fn determine_inheritance(
        sources: &[InheritanceSource],
        slots: usize,
        affinity: f32,
        rng: f32,
    ) -> Vec<InheritedTrait>;

    /// Calculate material consumption on failure
    ///
    /// # Arguments
    ///
    /// * `config` - Synthesis configuration
    /// * `recipe` - Recipe that failed
    /// * `success_rate` - Success rate at time of failure
    ///
    /// # Returns
    ///
    /// Fraction of materials consumed (0.0-1.0)
    fn calculate_failure_consumption(
        config: &SynthesisConfig,
        recipe: &Recipe,
        success_rate: f32,
    ) -> f32;

    /// Calculate average ingredient quality
    ///
    /// Helper function to compute quality from ingredients.
    ///
    /// # Arguments
    ///
    /// * `ingredients` - Provided ingredients
    ///
    /// # Returns
    ///
    /// Average quality level
    fn calculate_ingredient_quality(ingredients: &[IngredientInput]) -> QualityLevel {
        if ingredients.is_empty() {
            return QualityLevel::Common;
        }

        let total: u32 = ingredients.iter().map(|i| i.quality.value() as u32).sum();
        let avg = total / ingredients.len() as u32;
        QualityLevel::from_value(avg as u8)
    }
}
