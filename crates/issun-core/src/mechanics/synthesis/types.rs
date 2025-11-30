//! Type definitions for synthesis mechanic.
//!
//! Covers crafting, tech trees, skill trees, fusion, and research systems.

use std::collections::{HashMap, HashSet};

// ============================================================================
// Identifiers
// ============================================================================

/// Unique identifier for recipes
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecipeId(pub String);

impl From<&str> for RecipeId {
    fn from(s: &str) -> Self {
        RecipeId(s.to_string())
    }
}

impl From<String> for RecipeId {
    fn from(s: String) -> Self {
        RecipeId(s)
    }
}

/// Unique identifier for ingredients/items
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IngredientId(pub String);

impl From<&str> for IngredientId {
    fn from(s: &str) -> Self {
        IngredientId(s.to_string())
    }
}

impl From<String> for IngredientId {
    fn from(s: String) -> Self {
        IngredientId(s)
    }
}

/// Unique identifier for technologies
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TechId(pub String);

impl From<&str> for TechId {
    fn from(s: &str) -> Self {
        TechId(s.to_string())
    }
}

impl From<String> for TechId {
    fn from(s: String) -> Self {
        TechId(s)
    }
}

/// Unique identifier for flags/conditions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlagId(pub String);

impl From<&str> for FlagId {
    fn from(s: &str) -> Self {
        FlagId(s.to_string())
    }
}

impl From<String> for FlagId {
    fn from(s: String) -> Self {
        FlagId(s)
    }
}

/// Unique identifier for catalysts
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CatalystId(pub String);

impl From<&str> for CatalystId {
    fn from(s: &str) -> Self {
        CatalystId(s.to_string())
    }
}

impl From<String> for CatalystId {
    fn from(s: String) -> Self {
        CatalystId(s)
    }
}

/// Unique identifier for traits (for fusion inheritance)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TraitId(pub String);

impl From<&str> for TraitId {
    fn from(s: &str) -> Self {
        TraitId(s.to_string())
    }
}

impl From<String> for TraitId {
    fn from(s: String) -> Self {
        TraitId(s)
    }
}

/// Unique identifier for synthesizers (crafters, researchers, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynthesizerId(pub String);

impl From<&str> for SynthesizerId {
    fn from(s: &str) -> Self {
        SynthesizerId(s.to_string())
    }
}

impl From<String> for SynthesizerId {
    fn from(s: String) -> Self {
        SynthesizerId(s)
    }
}

// ============================================================================
// Quality
// ============================================================================

/// Quality level of synthesized items
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum QualityLevel {
    /// Broken/unusable
    Broken = 0,
    /// Poor quality
    Poor = 1,
    /// Common/normal quality
    #[default]
    Common = 2,
    /// Uncommon/above average
    Uncommon = 3,
    /// Rare/high quality
    Rare = 4,
    /// Epic/exceptional quality
    Epic = 5,
    /// Legendary/masterwork
    Legendary = 6,
}

impl QualityLevel {
    /// Get numeric value
    pub fn value(&self) -> u8 {
        *self as u8
    }

    /// Create from numeric value
    pub fn from_value(v: u8) -> Self {
        match v {
            0 => QualityLevel::Broken,
            1 => QualityLevel::Poor,
            2 => QualityLevel::Common,
            3 => QualityLevel::Uncommon,
            4 => QualityLevel::Rare,
            5 => QualityLevel::Epic,
            _ => QualityLevel::Legendary,
        }
    }

    /// Upgrade quality by steps (clamped)
    pub fn upgrade(&self, steps: i8) -> Self {
        let new_value = (self.value() as i8 + steps).clamp(0, 6) as u8;
        Self::from_value(new_value)
    }
}

// ============================================================================
// Prerequisites (Unlock/Lock system)
// ============================================================================

/// Prerequisite conditions for synthesis
#[derive(Debug, Clone, PartialEq)]
pub enum Prerequisite {
    /// Technology must be unlocked
    Tech {
        /// Required technology ID
        id: TechId,
    },

    /// Another recipe must be learned
    Recipe {
        /// Required recipe ID
        id: RecipeId,
    },

    /// Must possess item (not consumed by check)
    Item {
        /// Required item ID
        id: IngredientId,
        /// Minimum quantity required
        quantity: u64,
    },

    /// Minimum level/rank requirement
    Level {
        /// Minimum level
        min_level: u32,
    },

    /// Flag/condition must be set
    Flag {
        /// Required flag ID
        id: FlagId,
    },

