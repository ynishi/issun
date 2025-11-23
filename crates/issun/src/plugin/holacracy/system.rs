//! System for processing holacracy events
//!
//! This system orchestrates task assignment, bidding, and role management
//! by coordinating between service functions, hooks, and state management.

use crate::context::ResourceContext;
use crate::event::EventBus;

use super::config::HolacracyConfig;
use super::events::*;
use super::hook::HolacracyHook;
use super::service::TaskAssignmentService;
use super::state::HolacracyState;
use super::types::*;

/// System for processing holacracy events
///
/// This system:
/// 1. Listens for command events (TaskAddRequested, BidSubmitRequested, etc.)
/// 2. Validates via hook
/// 3. Executes logic via TaskAssignmentService
/// 4. Updates HolacracyState
/// 5. Publishes state events (TaskAddedEvent, BidSubmittedEvent, etc.)
pub struct HolacracySystem<H: HolacracyHook> {
    hook: H,
}

impl<H: HolacracyHook> HolacracySystem<H> {
    /// Create a new holacracy system with a custom hook
    pub fn new(hook: H) -> Self {
        Self { hook }
    }

    /// Process all pending holacracy events
    ///
    /// This should be called once per game tick/frame.
    pub async fn process_events(&mut self, resources: &mut ResourceContext) {
        // Collect events from EventBus
        let task_add_requests = self.collect_events::<TaskAddRequested>(resources).await;
        let bidding_start_requests = self
            .collect_events::<BiddingStartRequested>(resources)
            .await;
        let bid_submit_requests = self.collect_events::<BidSubmitRequested>(resources).await;
        let task_assign_requests = self.collect_events::<TaskAssignRequested>(resources).await;
        let task_complete_requests = self
            .collect_events::<TaskCompleteRequested>(resources)
            .await;
        let task_cancel_requests = self.collect_events::<TaskCancelRequested>(resources).await;
        let member_add_requests = self.collect_events::<MemberAddRequested>(resources).await;
        let member_remove_requests = self
            .collect_events::<MemberRemoveRequested>(resources)
            .await;
        let role_assign_requests = self.collect_events::<RoleAssignRequested>(resources).await;
        let role_unassign_requests = self
            .collect_events::<RoleUnassignRequested>(resources)
            .await;
        let circle_create_requests = self
            .collect_events::<CircleCreateRequested>(resources)
            .await;
        let bidding_process_requests = self
            .collect_events::<BiddingProcessRequested>(resources)
            .await;

        // Process events
        for request in task_add_requests {
            self.process_task_add(request, resources).await;
        }

        for request in bidding_start_requests {
            self.process_bidding_start(request, resources).await;
        }

        for request in bid_submit_requests {
            self.process_bid_submit(request, resources).await;
        }

        for request in task_assign_requests {
            self.process_task_assign(request, resources).await;
        }

        for request in task_complete_requests {
            self.process_task_complete(request, resources).await;
        }

        for request in task_cancel_requests {
            self.process_task_cancel(request, resources).await;
        }

        for request in member_add_requests {
            self.process_member_add(request, resources).await;
        }

        for request in member_remove_requests {
            self.process_member_remove(request, resources).await;
        }

        for request in role_assign_requests {
            self.process_role_assign(request, resources).await;
        }

        for request in role_unassign_requests {
            self.process_role_unassign(request, resources).await;
        }

        for request in circle_create_requests {
            self.process_circle_create(request, resources).await;
        }

        for request in bidding_process_requests {
            self.process_bidding_period(request, resources).await;
        }
    }

