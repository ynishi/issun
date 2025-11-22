use super::models::{Rumor, RumorEffect, RumorId};
use async_trait::async_trait;
use issun::prelude::ResourceContext;

/// Hook for game-specific rumor behavior
#[async_trait]
pub trait RumorHook: Send + Sync {
    /// Called before rumor is applied (validation)
    async fn on_before_apply(
        &self,
        _rumor: &Rumor,
        _resources: &ResourceContext,
    ) -> std::result::Result<(), String> {
        // Default: allow
        Ok(())
    }

    /// Called after rumor effect is calculated (modify effect)
    async fn on_rumor_applied(
        &self,
        _rumor: &Rumor,
        _effect_applied: &RumorEffect,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }

    /// Called when rumor expires
    async fn on_rumor_expired(&self, _rumor_id: &RumorId, _resources: &mut ResourceContext) {
        // Default: do nothing
    }

    /// Calculate migration target (game-specific logic)
    async fn calculate_migration_target(
        &self,
        _from_district: usize,
        _resources: &ResourceContext,
    ) -> Option<usize> {
        // Default: random or most healthy district
        None
    }
}

/// Default no-op implementation
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultRumorHook;

#[async_trait]
impl RumorHook for DefaultRumorHook {}