    /// All sub-prerequisites must be met (AND)
    All {
        /// List of prerequisites (all required)
        prerequisites: Vec<Prerequisite>,
    },

    /// Any sub-prerequisite must be met (OR)
    Any {
        /// List of prerequisites (any one required)
        prerequisites: Vec<Prerequisite>,
    },

    /// Prerequisite must NOT be met
    Not {
        /// Prerequisite that must be false
        prerequisite: Box<Prerequisite>,
    },
}

impl Prerequisite {
    /// Create a tech prerequisite
    pub fn tech(id: impl Into<TechId>) -> Self {
        Prerequisite::Tech { id: id.into() }
    }

    /// Create a recipe prerequisite
    pub fn recipe(id: impl Into<RecipeId>) -> Self {
        Prerequisite::Recipe { id: id.into() }
    }

    /// Create an item prerequisite
    pub fn item(id: impl Into<IngredientId>, quantity: u64) -> Self {
        Prerequisite::Item {
            id: id.into(),
            quantity,
        }
    }

    /// Create a level prerequisite
    pub fn level(min_level: u32) -> Self {
        Prerequisite::Level { min_level }
    }

    /// Create a flag prerequisite
    pub fn flag(id: impl Into<FlagId>) -> Self {
        Prerequisite::Flag { id: id.into() }
    }

    /// Create an AND combination
    pub fn all(prerequisites: Vec<Prerequisite>) -> Self {
        Prerequisite::All { prerequisites }
    }

    /// Create an OR combination
    pub fn any(prerequisites: Vec<Prerequisite>) -> Self {
        Prerequisite::Any { prerequisites }
    }

    /// Create a NOT condition
    pub fn not(prerequisite: Prerequisite) -> Self {
        Prerequisite::Not {
            prerequisite: Box::new(prerequisite),
        }
    }
}

/// Result of prerequisite check
#[derive(Debug, Clone, PartialEq)]
pub enum PrerequisiteResult {
    /// All prerequisites met
    Satisfied,
    /// Prerequisites not met
    Unsatisfied {
        /// List of missing prerequisites
        missing: Vec<Prerequisite>,
    },
}

impl PrerequisiteResult {
    /// Check if satisfied
    pub fn is_satisfied(&self) -> bool {
        matches!(self, PrerequisiteResult::Satisfied)
    }
}

// ============================================================================
// Ingredients & Cost
// ============================================================================

/// Ingredient required for synthesis
#[derive(Debug, Clone, PartialEq)]
pub struct Ingredient {
    /// Ingredient ID
    pub id: IngredientId,
    /// Quantity required
    pub quantity: u64,
    /// Minimum quality required (None = any quality)
    pub min_quality: Option<QualityLevel>,
    /// Whether this ingredient is consumed
    pub consumed: bool,
}

impl Ingredient {
    /// Create a new ingredient requirement
    pub fn new(id: impl Into<IngredientId>, quantity: u64) -> Self {
        Self {
            id: id.into(),
            quantity,
            min_quality: None,
            consumed: true,
        }
    }

    /// Set minimum quality requirement
    pub fn with_min_quality(mut self, quality: QualityLevel) -> Self {
        self.min_quality = Some(quality);
        self
    }

    /// Set as catalyst (not consumed)
    pub fn as_catalyst(mut self) -> Self {
        self.consumed = false;
        self
    }
}

/// Input ingredient with actual values
#[derive(Debug, Clone)]
pub struct IngredientInput {
    /// Ingredient ID
    pub id: IngredientId,
    /// Quantity provided
    pub quantity: u64,
    /// Quality of provided ingredient
    pub quality: QualityLevel,
}

/// Synthesis cost (non-ingredient resources)
#[derive(Debug, Clone, PartialEq)]
pub struct SynthesisCost {
    /// Time cost (in ticks)
    pub time: u64,
    /// Energy/mana cost
    pub energy: f32,
    /// Currency cost
    pub currency: u64,
    /// Custom resource costs
    pub custom: HashMap<String, f32>,
}

impl Default for SynthesisCost {
    fn default() -> Self {
        Self {
            time: 0,
            energy: 0.0,
            currency: 0,
            custom: HashMap::new(),
        }
    }
}

impl SynthesisCost {
    /// Create a time-only cost
    pub fn time(ticks: u64) -> Self {
        Self {
            time: ticks,
            ..Default::default()
        }
    }