    /// Collect events of a specific type from EventBus
    async fn collect_events<T: Clone + 'static + crate::event::Event>(
        &self,
        resources: &mut ResourceContext,
    ) -> Vec<T> {
        if let Some(mut bus) = resources.get_mut::<EventBus>().await {
            let reader = bus.reader::<T>();
            reader.iter().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Process task add request
    async fn process_task_add(
        &mut self,
        request: TaskAddRequested,
        resources: &mut ResourceContext,
    ) {
        let mut state = match resources.get_mut::<HolacracyState>().await {
            Some(s) => s,
            None => return,
        };

        let task_id = request.task.id.clone();
        if let Err(_e) = state.task_pool_mut().add_task(request.task) {
            return;
        }

        self.publish_event(TaskAddedEvent { task_id }, resources)
            .await;
    }

    /// Process bidding start request
    async fn process_bidding_start(
        &mut self,
        request: BiddingStartRequested,
        resources: &mut ResourceContext,
    ) {
        let mut state = match resources.get_mut::<HolacracyState>().await {
            Some(s) => s,
            None => return,
        };

        let current_turn = state.current_turn();
        if let Err(_e) = state
            .task_pool_mut()
            .start_bidding(&request.task_id, current_turn)
        {
            return;
        }

        self.publish_event(
            BiddingStartedEvent {
                task_id: request.task_id,
                started_at: current_turn,
            },
            resources,
        )
        .await;
    }

    /// Process bid submit request
    async fn process_bid_submit(
        &mut self,
        request: BidSubmitRequested,
        resources: &mut ResourceContext,
    ) {
        let config = match resources.get::<HolacracyConfig>().await {
            Some(c) => c.clone(),
            None => return,
        };

        let mut state = match resources.get_mut::<HolacracyState>().await {
            Some(s) => s,
            None => return,
        };

        let task = match state.task_pool().get_task(&request.bid.task_id) {
            Some(t) => t.clone(),
            None => return,
        };

        let member = match state.get_member(&request.bid.member_id) {
            Some(m) => m.clone(),
            None => return,
        };

        // Validate eligibility
        if let Err(reason) = TaskAssignmentService::can_bid_on_task(&member, &task, &config) {
            self.publish_event(
                BidRejectedEvent {
                    task_id: request.bid.task_id.clone(),
                    member_id: request.bid.member_id.clone(),
                    reason,
                },
                resources,
            )
            .await;
            return;
        }

        // Validate via hook
        let validation = self
            .hook
            .on_bid_submitted(
                &request.bid.task_id,
                &request.bid.member_id,
                &request.bid.score,
                resources,
            )
            .await;

        if let Err(reason) = validation {
            self.publish_event(
                BidRejectedEvent {
                    task_id: request.bid.task_id.clone(),
                    member_id: request.bid.member_id.clone(),
                    reason,
                },
                resources,
            )
            .await;
            return;
        }

        // Add bid
        if let Err(_e) = state.task_pool_mut().add_bid(request.bid.clone()) {
            return;
        }

        // Publish success event
        self.publish_event(
            BidSubmittedEvent {
                task_id: request.bid.task_id.clone(),
                member_id: request.bid.member_id.clone(),
                score: request.bid.score.clone(),
            },
            resources,
        )
        .await;

        // Notify hook
        self.hook
            .on_bid_accepted(
                &request.bid.task_id,
                &request.bid.member_id,
                &request.bid.score,
                resources,
            )
            .await;
    }

    /// Process task assign request
    async fn process_task_assign(
        &mut self,
        request: TaskAssignRequested,
        resources: &mut ResourceContext,
    ) {
        // Validate via hook
        let validation = self
            .hook
            .on_task_assign_requested(&request.task_id, &request.member_id, resources)
            .await;

        if let Err(reason) = validation {
            self.publish_event(
                TaskAssignmentFailedEvent {
                    task_id: request.task_id,
                    member_id: request.member_id,
                    reason,
                },
                resources,
            )
            .await;
            return;
        }

        let mut state = match resources.get_mut::<HolacracyState>().await {
            Some(s) => s,
            None => return,
        };

        // Assign task in pool
        if let Err(e) = state
            .task_pool_mut()
            .assign_task(&request.task_id, &request.member_id)
        {
            self.publish_event(
                TaskAssignmentFailedEvent {
                    task_id: request.task_id.clone(),
                    member_id: request.member_id.clone(),
                    reason: e.to_string(),
                },
                resources,
            )
            .await;
            return;
        }

        // Update member's assigned tasks
        if let Some(member) = state.get_member_mut(&request.member_id) {
            member.assign_task(request.task_id.clone());
        }

        // Publish success event
        self.publish_event(
            TaskAssignedEvent {
                task_id: request.task_id.clone(),
                member_id: request.member_id.clone(),
            },
            resources,
        )
        .await;

        // Notify hook
        self.hook
            .on_task_assigned(&request.task_id, &request.member_id, resources)
            .await;
    }

    /// Process task complete request
    async fn process_task_complete(
        &mut self,
        request: TaskCompleteRequested,
        resources: &mut ResourceContext,
    ) {
        let mut state = match resources.get_mut::<HolacracyState>().await {
            Some(s) => s,
            None => return,
        };

        let current_turn = state.current_turn();
        let task = match state.task_pool_mut().get_task_mut(&request.task_id) {
            Some(t) => t,
            None => return,
        };

        let member_id = match &task.assignee {
            Some(id) => id.clone(),
            None => return,
        };

        task.status = TaskStatus::Completed;

        // Remove from member's assigned tasks
        if let Some(member) = state.get_member_mut(&member_id) {
            member.unassign_task(&request.task_id);
        }

        // Publish event
        self.publish_event(
            TaskCompletedEvent {
                task_id: request.task_id.clone(),
                completed_by: member_id.clone(),
                completed_at: current_turn,
            },
            resources,
        )
        .await;

        // Notify hook
        self.hook
            .on_task_completed(&request.task_id, &member_id, resources)
            .await;
    }

    /// Process task cancel request
    async fn process_task_cancel(
        &mut self,
        request: TaskCancelRequested,
        resources: &mut ResourceContext,
    ) {
        let mut state = match resources.get_mut::<HolacracyState>().await {
            Some(s) => s,
            None => return,
        };

        // Get assignee before mutating task
        let assignee = state
            .task_pool()
            .get_task(&request.task_id)
            .and_then(|t| t.assignee.clone());

        // If assigned, remove from member
        if let Some(member_id) = assignee {
            if let Some(member) = state.get_member_mut(&member_id) {
                member.unassign_task(&request.task_id);
            }
        }

        // Now update task status
        if let Some(task) = state.task_pool_mut().get_task_mut(&request.task_id) {
            task.status = TaskStatus::Cancelled;
        }

        // Publish event
        self.publish_event(
            TaskCancelledEvent {
                task_id: request.task_id.clone(),
                reason: request.reason.clone(),
            },
            resources,
        )
        .await;

        // Notify hook
        self.hook
            .on_task_cancelled(&request.task_id, &request.reason, resources)
            .await;
    }

    /// Process member add request
    async fn process_member_add(
        &mut self,
        request: MemberAddRequested,
        resources: &mut ResourceContext,
    ) {
        let mut state = match resources.get_mut::<HolacracyState>().await {
            Some(s) => s,
            None => return,
        };

        let member_id = request.member.id.clone();
        state.add_member(request.member);

        self.publish_event(
            MemberAddedEvent {
                member_id: member_id.clone(),
            },
            resources,
        )
        .await;

        self.hook.on_member_added(&member_id, resources).await;
    }

    /// Process member remove request
    async fn process_member_remove(
        &mut self,
        request: MemberRemoveRequested,
        resources: &mut ResourceContext,
    ) {
        let mut state = match resources.get_mut::<HolacracyState>().await {
            Some(s) => s,
            None => return,
        };

        if state.remove_member(&request.member_id).is_some() {
            self.publish_event(
                MemberRemovedEvent {
                    member_id: request.member_id.clone(),
                },
                resources,
            )
            .await;

            self.hook
                .on_member_removed(&request.member_id, resources)
                .await;
        }
    }

    /// Process role assign request
    async fn process_role_assign(
        &mut self,
        request: RoleAssignRequested,
        resources: &mut ResourceContext,
    ) {
        // Validate via hook
        let validation = self
            .hook
            .on_role_assign_requested(
                &request.circle_id,
                &request.role_id,
                &request.member_id,
                resources,
            )
            .await;

        if let Err(reason) = validation {
            self.publish_event(
                RoleAssignmentFailedEvent {
                    circle_id: request.circle_id,
                    role_id: request.role_id,
                    member_id: request.member_id,
                    reason,
                },
                resources,
            )
            .await;
            return;
        }

        let mut state = match resources.get_mut::<HolacracyState>().await {
            Some(s) => s,
            None => return,
        };

        // Get circle and role
        let circle = match state.get_circle_mut(&request.circle_id) {
            Some(c) => c,
            None => return,
        };

        let role = match circle.get_role(&request.role_id) {
            Some(r) => r.clone(),
            None => return,
        };

        // Check if role is already filled
        if role.is_filled() {
            self.publish_event(
                RoleAssignmentFailedEvent {
                    circle_id: request.circle_id,
                    role_id: request.role_id,
                    member_id: request.member_id,
                    reason: "Role already filled".to_string(),
                },
                resources,
            )
            .await;
            return;
        }

        // Assign role in circle
        if let Some(circle) = state.get_circle_mut(&request.circle_id) {
            if let Some(role) = circle.roles.get_mut(&request.role_id) {
                role.current_holder = Some(request.member_id.clone());
            }
        }

        // Add role to member
        if let Some(member) = state.get_member_mut(&request.member_id) {
            member.add_role(request.role_id.clone());
        }

        // Publish event
        self.publish_event(
            RoleAssignedEvent {
                circle_id: request.circle_id.clone(),
                role_id: request.role_id.clone(),
                member_id: request.member_id.clone(),
            },
            resources,
        )
        .await;

        // Notify hook
        self.hook
            .on_role_assigned(
                &request.circle_id,
                &request.role_id,
                &request.member_id,
                resources,
            )
            .await;
    }

    /// Process role unassign request
    async fn process_role_unassign(
        &mut self,
        request: RoleUnassignRequested,
        resources: &mut ResourceContext,
    ) {
        let mut state = match resources.get_mut::<HolacracyState>().await {
            Some(s) => s,
            None => return,
        };

        // Get previous holder
        let previous_holder = {
            let circle = match state.get_circle(&request.circle_id) {
                Some(c) => c,
                None => return,
            };

            let role = match circle.get_role(&request.role_id) {
                Some(r) => r,
                None => return,
            };

            match &role.current_holder {
                Some(h) => h.clone(),
                None => return, // Role not filled
            }
        };

        // Unassign role in circle
        if let Some(circle) = state.get_circle_mut(&request.circle_id) {
            if let Some(role) = circle.roles.get_mut(&request.role_id) {
                role.current_holder = None;
            }
        }

        // Remove role from member
        if let Some(member) = state.get_member_mut(&previous_holder) {
            member.remove_role(&request.role_id);
        }

        // Publish event
        self.publish_event(
            RoleUnassignedEvent {
                circle_id: request.circle_id.clone(),
                role_id: request.role_id.clone(),
                previous_holder: previous_holder.clone(),
            },
            resources,
        )
        .await;

        // Notify hook
        self.hook
            .on_role_unassigned(
                &request.circle_id,
                &request.role_id,
                &previous_holder,
                resources,
            )
            .await;
    }

    /// Process circle create request
    async fn process_circle_create(
        &mut self,
        request: CircleCreateRequested,
        resources: &mut ResourceContext,
    ) {
        let mut state = match resources.get_mut::<HolacracyState>().await {
            Some(s) => s,
            None => return,
        };

        let circle_id = request.circle.id.clone();
        state.add_circle(request.circle);

        self.publish_event(
            CircleCreatedEvent {
                circle_id: circle_id.clone(),
            },
            resources,
        )
        .await;

        self.hook.on_circle_created(&circle_id, resources).await;
    }

    /// Process bidding period expiration
    async fn process_bidding_period(
        &mut self,
        request: BiddingProcessRequested,
        resources: &mut ResourceContext,
    ) {
        let config = match resources.get::<HolacracyConfig>().await {
            Some(c) => c.clone(),
            None => return,
        };

        let state = match resources.get::<HolacracyState>().await {
            Some(s) => s.clone(),
            None => return,
        };

        let task_pool = state.task_pool();

        // Find tasks with expired bidding
        let expired_tasks: Vec<_> = task_pool
            .available_tasks()
            .iter()
            .filter(|t| t.status == TaskStatus::Bidding)
            .filter(|t| task_pool.is_bidding_expired(&t.id, request.current_turn, &config))
            .map(|t| t.id.clone())
            .collect();

        for task_id in expired_tasks {
            let bids = task_pool.get_bids(&task_id);
            let bid_count = bids.len();

            // Check hook override
            let hook_override = self
                .hook
                .on_bidding_completed(&task_id, &bids, resources)
                .await;

            let assigned_to = if let Some(member_id) = hook_override {
                // Hook specified assignment
                Some(member_id)
            } else {
                task_pool
                    .get_best_bid(&task_id)
                    .map(|best_bid| best_bid.member_id.clone())
            };

            // Perform assignment if we have a winner
            if let Some(member_id) = &assigned_to {
                // Use task assign event
                self.process_task_assign(
                    TaskAssignRequested {
                        task_id: task_id.clone(),
                        member_id: member_id.clone(),
                    },
                    resources,
                )
                .await;
            }

            // Publish bidding completed event
            self.publish_event(
                BiddingCompletedEvent {
                    task_id,
                    assigned_to,
                    bid_count,
                },
                resources,
            )
            .await;
        }
    }

    /// Publish an event to the EventBus
    async fn publish_event<T: Clone + serde::Serialize + 'static + crate::event::Event>(
        &self,
        event: T,
        resources: &mut ResourceContext,
    ) {
        if let Some(mut bus) = resources.get_mut::<EventBus>().await {
            bus.publish(event);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::holacracy::hook::DefaultHolacracyHook;

    #[tokio::test]
    async fn test_system_creation() {
        let _system = HolacracySystem::new(DefaultHolacracyHook);
        assert!(true);
    }

    #[tokio::test]
    async fn test_process_events_empty() {
        let mut system = HolacracySystem::new(DefaultHolacracyHook);
        let mut resources = ResourceContext::new();

        // Should not panic with empty resources
        system.process_events(&mut resources).await;
    }
}
