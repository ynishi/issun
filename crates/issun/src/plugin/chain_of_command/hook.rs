//! Hook trait for game-specific chain of command logic (Phase 5)
//!
//! This provides the 20% extension point for custom game behavior.

use async_trait::async_trait;

use super::rank_definitions::RankDefinition;
use super::types::{FactionId, Member, MemberId, Order, RankId};
use crate::context::ResourceContext;

/// Hook trait for chain of command customization
///
/// Implement this trait to add game-specific logic for:
/// - Custom promotion conditions (combat victories, quests, etc.)
/// - Order execution logic (move units, craft items, etc.)
/// - Morale/loyalty modifiers based on game events
#[async_trait]
pub trait ChainOfCommandHook: Send + Sync {
    /// Check game-specific promotion conditions
    ///
    /// **Examples**:
    /// - Combat victories required
    /// - Quest completions
    /// - Peer approval rating
    /// - Skill level requirements
    ///
    /// # Returns
    ///
    /// `true` if custom conditions are met, `false` otherwise
    async fn can_promote_custom(
        &self,
        _member: &Member,
        _new_rank: &RankDefinition,
        _resources: &mut ResourceContext,
    ) -> bool {
        // Default: always allow (framework handles basic checks)
        true
    }

    /// Notification when member is promoted
    ///
    /// Use this to trigger game events like:
    /// - Award promotion bonus
    /// - Update UI
    /// - Send notifications to player
    async fn on_member_promoted(
        &self,
        _faction_id: &FactionId,
        _member_id: &MemberId,
        _new_rank: &RankId,
        _resources: &mut ResourceContext,
    ) {
        // Default: no-op
    }

    /// Execute order (game-specific logic)
    ///
    /// **Examples**:
    /// - Move unit on map
    /// - Start crafting
    /// - Engage in combat
    /// - Gather resources
    async fn execute_order(
        &self,
        _faction_id: &FactionId,
        _member_id: &MemberId,
        _order: &Order,
        _resources: &mut ResourceContext,
    ) {
        // Default: no-op
    }

    /// Notification when order is refused
    ///
    /// Use this to handle refusals:
    /// - Decrease superior's morale
    /// - Trigger disciplinary action
    /// - Update UI with refusal message
    async fn on_order_refused(
        &self,
        _faction_id: &FactionId,
        _member_id: &MemberId,
        _order: &Order,
        _resources: &mut ResourceContext,
    ) {
        // Default: no-op
    }
}

/// Default no-op hook implementation
///
/// Use this for testing or when you don't need custom behavior.
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultChainOfCommandHook;

#[async_trait]
impl ChainOfCommandHook for DefaultChainOfCommandHook {}