    /// Create an energy-only cost
    pub fn energy(amount: f32) -> Self {
        Self {
            energy: amount,
            ..Default::default()
        }
    }

    /// Add custom resource cost
    pub fn with_custom(mut self, resource: impl Into<String>, amount: f32) -> Self {
        self.custom.insert(resource.into(), amount);
        self
    }
}

// ============================================================================
// Recipe
// ============================================================================

/// Synthesis recipe definition
#[derive(Debug, Clone)]
pub struct Recipe {
    /// Unique identifier
    pub id: RecipeId,
    /// Display name
    pub name: String,
    /// Recipe category
    pub category: RecipeCategory,
    /// Difficulty level (affects success rate)
    pub difficulty: f32,
    /// Required ingredients
    pub ingredients: Vec<Ingredient>,
    /// Prerequisites to unlock/use
    pub prerequisites: Vec<Prerequisite>,
    /// Base cost
    pub cost: SynthesisCost,
    /// Base output
    pub output: SynthesisOutput,
    /// Base quality of output
    pub base_quality: QualityLevel,
    /// Outcome table for variable results
    pub outcome_table: Option<OutcomeTable>,
}

impl Recipe {
    /// Create a new recipe
    pub fn new(id: impl Into<RecipeId>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            category: RecipeCategory::Crafting,
            difficulty: 1.0,
            ingredients: Vec::new(),
            prerequisites: Vec::new(),
            cost: SynthesisCost::default(),
            output: SynthesisOutput::default(),
            base_quality: QualityLevel::Common,
            outcome_table: None,
        }
    }

    /// Set category
    pub fn with_category(mut self, category: RecipeCategory) -> Self {
        self.category = category;
        self
    }

    /// Set difficulty
    pub fn with_difficulty(mut self, difficulty: f32) -> Self {
        self.difficulty = difficulty;
        self
    }

    /// Add ingredient
    pub fn with_ingredient(mut self, ingredient: Ingredient) -> Self {
        self.ingredients.push(ingredient);
        self
    }

    /// Add prerequisite
    pub fn with_prerequisite(mut self, prerequisite: Prerequisite) -> Self {
        self.prerequisites.push(prerequisite);
        self
    }

    /// Set cost
    pub fn with_cost(mut self, cost: SynthesisCost) -> Self {
        self.cost = cost;
        self
    }

    /// Set output
    pub fn with_output(mut self, output: SynthesisOutput) -> Self {
        self.output = output;
        self
    }

    /// Set base quality
    pub fn with_base_quality(mut self, quality: QualityLevel) -> Self {
        self.base_quality = quality;
        self
    }

    /// Set outcome table
    pub fn with_outcome_table(mut self, table: OutcomeTable) -> Self {
        self.outcome_table = Some(table);
        self
    }
}

/// Recipe category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RecipeCategory {
    /// Item crafting
    #[default]
    Crafting,
    /// Technology research
    Research,
    /// Skill learning
    Skill,
    /// Entity fusion (Megaten style)
    Fusion,
    /// Alchemy/transmutation
    Alchemy,
    /// Enchanting/enhancement
    Enhancement,
    /// Custom category
    Custom,
}

// ============================================================================
// Output
// ============================================================================

/// Synthesis output definition
#[derive(Debug, Clone, PartialEq)]
pub struct SynthesisOutput {
    /// Output item/entity ID
    pub id: IngredientId,
    /// Base quantity produced
    pub quantity: u64,
    /// Inherited traits (for fusion)
    pub inherited_traits: Vec<TraitId>,
}

impl Default for SynthesisOutput {
    fn default() -> Self {
        Self {
            id: IngredientId("output".into()),
            quantity: 1,
            inherited_traits: Vec::new(),
        }
    }
}

impl SynthesisOutput {
    /// Create a new output
    pub fn new(id: impl Into<IngredientId>, quantity: u64) -> Self {
        Self {
            id: id.into(),
            quantity,
            inherited_traits: Vec::new(),
        }
    }

    /// Add inherited trait
    pub fn with_trait(mut self, trait_id: impl Into<TraitId>) -> Self {
        self.inherited_traits.push(trait_id.into());
        self
    }
}

/// Byproduct from synthesis
#[derive(Debug, Clone, PartialEq)]
pub struct Byproduct {
    /// Output item ID
    pub id: IngredientId,
    /// Quantity
    pub quantity: u64,
    /// Chance to produce (0.0-1.0)
    pub chance: f32,
}

