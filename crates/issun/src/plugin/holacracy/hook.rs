//! Hook trait for holacracy plugin
//!
//! Provides extension points for customizing task assignment, bidding, and
//! organizational behavior through the 80/20 principle.

use crate::context::ResourceContext;
use async_trait::async_trait;

use super::types::*;

/// Hook trait for holacracy plugin (20% extension points)
///
/// Implement this trait to customize:
/// - Bid validation and approval
/// - Task assignment logic
/// - Role assignment rules
/// - Custom reactions to organizational events
#[async_trait]
pub trait HolacracyHook: Send + Sync {
    /// Called when a bid is submitted (before acceptance)
    ///
    /// Return Err(reason) to reject the bid
    async fn on_bid_submitted(
        &self,
        _task_id: &TaskId,
        _member_id: &MemberId,
        _score: &BidScore,
        _resources: &mut ResourceContext,
    ) -> Result<(), String> {
        Ok(())
    }

    /// Called after a bid is accepted
    async fn on_bid_accepted(
        &self,
        _task_id: &TaskId,
        _member_id: &MemberId,
        _score: &BidScore,
        _resources: &mut ResourceContext,
    ) {
    }

    /// Called when a task is assigned (before execution)
    ///
    /// Return Err(reason) to prevent assignment
    async fn on_task_assign_requested(
        &self,
        _task_id: &TaskId,
        _member_id: &MemberId,
        _resources: &mut ResourceContext,
    ) -> Result<(), String> {
        Ok(())
    }

    /// Called after a task is successfully assigned
    async fn on_task_assigned(
        &self,
        _task_id: &TaskId,
        _member_id: &MemberId,
        _resources: &mut ResourceContext,
    ) {
    }

    /// Called when a task is completed
    async fn on_task_completed(
        &self,
        _task_id: &TaskId,
        _member_id: &MemberId,
        _resources: &mut ResourceContext,
    ) {
    }

    /// Called when a task is cancelled
    async fn on_task_cancelled(
        &self,
        _task_id: &TaskId,
        _reason: &str,
        _resources: &mut ResourceContext,
    ) {
    }

    /// Called when a role is assigned (before execution)
    ///
    /// Return Err(reason) to prevent role assignment
    async fn on_role_assign_requested(
        &self,
        _circle_id: &CircleId,
        _role_id: &RoleId,
        _member_id: &MemberId,
        _resources: &mut ResourceContext,
    ) -> Result<(), String> {
        Ok(())
    }

    /// Called after a role is successfully assigned
    async fn on_role_assigned(
        &self,
        _circle_id: &CircleId,
        _role_id: &RoleId,
        _member_id: &MemberId,
        _resources: &mut ResourceContext,
    ) {
    }

    /// Called when a role is unassigned
    async fn on_role_unassigned(
        &self,
        _circle_id: &CircleId,
        _role_id: &RoleId,
        _previous_holder: &MemberId,
        _resources: &mut ResourceContext,
    ) {
    }

    /// Called when bidding period expires
    ///
    /// Return Some(member_id) to override automatic assignment
    async fn on_bidding_completed(
        &self,
        _task_id: &TaskId,
        _bids: &[&Bid],
        _resources: &mut ResourceContext,
    ) -> Option<MemberId> {
        None
    }

    /// Called when a member is added
    async fn on_member_added(&self, _member_id: &MemberId, _resources: &mut ResourceContext) {}

    /// Called when a member is removed
    async fn on_member_removed(&self, _member_id: &MemberId, _resources: &mut ResourceContext) {}

    /// Called when a circle is created
    async fn on_circle_created(&self, _circle_id: &CircleId, _resources: &mut ResourceContext) {}
}

/// Default no-op hook implementation
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultHolacracyHook;

#[async_trait]
impl HolacracyHook for DefaultHolacracyHook {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_hook_creation() {
        let _hook = DefaultHolacracyHook;
        // Hook created successfully - compilation is the test
    }

    #[tokio::test]
    async fn test_default_hook_on_bid_submitted() {
        let hook = DefaultHolacracyHook;
        let mut resources = ResourceContext::new();
        let score = BidScore::new(0.8, 0.6, 0.9);

        let result = hook
            .on_bid_submitted(
                &"t1".to_string(),
                &"alice".to_string(),
                &score,
                &mut resources,
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_default_hook_on_task_assign_requested() {
        let hook = DefaultHolacracyHook;
        let mut resources = ResourceContext::new();

        let result = hook
            .on_task_assign_requested(&"t1".to_string(), &"alice".to_string(), &mut resources)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_default_hook_on_role_assign_requested() {
        let hook = DefaultHolacracyHook;
        let mut resources = ResourceContext::new();

        let result = hook
            .on_role_assign_requested(
                &"c1".to_string(),
                &"r1".to_string(),
                &"alice".to_string(),
                &mut resources,
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_default_hook_on_bidding_completed() {
        let hook = DefaultHolacracyHook;
        let mut resources = ResourceContext::new();
        let bids: Vec<&Bid> = Vec::new();

        let result = hook
            .on_bidding_completed(&"t1".to_string(), &bids, &mut resources)
            .await;
        assert!(result.is_none());
    }
}
