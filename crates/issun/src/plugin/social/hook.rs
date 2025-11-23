//! Hook trait for game-specific social network logic
//!
//! This provides the 20% extension point for custom game behavior.

use async_trait::async_trait;

use super::types::{CentralityMetrics, Faction, FactionId, MemberId, PoliticalAction};
use crate::context::ResourceContext;

/// Hook trait for social network customization
///
/// Implement this trait to add game-specific logic for:
/// - Custom political action validation
/// - Shadow leader detection notifications
/// - Faction dynamics (formation, split, merge)
/// - Influence and gossip propagation
#[async_trait]
pub trait SocialHook: Send + Sync {
    /// Notification when centrality metrics are calculated
    ///
    /// Use this to:
    /// - Update UI with influence indicators
    /// - Log network changes for analytics
    /// - Trigger events based on influence changes
    async fn on_centrality_calculated(
        &self,
        _faction_id: &FactionId,
        _member_id: &MemberId,
        _metrics: &CentralityMetrics,
        _resources: &mut ResourceContext,
    ) {
        // Default: no-op
    }

    /// Notification when shadow leader (KingMaker) is detected
    ///
    /// Use this to:
    /// - Display special UI for KingMakers
    /// - Grant special abilities or perks
    /// - Unlock hidden storylines
    /// - Create targeted assassination missions
    async fn on_shadow_leader_detected(
        &self,
        _faction_id: &FactionId,
        _member_id: &MemberId,
        _influence_score: f32,
        _resources: &mut ResourceContext,
    ) {
        // Default: no-op
    }

    /// Validation before political action execution
    ///
    /// Use this to:
    /// - Check game-specific preconditions
    /// - Apply custom costs (items, money, etc.)
    /// - Implement cooldown systems
    /// - Block actions based on game state
    ///
    /// # Returns
    ///
    /// `Ok(())` to allow action, `Err(reason)` to deny
    async fn on_political_action_requested(
        &self,
        _faction_id: &FactionId,
        _actor_id: &MemberId,
        _action: &PoliticalAction,
        _resources: &mut ResourceContext,
    ) -> Result<(), String> {
        // Default: always allow
        Ok(())
    }

    /// Notification after political action execution
    ///
    /// Use this to:
    /// - Apply game effects (buffs, items, etc.)
    /// - Update quest objectives
    /// - Trigger narrative events
    /// - Award achievements
    async fn on_political_action_executed(
        &self,
        _faction_id: &FactionId,
        _actor_id: &MemberId,
        _action: &PoliticalAction,
        _success: bool,
        _resources: &mut ResourceContext,
    ) {
        // Default: no-op
    }

    /// Notification when favor is exchanged
    ///
    /// Use this to:
    /// - Show reputation changes in UI
    /// - Update relationship meters
    /// - Trigger dialogue events
    async fn on_favor_exchanged(
        &self,
        _faction_id: &FactionId,
        _grantor: &MemberId,
        _recipient: &MemberId,
        _favor_value: f32,
        _resources: &mut ResourceContext,
    ) {
        // Default: no-op
    }

    /// Notification when secret is shared
    ///
    /// Use this to:
    /// - Create mutual dependency mechanics
    /// - Enable blackmail systems
    /// - Track secret knowledge for quests
    /// - Update trust/threat indicators
    async fn on_secret_shared(
        &self,
        _faction_id: &FactionId,
        _sharer: &MemberId,
        _receiver: &MemberId,
        _secret_id: &str,
        _sensitivity: f32,
        _resources: &mut ResourceContext,
    ) {
        // Default: no-op
    }

    /// Notification when gossip spreads
    ///
    /// Use this to:
    /// - Display rumor in UI/dialogue
    /// - Affect reputation systems
    /// - Create misinformation mechanics
    /// - Trigger social events
    async fn on_gossip_spread(
        &self,
        _faction_id: &FactionId,
        _spreader: &MemberId,
        _about: &MemberId,
        _content: &str,
        _is_positive: bool,
        _reached_count: usize,
        _resources: &mut ResourceContext,
    ) {
        // Default: no-op
    }

    /// Notification when new faction is formed
    ///
    /// Use this to:
    /// - Display faction creation cutscene
    /// - Update world map/UI
    /// - Trigger political events
    /// - Award achievements
    async fn on_faction_formed(
        &self,
        _org_faction_id: &FactionId,
        _new_faction: &Faction,
        _founding_members: &[MemberId],
        _resources: &mut ResourceContext,
    ) {
        // Default: no-op
    }

    /// Notification when faction splits
    ///
    /// Use this to:
    /// - Trigger civil war events
    /// - Display faction split cutscene
    /// - Update diplomatic status
    /// - Create new questlines
    async fn on_faction_split(
        &self,
        _org_faction_id: &FactionId,
        _original_faction_id: &FactionId,
        _new_faction_ids: &[FactionId],
        _reason: &str,
        _resources: &mut ResourceContext,
    ) {
        // Default: no-op
    }

    /// Notification when factions merge
    ///
    /// Use this to:
    /// - Display alliance formation cutscene
    /// - Update diplomatic relations
    /// - Combine resources/territories
    /// - Trigger power shift events
    async fn on_faction_merged(
        &self,
        _org_faction_id: &FactionId,
        _merged_faction_id: &FactionId,
        _source_faction_ids: &[FactionId],
        _total_members: usize,
        _resources: &mut ResourceContext,
    ) {
        // Default: no-op
    }

    /// Notification when trust relationship decays
    ///
    /// Use this to:
    /// - Show relationship deterioration in UI
    /// - Trigger reconciliation quests
    /// - Update dialogue options
    async fn on_trust_decayed(
        &self,
        _faction_id: &FactionId,
        _from: &MemberId,
        _to: &MemberId,
        _new_strength: f32,
        _resources: &mut ResourceContext,
    ) {
        // Default: no-op
    }

    /// Notification when favor expires
    ///
    /// Use this to:
    /// - Clear favor-related UI elements
    /// - Update relationship status
    /// - Trigger "debt forgiven" events
    async fn on_favor_expired(
        &self,
        _faction_id: &FactionId,
        _creditor: &MemberId,
        _debtor: &MemberId,
        _favor_value: f32,
        _resources: &mut ResourceContext,
    ) {
        // Default: no-op
    }
}

/// Default hook implementation (all no-op)
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultSocialHook;

#[async_trait]
impl SocialHook for DefaultSocialHook {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_hook() {
        let hook = DefaultSocialHook;
        let mut resources = ResourceContext::new();

        // All hooks should be no-op and not panic
        hook.on_centrality_calculated(
            &"faction1".to_string(),
            &"member1".to_string(),
            &CentralityMetrics::default(),
            &mut resources,
        )
        .await;

        hook.on_shadow_leader_detected(
            &"faction1".to_string(),
            &"kingmaker".to_string(),
            0.85,
            &mut resources,
        )
        .await;

        let action = PoliticalAction::GrantFavor {
            target: "member2".to_string(),
            favor_value: 1.0,
        };

        let result = hook
            .on_political_action_requested(
                &"faction1".to_string(),
                &"member1".to_string(),
                &action,
                &mut resources,
            )
            .await;

        assert!(result.is_ok());
    }
}