// ============================================================================
// Outcome System
// ============================================================================

/// Synthesis outcome (result of synthesis attempt)
#[derive(Debug, Clone, PartialEq)]
pub enum SynthesisOutcome {
    /// Normal success
    Success {
        /// Primary output
        output: SynthesisOutput,
        /// Achieved quality
        quality: QualityLevel,
        /// Byproducts produced
        byproducts: Vec<Byproduct>,
    },

    /// Critical success (better than expected)
    CriticalSuccess {
        /// Primary output
        output: SynthesisOutput,
        /// Achieved quality (higher than base)
        quality: QualityLevel,
        /// Bonus effects
        bonuses: Vec<SynthesisBonus>,
        /// Byproducts produced
        byproducts: Vec<Byproduct>,
    },

    /// Partial success (usable but degraded)
    PartialSuccess {
        /// Primary output
        output: SynthesisOutput,
        /// Achieved quality (lower than base)
        quality: QualityLevel,
        /// Defects on the output
        defects: Vec<Defect>,
    },

    /// Unexpected result (accident/mutation)
    Unexpected {
        /// Actual output (different from recipe)
        output: SynthesisOutput,
        /// Quality of unexpected output
        quality: QualityLevel,
        /// What triggered this outcome
        trigger: UnexpectedTrigger,
        /// Whether this is beneficial
        is_beneficial: bool,
    },

    /// Transmutation (transformed into different recipe result)
    Transmutation {
        /// Original recipe attempted
        original_recipe: RecipeId,
        /// Actual output produced
        actual_output: SynthesisOutput,
        /// Quality of transmuted output
        quality: QualityLevel,
        /// What caused transmutation
        catalyst: Option<CatalystId>,
    },

    /// Failure (no output)
    Failure {
        /// Why it failed
        reason: FailureReason,
        /// Fraction of materials consumed (0.0-1.0)
        consumption: f32,
        /// Materials that could be salvaged
        salvage: Vec<SynthesisOutput>,
    },
}

impl SynthesisOutcome {
    /// Check if outcome is successful (any form of success)
    pub fn is_success(&self) -> bool {
        matches!(
            self,
            SynthesisOutcome::Success { .. }
                | SynthesisOutcome::CriticalSuccess { .. }
                | SynthesisOutcome::PartialSuccess { .. }
        )
    }

    /// Check if outcome produced any output
    pub fn has_output(&self) -> bool {
        !matches!(self, SynthesisOutcome::Failure { .. })
    }

    /// Get output if any
    pub fn output(&self) -> Option<&SynthesisOutput> {
        match self {
            SynthesisOutcome::Success { output, .. }
            | SynthesisOutcome::CriticalSuccess { output, .. }
            | SynthesisOutcome::PartialSuccess { output, .. }
            | SynthesisOutcome::Unexpected { output, .. }
            | SynthesisOutcome::Transmutation {
                actual_output: output,
                ..
            } => Some(output),
            SynthesisOutcome::Failure { .. } => None,
        }
    }

    /// Get quality if any
    pub fn quality(&self) -> Option<QualityLevel> {
        match self {
            SynthesisOutcome::Success { quality, .. }
            | SynthesisOutcome::CriticalSuccess { quality, .. }
            | SynthesisOutcome::PartialSuccess { quality, .. }
            | SynthesisOutcome::Unexpected { quality, .. }
            | SynthesisOutcome::Transmutation { quality, .. } => Some(*quality),
            SynthesisOutcome::Failure { .. } => None,
        }
    }
}

/// Bonus effect from critical success
#[derive(Debug, Clone, PartialEq)]
pub enum SynthesisBonus {
    /// Extra quantity produced
    ExtraQuantity {
        /// Additional amount
        amount: u64,
    },
    /// Quality upgrade
    QualityUpgrade {
        /// Steps upgraded
        steps: u8,
    },
    /// Trait inherited (fusion)
    TraitInherited {
        /// Inherited trait ID
        trait_id: TraitId,
    },
    /// Skill bonus (fusion)
    SkillBonus {
        /// Skill ID
        skill_id: String,
        /// Bonus amount
        bonus: f32,
    },
    /// Resource refund
    ResourceRefund {
        /// Resource type
        resource: String,
        /// Amount refunded
        amount: f32,
    },
}

