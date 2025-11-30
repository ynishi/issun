//! Synthesis mechanic: Crafting, research, fusion, and transformation systems.
//!
//! This module provides a policy-based system for modeling how items, technologies,
//! skills, and entities are created through combination and transformation.
//!
//! # Key Insight: Synthesis is Transformation
//!
//! At its core, synthesis is about transforming inputs into outputs:
//! - **Crafting**: Materials → Items
//! - **Research**: Resources + Time → Technology
//! - **Skills**: Experience → Abilities
//! - **Fusion**: Entities → New Entity (with inheritance)
//!
//! # Architecture
//!
//! The synthesis mechanic follows **Policy-Based Design**:
//! - The core `SynthesisMechanic<P>` is generic over `SynthesisPolicy`
//! - `P: SynthesisPolicy` determines success rates, quality, and outcomes
//! - All logic is resolved at compile time via static dispatch
//!
//! # Outcome System
//!
//! Unlike simple success/failure, synthesis supports rich outcomes:
//!
//! - **Success**: Normal completion at expected quality
//! - **Critical Success**: Exceptional result with bonuses
//! - **Partial Success**: Degraded output with defects
//! - **Unexpected**: Accident or mutation (can be beneficial or harmful)
//! - **Transmutation**: Output transforms into different result
//! - **Failure**: No output, partial material consumption
//!
//! # Quick Start
//!
//! ```
//! use issun_core::mechanics::synthesis::prelude::*;
//! use issun_core::mechanics::{Mechanic, EventEmitter};
//!
//! // Define synthesis type (using default CraftingPolicy)
//! type Crafting = SynthesisMechanic;
//!
//! // Create configuration
//! let config = SynthesisConfig::default();
//! let mut state = SynthesisState::default();
//!
//! // Define a recipe
//! let recipe = Recipe::new("health_potion", "Health Potion")
//!     .with_category(RecipeCategory::Alchemy)
//!     .with_difficulty(1.2)
//!     .with_ingredient(Ingredient::new("red_herb", 2))
//!     .with_ingredient(Ingredient::new("water", 1))
//!     .with_prerequisite(Prerequisite::tech("basic_alchemy"))
//!     .with_output(SynthesisOutput::new("health_potion", 1))
//!     .with_base_quality(QualityLevel::Common);
//!
//! // Prepare input
//! let input = SynthesisInput {
//!     recipe,
//!     ingredients: vec![
//!         IngredientInput { id: "red_herb".into(), quantity: 2, quality: QualityLevel::Common },
//!         IngredientInput { id: "water".into(), quantity: 1, quality: QualityLevel::Common },
//!     ],
//!     synthesizer: SynthesizerStats {
//!         entity_id: "alchemist".into(),
//!         skill_level: 0.7,
//!         luck: 0.1,
//!         quality_bonus: 0.0,
//!         specializations: [(RecipeCategory::Alchemy, 0.2)].into_iter().collect(),
//!     },
//!     context: SynthesisContext::default(),
//!     unlocked: {
//!         let mut u = UnlockedPrerequisites::default();
//!         u.techs.insert(TechId("basic_alchemy".into()));
//!         u
//!     },
//!     rng: 0.6,
//!     current_tick: 100,
//! };
//!
//! // Event collector
//! # struct TestEmitter { events: Vec<SynthesisEvent> }
//! # impl EventEmitter<SynthesisEvent> for TestEmitter {
//! #     fn emit(&mut self, event: SynthesisEvent) { self.events.push(event); }
//! # }
//! let mut emitter = TestEmitter { events: vec![] };
//!
//! // Execute synthesis
//! Crafting::step(&config, &mut state, input, &mut emitter);
//!
//! // Check result
//! if let Some(outcome) = &state.outcome {
//!     if outcome.is_success() {
//!         println!("Created health potion!");
//!     }
//! }
//! ```
//!
//! # Use Cases
//!
//! - **Item Crafting**: Blacksmithing, cooking, alchemy
//! - **Technology Research**: Science trees, unlocking capabilities
//! - **Skill Systems**: Learning abilities, talent trees
//! - **Demon Fusion**: Megaten-style entity combination
//! - **Enchanting**: Adding properties to items
//! - **Recipe Discovery**: Finding hidden combinations

pub mod mechanic;
pub mod policies;
pub mod strategies;
pub mod types;

// Re-export core types
pub use mechanic::{SimpleSynthesisMechanic, SynthesisMechanic};
pub use policies::SynthesisPolicy;
pub use types::{
    Byproduct, CatalystId, Defect, FailureReason, FlagId, Ingredient, IngredientId,
    IngredientInput, InheritanceSource, InheritedTrait, OutcomeTable, OutcomeType, Prerequisite,
    PrerequisiteResult, QualityLevel, Recipe, RecipeCategory, RecipeId, SynthesisBonus,
    SynthesisConfig, SynthesisContext, SynthesisCost, SynthesisEvent, SynthesisHistoryEntry,
    SynthesisInput, SynthesisOutcome, SynthesisOutput, SynthesisState, SynthesizerId,
    SynthesizerStats, TechId, TraitId, TransmutationEntry, UnexpectedEntry, UnexpectedTrigger,
    UnlockedPrerequisites,
};

/// Prelude module for convenient imports.
///
/// Import everything needed to use the synthesis mechanic:
///
/// ```
/// use issun_core::mechanics::synthesis::prelude::*;
/// ```
pub mod prelude {
    pub use super::mechanic::{SimpleSynthesisMechanic, SynthesisMechanic};
    pub use super::policies::SynthesisPolicy;
    pub use super::strategies::CraftingPolicy;
    pub use super::types::{
        Byproduct, CatalystId, Defect, FailureReason, FlagId, Ingredient, IngredientId,
        IngredientInput, InheritanceSource, InheritedTrait, OutcomeTable, OutcomeType,
        Prerequisite, PrerequisiteResult, QualityLevel, Recipe, RecipeCategory, RecipeId,
        SynthesisBonus, SynthesisConfig, SynthesisContext, SynthesisCost, SynthesisEvent,
        SynthesisHistoryEntry, SynthesisInput, SynthesisOutcome, SynthesisOutput, SynthesisState,
        SynthesizerId, SynthesizerStats, TechId, TraitId, TransmutationEntry, UnexpectedEntry,
        UnexpectedTrigger, UnlockedPrerequisites,
    };
}
