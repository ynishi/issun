//! System for processing hierarchy events
//!
//! This system orchestrates promotions, order execution, and loyalty/morale updates
//! by coordinating between service functions, hooks, and state management.

use crate::context::ResourceContext;
use crate::event::EventBus;

use super::config::ChainOfCommandConfig;
use super::events::*;
use super::hook::ChainOfCommandHook;
use super::rank_definitions::RankDefinitions;
use super::service::HierarchyService;
use super::state::HierarchyState;
use super::types::{OrderOutcome, PromotionError};

/// System for processing chain of command events
///
/// This system:
/// 1. Listens for command events (MemberPromoteRequested, OrderIssueRequested, etc.)
/// 2. Validates via hook
/// 3. Executes logic via HierarchyService
/// 4. Updates HierarchyState
/// 5. Publishes state events (MemberPromotedEvent, OrderExecutedEvent, etc.)
#[allow(dead_code)]
pub struct HierarchySystem<H: ChainOfCommandHook> {
    hook: H,
    service: HierarchyService,
}

impl<H: ChainOfCommandHook> HierarchySystem<H> {
    /// Create a new hierarchy system with a custom hook
    pub fn new(hook: H) -> Self {
        Self {
            hook,
            service: HierarchyService,
        }
    }

    /// Process all pending hierarchy events
    ///
    /// This should be called once per game tick/frame.
    pub async fn process_events(&mut self, resources: &mut ResourceContext) {
        // Collect events from EventBus
        let promote_requests = self
            .collect_events::<MemberPromoteRequested>(resources)
            .await;
        let order_requests = self.collect_events::<OrderIssueRequested>(resources).await;
        let loyalty_requests = self
            .collect_events::<LoyaltyDecayRequested>(resources)
            .await;
        let add_requests = self.collect_events::<MemberAddRequested>(resources).await;
        let remove_requests = self
            .collect_events::<MemberRemoveRequested>(resources)
            .await;

        // Process events
        for request in promote_requests {
            self.process_promote_request(request, resources).await;
        }

        for request in order_requests {
            self.process_order_request(request, resources).await;
        }

        for request in loyalty_requests {
            self.process_loyalty_decay_request(request, resources).await;
        }

        for request in add_requests {
            self.process_add_member_request(request, resources).await;
        }

        for request in remove_requests {
            self.process_remove_member_request(request, resources).await;
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

    /// Process a promotion request
    async fn process_promote_request(
        &mut self,
        request: MemberPromoteRequested,
        resources: &mut ResourceContext,
    ) {
        let faction_id = request.faction_id.clone();
        let member_id = request.member_id.clone();
        let new_rank = request.new_rank.clone();

        // Get config and rank definitions
        let config = match resources.get::<ChainOfCommandConfig>().await {
            Some(c) => c.clone(),
            None => return,
        };

        let rank_defs = match resources.get::<RankDefinitions>().await {
            Some(r) => r.clone(),
            None => return,
        };

        // Get current state
        let (old_rank, result) = {
            let mut state = match resources.get_mut::<HierarchyState>().await {
                Some(s) => s,
                None => return,
            };

            let hierarchy = match state.get_hierarchy_mut(&faction_id) {
                Some(h) => h,
                None => {
                    self.publish_event(
                        PromotionFailedEvent {
                            faction_id,
                            member_id,
                            reason: PromotionError::FactionNotFound,
                        },
                        resources,
                    )
                    .await;
                    return;
                }
            };

            let member = match hierarchy.get_member_mut(&member_id) {
                Some(m) => m,
                None => {
                    self.publish_event(
                        PromotionFailedEvent {
                            faction_id,
                            member_id,
                            reason: PromotionError::MemberNotFound,
                        },
                        resources,
                    )
                    .await;
                    return;
                }
            };

            let old_rank = member.rank.clone();

            let current_rank_def = match rank_defs.get(&member.rank) {
                Some(r) => r,
                None => {
                    self.publish_event(
                        PromotionFailedEvent {
                            faction_id,
                            member_id,
                            reason: PromotionError::RankNotFound,
                        },
                        resources,
                    )
                    .await;
                    return;
                }
            };

            let new_rank_def = match rank_defs.get(&new_rank) {
                Some(r) => r,
                None => {
                    self.publish_event(
                        PromotionFailedEvent {
                            faction_id,
                            member_id,
                            reason: PromotionError::RankNotFound,
                        },
                        resources,
                    )
                    .await;
                    return;
                }
            };

            // Service: Check eligibility
            if !HierarchyService::can_promote(member, current_rank_def, new_rank_def, &config) {
                self.publish_event(
                    PromotionFailedEvent {
                        faction_id,
                        member_id,
                        reason: PromotionError::NotEligible,
                    },
                    resources,
                )
                .await;
                return;
            }

            // Hook: Game-specific conditions
            if !self
                .hook
                .can_promote_custom(member, new_rank_def, resources)
                .await
            {
                self.publish_event(
                    PromotionFailedEvent {
                        faction_id,
                        member_id,
                        reason: PromotionError::CustomConditionFailed,
                    },
                    resources,
                )
                .await;
                return;
            }

            // Execute promotion
            member.rank = new_rank.clone();
            member.turns_since_promotion = 0;
            member.morale = (member.morale + 0.2).min(1.0);
            member.loyalty = (member.loyalty + 0.1).min(1.0);

            (old_rank, Ok::<(), ()>(()))
        };

        if result.is_ok() {
            // Hook: Notify
            self.hook
                .on_member_promoted(&faction_id, &member_id, &new_rank, resources)
                .await;

            // Publish success event
            self.publish_event(
                MemberPromotedEvent {
                    faction_id,
                    member_id,
                    old_rank,
                    new_rank,
                },
                resources,
            )
            .await;
        }
    }

    /// Process an order issuance request
    async fn process_order_request(
        &mut self,
        request: OrderIssueRequested,
        resources: &mut ResourceContext,
    ) {
        let faction_id = request.faction_id.clone();
        let superior_id = request.superior_id.clone();
        let subordinate_id = request.subordinate_id.clone();
        let order = request.order.clone();

        // Get config
        let config = match resources.get::<ChainOfCommandConfig>().await {
            Some(c) => c.clone(),
            None => return,
        };

        // Get state (read-only for order execution)
        let outcome = {
            let state = match resources.get::<HierarchyState>().await {
                Some(s) => s,
                None => return,
            };

            let hierarchy = match state.get_hierarchy(&faction_id) {
                Some(h) => h,
                None => return,
            };

            let superior = match hierarchy.get_member(&superior_id) {
                Some(m) => m.clone(),
                None => return,
            };

            let subordinate = match hierarchy.get_member(&subordinate_id) {
                Some(m) => m.clone(),
                None => return,
            };

            // Verify reporting relationship
            if !hierarchy.is_direct_subordinate(&subordinate_id, &superior_id) {
                return; // Silently ignore invalid orders
            }

            // Calculate compliance
            let compliance_rate = HierarchyService::calculate_order_compliance(
                &subordinate,
                &superior,
                config.base_order_compliance_rate,
            );

            // Random roll
            let executed = rand::random::<f32>() < compliance_rate;

            if executed {
                OrderOutcome::Executed
            } else {
                OrderOutcome::Refused {
                    reason: format!(
                        "Low loyalty ({:.0}%) or morale ({:.0}%)",
                        subordinate.loyalty * 100.0,
                        subordinate.morale * 100.0
                    ),
                }
            }
        };

        // Process outcome
        match outcome {
            OrderOutcome::Executed => {
                // Hook: Execute order
                self.hook
                    .execute_order(&faction_id, &subordinate_id, &order, resources)
                    .await;

                // Publish executed event
                self.publish_event(
                    OrderExecutedEvent {
                        faction_id,
                        superior_id,
                        subordinate_id,
                        order,
                    },
                    resources,
                )
                .await;
            }
            OrderOutcome::Refused { reason } => {
                // Hook: Order refused
                self.hook
                    .on_order_refused(&faction_id, &subordinate_id, &order, resources)
                    .await;

                // Publish refused event
                self.publish_event(
                    OrderRefusedEvent {
                        faction_id,
                        superior_id,
                        subordinate_id,
                        order,
                        reason,
                    },
                    resources,
                )
                .await;
            }
        }
    }

    /// Process loyalty decay request
    async fn process_loyalty_decay_request(
        &mut self,
        request: LoyaltyDecayRequested,
        resources: &mut ResourceContext,
    ) {
        let delta_turns = request.delta_turns;

        // Get config
        let config = match resources.get::<ChainOfCommandConfig>().await {
            Some(c) => c.clone(),
            None => return,
        };

        // Update all members
        let members_affected = {
            let mut state = match resources.get_mut::<HierarchyState>().await {
                Some(s) => s,
                None => return,
            };

            let mut count = 0;

            for (_faction_id, hierarchy) in state.all_hierarchies_mut() {
                // Collect member IDs first (to avoid borrow issues)
                let member_ids: Vec<_> =
                    hierarchy.all_members().map(|(id, _)| id.clone()).collect();

                for member_id in member_ids {
                    // Get superior modifier first (before mutable borrow)
                    let superior_modifier = hierarchy
                        .get_member(&member_id)
                        .and_then(|m| m.superior.clone())
                        .and_then(|sup_id| hierarchy.get_member(&sup_id))
                        .map(|sup| {
                            let member = hierarchy.get_member(&member_id).unwrap();
                            HierarchyService::calculate_loyalty_modifier(member, sup)
                        })
                        .unwrap_or(0.0);

                    // Now apply changes with mutable borrow
                    if let Some(member) = hierarchy.get_member_mut(&member_id) {
                        // Natural loyalty decay
                        member.loyalty = HierarchyService::decay_loyalty(
                            member.loyalty,
                            config.loyalty_decay_rate,
                            delta_turns,
                        );

                        // Superior relationship bonus
                        member.loyalty = (member.loyalty + superior_modifier).min(1.0);

                        // Update tenure
                        member.tenure += delta_turns;
                        member.turns_since_promotion += delta_turns;

                        count += 1;
                    }
                }
            }

            count
        };

        // Publish processed event
        self.publish_event(
            LoyaltyDecayProcessedEvent {
                delta_turns,
                members_affected,
            },
            resources,
        )
        .await;
    }

    /// Process add member request
    async fn process_add_member_request(
        &mut self,
        request: MemberAddRequested,
        resources: &mut ResourceContext,
    ) {
        let faction_id = request.faction_id.clone();
        let member_id = request.member.id.clone();

        {
            let mut state = match resources.get_mut::<HierarchyState>().await {
                Some(s) => s,
                None => return,
            };

            if let Some(hierarchy) = state.get_hierarchy_mut(&faction_id) {
                hierarchy.add_member(request.member);
            }
        }

        // Publish added event
        self.publish_event(
            MemberAddedEvent {
                faction_id,
                member_id,
            },
            resources,
        )
        .await;
    }

    /// Process remove member request
    async fn process_remove_member_request(
        &mut self,
        request: MemberRemoveRequested,
        resources: &mut ResourceContext,
    ) {
        let faction_id = request.faction_id.clone();
        let member_id = request.member_id.clone();

        {
            let mut state = match resources.get_mut::<HierarchyState>().await {
                Some(s) => s,
                None => return,
            };

            if let Some(hierarchy) = state.get_hierarchy_mut(&faction_id) {
                hierarchy.remove_member(&member_id);
            }
        }

        // Publish removed event
        self.publish_event(
            MemberRemovedEvent {
                faction_id,
                member_id,
            },
            resources,
        )
        .await;
    }

    /// Publish an event to the EventBus
    async fn publish_event<T: Clone + 'static + crate::event::Event + serde::Serialize>(
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
    use crate::plugin::chain_of_command::{DefaultChainOfCommandHook, Member, RankDefinition};
    use crate::plugin::AuthorityLevel;

    fn create_test_resources() -> ResourceContext {
        let mut ctx = ResourceContext::new();

        // Add EventBus
        let bus = EventBus::new();
        ctx.insert(bus);

        // Add config
        ctx.insert(ChainOfCommandConfig::default());

        // Add rank definitions
        let mut ranks = RankDefinitions::new();
        ranks.add(RankDefinition::new(
            "private",
            "Private",
            0,
            AuthorityLevel::Private,
        ));
        ranks.add(RankDefinition::new(
            "sergeant",
            "Sergeant",
            1,
            AuthorityLevel::SquadLeader,
        ));
        ctx.insert(ranks);

        // Add hierarchy state
        let mut state = HierarchyState::new();
        state.register_faction("test_faction");
        ctx.insert(state);

        ctx
    }

    #[tokio::test]
    async fn test_system_creation() {
        let hook = DefaultChainOfCommandHook;
        let _system = HierarchySystem::new(hook);
    }

    #[tokio::test]
    async fn test_process_add_member_request() {
        let hook = DefaultChainOfCommandHook;
        let mut system = HierarchySystem::new(hook);
        let mut resources = create_test_resources();

        // Publish add request
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(MemberAddRequested {
                faction_id: "test_faction".to_string(),
                member: Member::new("m1", "Member 1", "private"),
            });
            bus.dispatch();
        }

        // Process events
        system.process_events(&mut resources).await;

        // Verify member was added
        let state = resources.get::<HierarchyState>().await.unwrap();
        let hierarchy = state.get_hierarchy(&"test_faction".to_string()).unwrap();
        assert!(hierarchy.has_member(&"m1".to_string()));
    }

    #[tokio::test]
    async fn test_process_remove_member_request() {
        let hook = DefaultChainOfCommandHook;
        let mut system = HierarchySystem::new(hook);
        let mut resources = create_test_resources();

        // Add a member first
        {
            let mut state = resources.get_mut::<HierarchyState>().await.unwrap();
            let hierarchy = state
                .get_hierarchy_mut(&"test_faction".to_string())
                .unwrap();
            hierarchy.add_member(Member::new("m1", "Member 1", "private"));
        }

        // Publish remove request
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(MemberRemoveRequested {
                faction_id: "test_faction".to_string(),
                member_id: "m1".to_string(),
            });
            bus.dispatch();
        }

        // Process events
        system.process_events(&mut resources).await;

        // Verify member was removed
        let state = resources.get::<HierarchyState>().await.unwrap();
        let hierarchy = state.get_hierarchy(&"test_faction".to_string()).unwrap();
        assert!(!hierarchy.has_member(&"m1".to_string()));
    }