/// Defect on partial success output
#[derive(Debug, Clone, PartialEq)]
pub enum Defect {
    /// Reduced durability
    ReducedDurability {
        /// Durability multiplier (0.0-1.0)
        multiplier: f32,
    },
    /// Reduced effectiveness
    ReducedEffectiveness {
        /// Effectiveness multiplier (0.0-1.0)
        multiplier: f32,
    },
    /// Missing trait (fusion)
    MissingTrait {
        /// Trait that should have been inherited
        trait_id: TraitId,
    },
    /// Negative trait added
    NegativeTrait {
        /// Added negative trait
        trait_id: TraitId,
    },
    /// Unstable (may break)
    Unstable {
        /// Chance to break on use (0.0-1.0)
        break_chance: f32,
    },
}

/// Trigger for unexpected outcomes
#[derive(Debug, Clone, PartialEq)]
pub enum UnexpectedTrigger {
    /// Temporal condition (moon phase, time of day, etc.)
    TemporalCondition {
        /// Condition identifier
        condition_id: String,
    },

    /// Synergy between specific ingredients
    IngredientSynergy {
        /// Ingredients that triggered synergy
        ingredients: Vec<IngredientId>,
    },

    /// Random accident
    RandomAccident {
        /// Base chance of this accident
        base_chance: f32,
    },

    /// Skill mismatch (too difficult for synthesizer)
    SkillMismatch {
        /// Required skill level
        required: f32,
        /// Actual skill level
        actual: f32,
    },

    /// Hidden recipe discovered
    HiddenRecipe {
        /// Discovered recipe ID
        recipe_id: RecipeId,
    },

    /// Catalyst interaction
    CatalystReaction {
        /// Catalyst that caused reaction
        catalyst_id: CatalystId,
    },
}

/// Reason for synthesis failure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureReason {
    /// Random failure (bad luck)
    BadLuck,
    /// Insufficient skill
    InsufficientSkill,
    /// Ingredient quality too low
    PoorIngredients,
    /// Equipment malfunction
    EquipmentFailure,
    /// Interrupted
    Interrupted,
    /// Prerequisites not met
    PrerequisitesNotMet,
    /// Insufficient resources
    InsufficientResources,
}

// ============================================================================
// Outcome Table
// ============================================================================

/// Table of possible outcomes for a recipe
#[derive(Debug, Clone, Default)]
pub struct OutcomeTable {
    /// Possible unexpected outcomes
    pub unexpected_outcomes: Vec<UnexpectedEntry>,
    /// Possible transmutation targets
    pub transmutation_targets: Vec<TransmutationEntry>,
    /// Critical success threshold (roll above this = critical)
    pub critical_threshold: f32,
    /// Partial success threshold (roll below this but above failure = partial)
    pub partial_threshold: f32,
}

impl OutcomeTable {
    /// Create a new outcome table
    pub fn new() -> Self {
        Self {
            unexpected_outcomes: Vec::new(),
            transmutation_targets: Vec::new(),
            critical_threshold: 0.95,
            partial_threshold: 0.3,
        }
    }

    /// Add unexpected outcome
    pub fn with_unexpected(mut self, entry: UnexpectedEntry) -> Self {
        self.unexpected_outcomes.push(entry);
        self
    }

    /// Add transmutation target
    pub fn with_transmutation(mut self, entry: TransmutationEntry) -> Self {
        self.transmutation_targets.push(entry);
        self
    }

    /// Set critical threshold
    pub fn with_critical_threshold(mut self, threshold: f32) -> Self {
        self.critical_threshold = threshold;
        self
    }

    /// Set partial threshold
    pub fn with_partial_threshold(mut self, threshold: f32) -> Self {
        self.partial_threshold = threshold;
        self
    }
}

/// Entry for unexpected outcome
#[derive(Debug, Clone)]
pub struct UnexpectedEntry {
    /// Trigger condition
    pub trigger: UnexpectedTrigger,
    /// Output if triggered
    pub output: SynthesisOutput,
    /// Whether beneficial
    pub is_beneficial: bool,
    /// Hint for discovery (optional)
    pub discovery_hint: Option<String>,
}

/// Entry for transmutation
#[derive(Debug, Clone)]
pub struct TransmutationEntry {
    /// Required catalyst
    pub catalyst: Option<CatalystId>,
    /// Required ingredients (in addition to recipe)
    pub required_ingredients: Vec<IngredientId>,
    /// Target recipe to transmute into
    pub target_recipe: RecipeId,
    /// Target output
    pub output: SynthesisOutput,
    /// Chance of transmutation (0.0-1.0)
    pub chance: f32,
}

