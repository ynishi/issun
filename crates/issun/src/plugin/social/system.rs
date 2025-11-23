//! System for processing social network events
//!
//! This system orchestrates centrality calculation, political actions, and faction dynamics
//! by coordinating between service functions, hooks, and state management.

use crate::context::ResourceContext;
use crate::event::EventBus;

use super::config::SocialConfig;
use super::events::*;
use super::hook::SocialHook;
use super::service::NetworkAnalysisService;
use super::state::{SocialMember, SocialState};
use super::types::{FactionId, MemberId, PoliticalAction};

/// System for processing social network events
///
/// This system:
/// 1. Listens for command events (CentralityRecalculationRequested, PoliticalActionRequested, etc.)
/// 2. Validates via hook
/// 3. Executes logic via NetworkAnalysisService
/// 4. Updates SocialState
/// 5. Publishes state events (CentralityCalculatedEvent, ShadowLeaderDetectedEvent, etc.)
pub struct SocialSystem<H: SocialHook> {
    hook: H,
    _service: NetworkAnalysisService,
}

impl<H: SocialHook> SocialSystem<H> {
    /// Create a new social system with a custom hook
    pub fn new(hook: H) -> Self {
        Self {
            hook,
            _service: NetworkAnalysisService,
        }
    }

    /// Process all pending social network events
    ///
    /// This should be called once per game tick/frame.
    pub async fn process_events(&mut self, resources: &mut ResourceContext) {
        // Collect events from EventBus
        let centrality_requests = self
            .collect_events::<CentralityRecalculationRequested>(resources)
            .await;
        let political_action_requests = self
            .collect_events::<PoliticalActionRequested>(resources)
            .await;
        let relation_add_requests = self.collect_events::<RelationAddRequested>(resources).await;
        let member_add_requests = self.collect_events::<MemberAddRequested>(resources).await;
        let member_remove_requests = self
            .collect_events::<MemberRemoveRequested>(resources)
            .await;

        // Process events
        for request in centrality_requests {
            self.process_centrality_recalculation(request, resources)
                .await;
        }

        for request in political_action_requests {
            self.process_political_action(request, resources).await;
        }

        for request in relation_add_requests {
            self.process_relation_add(request, resources).await;
        }

        for request in member_add_requests {
            self.process_member_add(request, resources).await;
        }

        for request in member_remove_requests {
            self.process_member_remove(request, resources).await;
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

    /// Process centrality recalculation request
    async fn process_centrality_recalculation(
        &mut self,
        request: CentralityRecalculationRequested,
        resources: &mut ResourceContext,
    ) {
        let config = match resources.get::<SocialConfig>().await {
            Some(c) => c.clone(),
            None => return,
        };

        let mut state = match resources.get_mut::<SocialState>().await {
            Some(s) => s,
            None => return,
        };

        let network = match state.get_network_mut(&request.faction_id) {
            Some(n) => n,
            None => return,
        };

        // Calculate eigenvector centrality for all members (expensive)
        let eigenvector_scores =
            NetworkAnalysisService::calculate_eigenvector_centrality(network, 100, 0.0001);

        // Update centrality for each member
        let member_ids: Vec<MemberId> = network.all_members().map(|(id, _)| id.clone()).collect();

        for member_id in member_ids {
            if let Ok(metrics) = NetworkAnalysisService::calculate_all_centrality(
                &member_id,
                network,
                &config,
                Some(&eigenvector_scores),
            ) {
                // Update member's centrality
                if let Some(member) = network.get_member_mut(&member_id) {
                    member.capital.centrality_scores = metrics.clone();
                }

                // Publish event
                self.publish_event(
                    CentralityCalculatedEvent {
                        faction_id: request.faction_id.clone(),
                        member_id: member_id.clone(),
                        metrics: metrics.clone(),
                    },
                    resources,
                )
                .await;

                // Notify hook
                self.hook
                    .on_centrality_calculated(&request.faction_id, &member_id, &metrics, resources)
                    .await;

                // Check for shadow leaders
                if metrics.is_shadow_leader(config.shadow_leader_threshold)
                    && metrics.betweenness > 0.5
                {
                    self.publish_event(
                        ShadowLeaderDetectedEvent {
                            faction_id: request.faction_id.clone(),
                            member_id: member_id.clone(),
                            influence_score: metrics.overall_influence,
                            betweenness: metrics.betweenness,
                        },
                        resources,
                    )
                    .await;

                    self.hook
                        .on_shadow_leader_detected(
                            &request.faction_id,
                            &member_id,
                            metrics.overall_influence,
                            resources,
                        )
                        .await;
                }
            }
        }

        // Mark cache as valid
        if let Some(network) = state.get_network_mut(&request.faction_id) {
            network.mark_centrality_cache_valid(0); // TODO: actual turn number
        }
    }

    /// Process political action request
    async fn process_political_action(
        &mut self,
        request: PoliticalActionRequested,
        resources: &mut ResourceContext,
    ) {
        // Validate via hook
        let validation_result = self
            .hook
            .on_political_action_requested(
                &request.faction_id,
                &request.actor_id,
                &request.action,
                resources,
            )
            .await;

        if let Err(reason) = validation_result {
            self.publish_event(
                PoliticalActionExecutedEvent {
                    faction_id: request.faction_id.clone(),
                    actor_id: request.actor_id.clone(),
                    action: request.action.clone(),
                    success: false,
                    reason: Some(reason),
                },
                resources,
            )
            .await;
            return;
        }

        // Execute action based on type
        let success = match &request.action {
            PoliticalAction::GrantFavor {
                target,
                favor_value,
            } => {
                self.execute_grant_favor(
                    &request.faction_id,
                    &request.actor_id,
                    target,
                    *favor_value,
                    resources,
                )
                .await
            }
            PoliticalAction::ShareSecret {
                target,
                secret_id,
                sensitivity,
            } => {
                self.execute_share_secret(
                    &request.faction_id,
                    &request.actor_id,
                    target,
                    secret_id,
                    *sensitivity,
                    resources,
                )
                .await
            }
            PoliticalAction::SpreadGossip {
                about,
                content,
                is_positive,
            } => {
                self.execute_spread_gossip(
                    &request.faction_id,
                    &request.actor_id,
                    about,
                    content,
                    *is_positive,
                    resources,
                )
                .await
            }
            _ => true, // Other actions not yet implemented
        };

        // Publish result event
        self.publish_event(
            PoliticalActionExecutedEvent {
                faction_id: request.faction_id.clone(),
                actor_id: request.actor_id.clone(),
                action: request.action.clone(),
                success,
                reason: None,
            },
            resources,
        )
        .await;

        // Notify hook
        self.hook
            .on_political_action_executed(
                &request.faction_id,
                &request.actor_id,
                &request.action,
                success,
                resources,
            )
            .await;
    }

    /// Execute GrantFavor action
    async fn execute_grant_favor(
        &mut self,
        faction_id: &FactionId,
        grantor: &MemberId,
        recipient: &MemberId,
        favor_value: f32,
        resources: &mut ResourceContext,
    ) -> bool {
        let mut state = match resources.get_mut::<SocialState>().await {
            Some(s) => s,
            None => return false,
        };

        let network = match state.get_network_mut(faction_id) {
            Some(n) => n,
            None => return false,
        };

        // Update social capital
        if let Some(grantor_member) = network.get_member_mut(grantor) {
            grantor_member.capital.total_favors_owed_to_me += favor_value;
        } else {
            return false;
        }

        if let Some(recipient_member) = network.get_member_mut(recipient) {
            recipient_member.capital.total_favors_i_owe += favor_value;
        } else {
            return false;
        }

        // Publish event
        self.publish_event(
            FavorExchangedEvent {
                faction_id: faction_id.clone(),
                grantor: grantor.clone(),
                recipient: recipient.clone(),
                favor_value,
            },
            resources,
        )
        .await;

        // Notify hook
        self.hook
            .on_favor_exchanged(faction_id, grantor, recipient, favor_value, resources)
            .await;

        true
    }

    /// Execute ShareSecret action
    async fn execute_share_secret(
        &mut self,
        faction_id: &FactionId,
        sharer: &MemberId,
        receiver: &MemberId,
        secret_id: &str,
        sensitivity: f32,
        resources: &mut ResourceContext,
    ) -> bool {
        let mut state = match resources.get_mut::<SocialState>().await {
            Some(s) => s,
            None => return false,
        };

        let network = match state.get_network_mut(faction_id) {
            Some(n) => n,
            None => return false,
        };

        // Update secrets count
        if let Some(sharer_member) = network.get_member_mut(sharer) {
            sharer_member.capital.secrets_held += 1;
        } else {
            return false;
        }

        if let Some(receiver_member) = network.get_member_mut(receiver) {
            receiver_member.capital.secrets_held += 1;
        } else {
            return false;
        }

        // Add SharedSecret relation
        let _ = network.add_relation(
            sharer.clone(),
            receiver.clone(),
            super::types::RelationType::SharedSecret {
                secret_id: secret_id.to_string(),
                sensitivity,
            },
        );

        // Publish event
        self.publish_event(
            SecretSharedEvent {
                faction_id: faction_id.clone(),
                sharer: sharer.clone(),
                receiver: receiver.clone(),
                secret_id: secret_id.to_string(),
                sensitivity,
            },
            resources,
        )
        .await;

        // Notify hook
        self.hook
            .on_secret_shared(
                faction_id,
                sharer,
                receiver,
                secret_id,
                sensitivity,
                resources,
            )
            .await;

        true
    }

    /// Execute SpreadGossip action
    async fn execute_spread_gossip(
        &mut self,
        faction_id: &FactionId,
        spreader: &MemberId,
        about: &MemberId,
        content: &str,
        is_positive: bool,
        resources: &mut ResourceContext,
    ) -> bool {
        let state = match resources.get_mut::<SocialState>().await {
            Some(s) => s,
            None => return false,
        };

        let network = match state.get_network(faction_id) {
            Some(n) => n,
            None => return false,
        };

        // Get neighbors (simplified gossip spread)
        let reached_members = network.get_neighbors(spreader);

        // Publish event
        self.publish_event(
            GossipSpreadEvent {
                faction_id: faction_id.clone(),
                spreader: spreader.clone(),
                about: about.clone(),
                content: content.to_string(),
                is_positive,
                reached_members: reached_members.clone(),
            },
            resources,
        )
        .await;

        // Notify hook
        self.hook
            .on_gossip_spread(
                faction_id,
                spreader,
                about,
                content,
                is_positive,
                reached_members.len(),
                resources,
            )
            .await;

        true
    }

    /// Process relation add request
    async fn process_relation_add(
        &mut self,
        request: RelationAddRequested,
        resources: &mut ResourceContext,
    ) {
        let mut state = match resources.get_mut::<SocialState>().await {
            Some(s) => s,
            None => return,
        };

        let network = match state.get_network_mut(&request.faction_id) {
            Some(n) => n,
            None => return,
        };

        let result = network.add_relation(
            request.from.clone(),
            request.to.clone(),
            request.relation.clone(),
        );

        if result.is_ok() {
            self.publish_event(
                RelationshipChangedEvent {
                    faction_id: request.faction_id.clone(),
                    from: request.from.clone(),
                    to: request.to.clone(),
                    old_relation: None,
                    new_relation: Some(request.relation),
                },
                resources,
            )
            .await;
        }
    }

    /// Process member add request
    async fn process_member_add(
        &mut self,
        request: MemberAddRequested,
        resources: &mut ResourceContext,
    ) {
        let mut state = match resources.get_mut::<SocialState>().await {
            Some(s) => s,
            None => return,
        };

        let network = match state.get_network_mut(&request.faction_id) {
            Some(n) => n,
            None => return,
        };

        let member = SocialMember::new(request.member_id.clone(), request.member_name);
        network.add_member(member);

        self.publish_event(
            MemberAddedEvent {
                faction_id: request.faction_id.clone(),
                member_id: request.member_id.clone(),
            },
            resources,
        )
        .await;
    }

    /// Process member remove request
    async fn process_member_remove(
        &mut self,
        request: MemberRemoveRequested,
        resources: &mut ResourceContext,
    ) {
        let mut state = match resources.get_mut::<SocialState>().await {
            Some(s) => s,
            None => return,
        };

        let network = match state.get_network_mut(&request.faction_id) {
            Some(n) => n,
            None => return,
        };

        if network.remove_member(&request.member_id).is_some() {
            self.publish_event(
                MemberRemovedEvent {
                    faction_id: request.faction_id.clone(),
                    member_id: request.member_id.clone(),
                    reason: "Requested".to_string(),
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
    use crate::plugin::social::hook::DefaultSocialHook;

    #[tokio::test]
    async fn test_system_creation() {
        let _system = SocialSystem::new(DefaultSocialHook);
        // System created successfully - compilation is the test
    }

    #[tokio::test]
    async fn test_process_events_empty() {
        let mut system = SocialSystem::new(DefaultSocialHook);
        let mut resources = ResourceContext::new();

        // Should not panic with empty resources
        system.process_events(&mut resources).await;
    }
}