    #[tokio::test]
    async fn test_process_promote_request_success() {
        let hook = DefaultChainOfCommandHook;
        let mut system = HierarchySystem::new(hook);
        let mut resources = create_test_resources();

        // Add a member with sufficient tenure and loyalty
        {
            let mut state = resources.get_mut::<HierarchyState>().await.unwrap();
            let hierarchy = state
                .get_hierarchy_mut(&"test_faction".to_string())
                .unwrap();
            hierarchy.add_member(
                Member::new("m1", "Member 1", "private")
                    .with_tenure(10)
                    .with_loyalty(0.8),
            );
        }

        // Publish promote request
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(MemberPromoteRequested {
                faction_id: "test_faction".to_string(),
                member_id: "m1".to_string(),
                new_rank: "sergeant".to_string(),
            });
            bus.dispatch();
        }

        // Process events
        system.process_events(&mut resources).await;

        // Verify promotion
        let state = resources.get::<HierarchyState>().await.unwrap();
        let hierarchy = state.get_hierarchy(&"test_faction".to_string()).unwrap();
        let member = hierarchy.get_member(&"m1".to_string()).unwrap();
        assert_eq!(member.rank, "sergeant");
        assert_eq!(member.turns_since_promotion, 0);
    }

    #[tokio::test]
    async fn test_process_promote_request_insufficient_tenure() {
        let hook = DefaultChainOfCommandHook;
        let mut system = HierarchySystem::new(hook);
        let mut resources = create_test_resources();

        // Add a member with insufficient tenure
        {
            let mut state = resources.get_mut::<HierarchyState>().await.unwrap();
            let hierarchy = state
                .get_hierarchy_mut(&"test_faction".to_string())
                .unwrap();
            hierarchy.add_member(
                Member::new("m1", "Member 1", "private")
                    .with_tenure(2) // Need 5
                    .with_loyalty(0.8),
            );
        }

        // Publish promote request
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(MemberPromoteRequested {
                faction_id: "test_faction".to_string(),
                member_id: "m1".to_string(),
                new_rank: "sergeant".to_string(),
            });
            bus.dispatch();
        }

        // Process events
        system.process_events(&mut resources).await;

        // Verify no promotion
        let state = resources.get::<HierarchyState>().await.unwrap();
        let hierarchy = state.get_hierarchy(&"test_faction".to_string()).unwrap();
        let member = hierarchy.get_member(&"m1".to_string()).unwrap();
        assert_eq!(member.rank, "private"); // Still private
    }

    #[tokio::test]
    async fn test_process_loyalty_decay() {
        let hook = DefaultChainOfCommandHook;
        let mut system = HierarchySystem::new(hook);
        let mut resources = create_test_resources();

        // Add a member
        {
            let mut state = resources.get_mut::<HierarchyState>().await.unwrap();
            let hierarchy = state
                .get_hierarchy_mut(&"test_faction".to_string())
                .unwrap();
            hierarchy.add_member(
                Member::new("m1", "Member 1", "private")
                    .with_loyalty(1.0)
                    .with_tenure(0),
            );
        }

        // Publish loyalty decay request
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(LoyaltyDecayRequested { delta_turns: 5 });
            bus.dispatch();
        }

        // Process events
        system.process_events(&mut resources).await;

        // Verify loyalty decayed
        let state = resources.get::<HierarchyState>().await.unwrap();
        let hierarchy = state.get_hierarchy(&"test_faction".to_string()).unwrap();
        let member = hierarchy.get_member(&"m1".to_string()).unwrap();

        // 1.0 - 0.02 * 5 = 0.9
        assert!((member.loyalty - 0.9).abs() < 0.001);
        assert_eq!(member.tenure, 5);
    }
}