// ============================================================================
// Fusion (Inheritance System)
// ============================================================================

/// Source for trait inheritance (fusion)
#[derive(Debug, Clone)]
pub struct InheritanceSource {
    /// Source entity ID
    pub entity_id: IngredientId,
    /// Available traits
    pub traits: Vec<TraitId>,
    /// Trait affinities (higher = more likely to inherit)
    pub affinities: HashMap<TraitId, f32>,
}

/// Inherited trait result
#[derive(Debug, Clone, PartialEq)]
pub struct InheritedTrait {
    /// Trait ID
    pub trait_id: TraitId,
    /// Source it came from
    pub source: IngredientId,
    /// Inheritance strength (may affect trait power)
    pub strength: f32,
}

// ============================================================================
// Config
// ============================================================================

/// Configuration for synthesis mechanic
#[derive(Debug, Clone)]
pub struct SynthesisConfig {
    /// Base success rate before modifiers
    pub base_success_rate: f32,
    /// Skill effectiveness (how much skill affects success)
    pub skill_effectiveness: f32,
    /// Quality variation range
    pub quality_variation: f32,
    /// Material consumption on failure (0.0-1.0)
    pub failure_consumption_rate: f32,
    /// Chance for unexpected outcomes
    pub unexpected_chance: f32,
    /// Minimum success rate floor
    pub min_success_rate: f32,
    /// Maximum success rate ceiling
    pub max_success_rate: f32,
}

impl Default for SynthesisConfig {
    fn default() -> Self {
        Self {
            base_success_rate: 0.7,
            skill_effectiveness: 0.3,
            quality_variation: 0.2,
            failure_consumption_rate: 0.5,
            unexpected_chance: 0.05,
            min_success_rate: 0.05,
            max_success_rate: 0.99,
        }
    }
}

// ============================================================================
// Input
// ============================================================================

/// Synthesis context (environmental factors)
#[derive(Debug, Clone, Default)]
pub struct SynthesisContext {
    /// Current time/tick
    pub current_tick: u64,
    /// Active temporal conditions
    pub temporal_conditions: HashSet<String>,
    /// Available catalysts
    pub catalysts: Vec<CatalystId>,
    /// Environmental modifiers
    pub modifiers: HashMap<String, f32>,
}

/// Input snapshot for synthesis mechanic
#[derive(Debug, Clone)]
pub struct SynthesisInput {
    /// Recipe being attempted
    pub recipe: Recipe,
    /// Provided ingredients
    pub ingredients: Vec<IngredientInput>,
    /// Synthesizer stats
    pub synthesizer: SynthesizerStats,
    /// Synthesis context
    pub context: SynthesisContext,
    /// Unlocked prerequisites
    pub unlocked: UnlockedPrerequisites,
    /// Random value for outcome determination (0.0-1.0)
    pub rng: f32,
    /// Current tick
    pub current_tick: u64,
}

/// Statistics about the synthesizer (crafter, researcher, etc.)
#[derive(Debug, Clone)]
pub struct SynthesizerStats {
    /// Entity ID
    pub entity_id: SynthesizerId,
    /// Skill level (0.0-1.0+)
    pub skill_level: f32,
    /// Luck modifier
    pub luck: f32,
    /// Quality bonus
    pub quality_bonus: f32,
    /// Specializations (category bonuses)
    pub specializations: HashMap<RecipeCategory, f32>,
}

impl Default for SynthesizerStats {
    fn default() -> Self {
        Self {
            entity_id: SynthesizerId("default".into()),
            skill_level: 0.5,
            luck: 0.0,
            quality_bonus: 0.0,
            specializations: HashMap::new(),
        }
    }
}

/// Unlocked prerequisites state
#[derive(Debug, Clone, Default)]
pub struct UnlockedPrerequisites {
    /// Unlocked technologies
    pub techs: HashSet<TechId>,
    /// Learned recipes
    pub recipes: HashSet<RecipeId>,
    /// Possessed items (id -> quantity)
    pub items: HashMap<IngredientId, u64>,
    /// Current level
    pub level: u32,
    /// Active flags
    pub flags: HashSet<FlagId>,
}

impl UnlockedPrerequisites {
    /// Check if a tech is unlocked
    pub fn has_tech(&self, id: &TechId) -> bool {
        self.techs.contains(id)
    }

    /// Check if a recipe is learned
    pub fn has_recipe(&self, id: &RecipeId) -> bool {
        self.recipes.contains(id)
    }

