//! Hook trait for game-specific culture logic
//!
//! This provides the 20% extension point for custom game behavior.

use async_trait::async_trait;

use super::types::{Alignment, FactionId, Member, MemberId};
use crate::context::ResourceContext;

/// Hook trait for culture customization
///
/// Implement this trait to add game-specific logic for:
/// - Custom alignment modifiers (based on game events, items, etc.)
/// - Member breakdown handling (removal, debuffs, etc.)
/// - Fanaticism effects (fearlessness, special abilities, etc.)
/// - Culture change triggers (based on events)
#[async_trait]
pub trait CultureHook: Send + Sync {
    /// Notification when alignment is checked
    ///
    /// Use this to:
    /// - Log alignment changes for analytics
    /// - Update UI with member status
    /// - Trigger game events based on alignment
    async fn on_alignment_checked(
        &self,
        _faction_id: &FactionId,
        _member_id: &MemberId,
        _alignment: &Alignment,
        _resources: &mut ResourceContext,
    ) {
        // Default: no-op
    }

    /// Notification when member accumulates stress
    ///
    /// Use this to:
    /// - Show stress indicator in UI
    /// - Trigger stress-relief events
    /// - Award stress management bonuses
    async fn on_stress_accumulated(
        &self,
        _faction_id: &FactionId,
        _member_id: &MemberId,
        _new_stress: f32,
        _resources: &mut ResourceContext,
    ) {
        // Default: no-op
    }

    /// Notification when member gains fervor
    ///
    /// Use this to:
    /// - Show fervor indicator in UI
    /// - Grant fervor-based bonuses
    /// - Unlock fanatical abilities
    async fn on_fervor_increased(
        &self,
        _faction_id: &FactionId,
        _member_id: &MemberId,
        _new_fervor: f32,
        _resources: &mut ResourceContext,
    ) {
        // Default: no-op
    }

    /// Handle member breakdown (game-specific logic)
    ///
    /// **Examples**:
    /// - Remove member from organization
    /// - Apply debuffs (reduced stats)
    /// - Trigger mental health event
    /// - Force member to quit/rebel
    ///
    /// # Returns
    ///
    /// `true` if member should be removed from organization
    async fn on_member_breakdown(
        &self,
        _faction_id: &FactionId,
        _member: &Member,
        _resources: &mut ResourceContext,
    ) -> bool {
        // Default: remove member (breakdown = quit)
        true
    }

    /// Handle member fanaticism (game-specific logic)
    ///
    /// **Examples**:
    /// - Grant fearlessness buff
    /// - Enable martyrdom abilities
    /// - Ignore self-preservation in combat
    /// - Increase damage/loyalty dramatically
    ///
    /// This is called once when fervor crosses threshold.
    async fn on_member_fanaticized(
        &self,
        _faction_id: &FactionId,
        _member: &Member,
        _resources: &mut ResourceContext,
    ) {
        // Default: no-op (hook decides what fanaticism means)
    }

    /// Check if custom conditions prevent culture tag addition
    ///
    /// Use this to:
    /// - Require specific items/events to enable culture tags
    /// - Prevent conflicting culture tags
    /// - Gate culture tags behind progression
    ///
    /// # Returns
    ///
    /// `true` if tag can be added, `false` to prevent
    async fn can_add_culture_tag(
        &self,
        _faction_id: &FactionId,
        _tag: &super::types::CultureTag,
        _resources: &mut ResourceContext,
    ) -> bool {
        // Default: always allow
        true
    }

    /// Notification when culture tag is added
    ///
    /// Use this to:
    /// - Update UI
    /// - Trigger narrative events
    /// - Apply global faction buffs
    async fn on_culture_tag_added(
        &self,
        _faction_id: &FactionId,
        _tag: &super::types::CultureTag,
        _resources: &mut ResourceContext,
    ) {
        // Default: no-op
    }

    /// Notification when culture tag is removed
    ///
    /// Use this to:
    /// - Update UI
    /// - Remove associated buffs
    /// - Trigger culture shift events
    async fn on_culture_tag_removed(
        &self,
        _faction_id: &FactionId,
        _tag: &super::types::CultureTag,
        _resources: &mut ResourceContext,
    ) {
        // Default: no-op
    }
}

/// Default no-op hook implementation
///
/// Use this for testing or when you don't need custom behavior.
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultCultureHook;

#[async_trait]
impl CultureHook for DefaultCultureHook {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_hook_no_op() {
        let hook = DefaultCultureHook;
        let mut resources = ResourceContext::new();
        let faction_id = "faction_a".to_string();
        let member_id = "m1".to_string();
        let alignment = Alignment::Neutral;

        // Should not panic
        hook.on_alignment_checked(&faction_id, &member_id, &alignment, &mut resources)
            .await;
        hook.on_stress_accumulated(&faction_id, &member_id, 0.7, &mut resources)
            .await;
        hook.on_fervor_increased(&faction_id, &member_id, 0.8, &mut resources)
            .await;
    }

    #[tokio::test]
    async fn test_default_breakdown_removes_member() {
        let hook = DefaultCultureHook;
        let mut resources = ResourceContext::new();
        let faction_id = "faction_a".to_string();
        let member = Member::new("m1", "Test").with_stress(0.95);

        let should_remove = hook
            .on_member_breakdown(&faction_id, &member, &mut resources)
            .await;

        assert!(should_remove);
    }

    #[tokio::test]
    async fn test_can_add_culture_tag_default() {
        let hook = DefaultCultureHook;
        let mut resources = ResourceContext::new();
        let faction_id = "faction_a".to_string();
        let tag = super::super::types::CultureTag::RiskTaking;

        let can_add = hook
            .can_add_culture_tag(&faction_id, &tag, &mut resources)
            .await;

        assert!(can_add);
    }
}