    /// Check if has item quantity
    pub fn has_item(&self, id: &IngredientId, quantity: u64) -> bool {
        self.items.get(id).copied().unwrap_or(0) >= quantity
    }

    /// Check if has flag
    pub fn has_flag(&self, id: &FlagId) -> bool {
        self.flags.contains(id)
    }

    /// Check if meets level
    pub fn meets_level(&self, min_level: u32) -> bool {
        self.level >= min_level
    }
}

// ============================================================================
// State
// ============================================================================

/// State output from synthesis mechanic
#[derive(Debug, Clone, Default)]
pub struct SynthesisState {
    /// Calculated success rate
    pub success_rate: f32,
    /// Determined outcome
    pub outcome: Option<SynthesisOutcome>,
    /// Prerequisites satisfied?
    pub prerequisites_met: bool,
    /// Missing prerequisites (if any)
    pub missing_prerequisites: Vec<Prerequisite>,
    /// Last synthesis tick
    pub last_synthesis: u64,
    /// Synthesis history (recent outcomes)
    pub history: Vec<SynthesisHistoryEntry>,
}

/// Entry in synthesis history
#[derive(Debug, Clone)]
pub struct SynthesisHistoryEntry {
    /// Recipe attempted
    pub recipe_id: RecipeId,
    /// Outcome type
    pub outcome_type: OutcomeType,
    /// Quality achieved
    pub quality: Option<QualityLevel>,
    /// Tick when synthesized
    pub tick: u64,
}

/// Outcome type (for history tracking)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutcomeType {
    /// Normal success
    Success,
    /// Critical success
    CriticalSuccess,
    /// Partial success
    PartialSuccess,
    /// Unexpected outcome
    Unexpected,
    /// Transmutation
    Transmutation,
    /// Failure
    Failure,
}

impl From<&SynthesisOutcome> for OutcomeType {
    fn from(outcome: &SynthesisOutcome) -> Self {
        match outcome {
            SynthesisOutcome::Success { .. } => OutcomeType::Success,
            SynthesisOutcome::CriticalSuccess { .. } => OutcomeType::CriticalSuccess,
            SynthesisOutcome::PartialSuccess { .. } => OutcomeType::PartialSuccess,
            SynthesisOutcome::Unexpected { .. } => OutcomeType::Unexpected,
            SynthesisOutcome::Transmutation { .. } => OutcomeType::Transmutation,
            SynthesisOutcome::Failure { .. } => OutcomeType::Failure,
        }
    }
}

// ============================================================================
// Events
// ============================================================================

/// Events emitted by synthesis mechanic
#[derive(Debug, Clone, PartialEq)]
pub enum SynthesisEvent {
    /// Synthesis attempted
    SynthesisAttempted {
        /// Recipe ID
        recipe_id: RecipeId,
        /// Synthesizer
        synthesizer: SynthesizerId,
        /// Calculated success rate
        success_rate: f32,
    },

    /// Synthesis succeeded
    SynthesisSucceeded {
        /// Recipe ID
        recipe_id: RecipeId,
        /// Output produced
        output: SynthesisOutput,
        /// Quality achieved
        quality: QualityLevel,
    },

    /// Critical success
    CriticalSuccess {
        /// Recipe ID
        recipe_id: RecipeId,
        /// Bonuses received
        bonuses: Vec<SynthesisBonus>,
    },

    /// Partial success
    PartialSuccess {
        /// Recipe ID
        recipe_id: RecipeId,
        /// Defects on output
        defects: Vec<Defect>,
    },

    /// Unexpected outcome
    UnexpectedOutcome {
        /// Original recipe ID
        recipe_id: RecipeId,
        /// What was produced
        output: SynthesisOutput,
        /// Trigger
        trigger: UnexpectedTrigger,
        /// Was it beneficial?
        is_beneficial: bool,
    },

    /// Transmutation occurred
    TransmutationOccurred {
        /// Original recipe
        original_recipe: RecipeId,
        /// What was produced
        output: SynthesisOutput,
        /// Catalyst involved
        catalyst: Option<CatalystId>,
    },

    /// Synthesis failed
    SynthesisFailed {
        /// Recipe ID
        recipe_id: RecipeId,
        /// Failure reason
        reason: FailureReason,
        /// Materials consumed fraction
        consumption: f32,
    },

    /// Prerequisites not met
    PrerequisitesNotMet {
        /// Recipe ID
        recipe_id: RecipeId,
        /// Missing prerequisites
        missing: Vec<Prerequisite>,
    },

    /// Recipe discovered (hidden recipe found)
    RecipeDiscovered {
        /// Discovered recipe ID
        recipe_id: RecipeId,
        /// How it was discovered
        trigger: UnexpectedTrigger,
    },

    /// Trait inherited (fusion)
    TraitInherited {
        /// Output entity
        output_id: IngredientId,
        /// Inherited trait
        trait_id: TraitId,
        /// Source of trait
        source: IngredientId,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_level_ordering() {
        assert!(QualityLevel::Poor < QualityLevel::Common);
        assert!(QualityLevel::Legendary > QualityLevel::Epic);
    }

    #[test]
    fn test_quality_level_upgrade() {
        assert_eq!(QualityLevel::Common.upgrade(1), QualityLevel::Uncommon);
        assert_eq!(QualityLevel::Common.upgrade(-1), QualityLevel::Poor);
        assert_eq!(QualityLevel::Legendary.upgrade(5), QualityLevel::Legendary); // clamped
        assert_eq!(QualityLevel::Broken.upgrade(-5), QualityLevel::Broken); // clamped
    }

    #[test]
    fn test_prerequisite_constructors() {
        let tech = Prerequisite::tech("fire_magic");
        assert!(matches!(tech, Prerequisite::Tech { .. }));

        let level = Prerequisite::level(10);
        assert!(matches!(level, Prerequisite::Level { min_level: 10 }));

        let combined = Prerequisite::all(vec![Prerequisite::tech("magic"), Prerequisite::level(5)]);
        assert!(matches!(combined, Prerequisite::All { .. }));
    }

    #[test]
    fn test_ingredient_builder() {
        let ingredient = Ingredient::new("iron_ore", 5)
            .with_min_quality(QualityLevel::Common)
            .as_catalyst();

        assert_eq!(ingredient.quantity, 5);
        assert_eq!(ingredient.min_quality, Some(QualityLevel::Common));
        assert!(!ingredient.consumed);
    }

    #[test]
    fn test_recipe_builder() {
        let recipe = Recipe::new("iron_sword", "Iron Sword")
            .with_category(RecipeCategory::Crafting)
            .with_difficulty(1.5)
            .with_ingredient(Ingredient::new("iron_ingot", 3))
            .with_prerequisite(Prerequisite::tech("smithing"))
            .with_base_quality(QualityLevel::Common);

        assert_eq!(recipe.difficulty, 1.5);
        assert_eq!(recipe.ingredients.len(), 1);
        assert_eq!(recipe.prerequisites.len(), 1);
    }

    #[test]
    fn test_synthesis_outcome_helpers() {
        let success = SynthesisOutcome::Success {
            output: SynthesisOutput::new("sword", 1),
            quality: QualityLevel::Rare,
            byproducts: vec![],
        };

        assert!(success.is_success());
        assert!(success.has_output());
        assert_eq!(success.quality(), Some(QualityLevel::Rare));

        let failure = SynthesisOutcome::Failure {
            reason: FailureReason::BadLuck,
            consumption: 0.5,
            salvage: vec![],
        };

        assert!(!failure.is_success());
        assert!(!failure.has_output());
        assert_eq!(failure.quality(), None);
    }

    #[test]
    fn test_unlocked_prerequisites() {
        let mut unlocked = UnlockedPrerequisites::default();
        unlocked.techs.insert(TechId("smithing".into()));
        unlocked.level = 10;
        unlocked.items.insert(IngredientId("gold".into()), 100);

        assert!(unlocked.has_tech(&TechId("smithing".into())));
        assert!(!unlocked.has_tech(&TechId("magic".into())));
        assert!(unlocked.meets_level(10));
        assert!(!unlocked.meets_level(11));
        assert!(unlocked.has_item(&IngredientId("gold".into()), 50));
        assert!(!unlocked.has_item(&IngredientId("gold".into()), 150));
    }

    #[test]
    fn test_outcome_table_builder() {
        let table = OutcomeTable::new()
            .with_critical_threshold(0.9)
            .with_partial_threshold(0.4);

        assert_eq!(table.critical_threshold, 0.9);
        assert_eq!(table.partial_threshold, 0.4);
    }

    #[test]
    fn test_synthesis_cost() {
        let cost = SynthesisCost::time(100).with_custom("mana", 50.0);

        assert_eq!(cost.time, 100);
        assert_eq!(cost.custom.get("mana"), Some(&50.0));
    }
}
