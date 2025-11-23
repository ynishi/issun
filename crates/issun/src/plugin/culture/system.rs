//! System for processing culture events
//!
//! This system orchestrates alignment checks, stress/fervor updates, and culture management
//! by coordinating between service functions, hooks, and state management.

use crate::context::ResourceContext;
use crate::event::EventBus;

use super::config::CultureConfig;
use super::events::*;
use super::hook::CultureHook;
use super::service::CultureService;
use super::state::CultureState;

/// System for processing culture events
///
/// This system:
/// 1. Listens for command events (AlignmentCheckRequested, MemberAddRequested, etc.)
/// 2. Validates via hook
/// 3. Executes logic via CultureService
/// 4. Updates CultureState
/// 5. Publishes state events (StressAccumulatedEvent, MemberBreakdownEvent, etc.)
pub struct CultureSystem<H: CultureHook> {
    hook: H,
    service: CultureService,
}

impl<H: CultureHook> CultureSystem<H> {
    /// Create a new culture system with a custom hook
    pub fn new(hook: H) -> Self {
        Self {
            hook,
            service: CultureService,
        }
    }

    /// Process all pending culture events
    ///
    /// This should be called once per game tick/frame.
    pub async fn process_events(&mut self, resources: &mut ResourceContext) {
        // Collect events from EventBus
        let alignment_requests = self
            .collect_events::<AlignmentCheckRequested>(resources)
            .await;
        let add_requests = self.collect_events::<MemberAddRequested>(resources).await;
        let remove_requests = self
            .collect_events::<MemberRemoveRequested>(resources)
            .await;
        let tag_add_requests = self
            .collect_events::<CultureTagAddRequested>(resources)
            .await;
        let tag_remove_requests = self
            .collect_events::<CultureTagRemoveRequested>(resources)
            .await;

        // Process events
        for request in alignment_requests {
            self.process_alignment_check_request(request, resources)
                .await;
        }

        for request in add_requests {
            self.process_add_member_request(request, resources).await;
        }

        for request in remove_requests {
            self.process_remove_member_request(request, resources)
                .await;
        }

        for request in tag_add_requests {
            self.process_add_culture_tag_request(request, resources)
                .await;
        }

        for request in tag_remove_requests {
            self.process_remove_culture_tag_request(request, resources)
                .await;
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

    /// Process alignment check for all factions and members
    async fn process_alignment_check_request(
        &mut self,
        _request: AlignmentCheckRequested,
        resources: &mut ResourceContext,
    ) {
        // Get config
        let config = match resources.get::<CultureConfig>().await {
            Some(c) => c.clone(),
            None => return,
        };

        // Get all factions and check alignment for each member
        let faction_member_pairs: Vec<_> = {
            let state = match resources.get_mut::<CultureState>().await {
                Some(s) => s,
                None => return,
            };

            state
                .all_cultures()
                .flat_map(|(faction_id, culture)| {
                    culture
                        .all_members()
                        .map(|(member_id, _)| (faction_id.clone(), member_id.clone()))
                        .collect::<Vec<_>>()
                })
                .collect()
        };

        // Process each member
        for (faction_id, member_id) in faction_member_pairs {
            self.check_member_alignment(&faction_id, &member_id, &config, resources)
                .await;
        }
    }

    /// Check alignment for a single member and update stress/fervor
    async fn check_member_alignment(
        &mut self,
        faction_id: &String,
        member_id: &String,
        config: &CultureConfig,
        resources: &mut ResourceContext,
    ) {
        // Get culture tags and member info
        let (culture_tags, member_info) = {
            let mut state = match resources.get_mut::<CultureState>().await {
                Some(s) => s,
                None => return,
            };

            let culture = match state.get_culture_mut(faction_id) {
                Some(c) => c,
                None => return,
            };

            let member = match culture.get_member(member_id) {
                Some(m) => m.clone(),
                None => return,
            };

            (culture.culture_tags().clone(), member)
        };

        // Check alignment
        let alignment = CultureService::check_alignment(&member_info, &culture_tags);

        // Publish alignment checked event
        self.publish_event(
            AlignmentCheckedEvent {
                faction_id: faction_id.to_string(),
                member_id: member_id.to_string(),
                alignment: alignment.clone(),
            },
            resources,
        )
        .await;

        // Notify hook
        self.hook
            .on_alignment_checked(faction_id, member_id, &alignment, resources)
            .await;

        // Get culture strength
        let culture_strength = {
            let state = match resources.get_mut::<CultureState>().await {
                Some(s) => s,
                None => return,
            };

            let culture = match state.get_culture(faction_id) {
                Some(c) => c,
                None => return,
            };

            culture.culture_strength().unwrap_or(config.culture_strength)
        };

        // Calculate new stress and fervor
        let old_stress = member_info.stress;
        let old_fervor = member_info.fervor;

        let new_stress = CultureService::calculate_stress_change(
            old_stress,
            &alignment,
            config,
            culture_strength,
        );

        let new_fervor = CultureService::calculate_fervor_change(
            old_fervor,
            &alignment,
            config,
            culture_strength,
        );

        // Update member state
        {
            let mut state = match resources.get_mut::<CultureState>().await {
                Some(s) => s,
                None => return,
            };

            let culture = match state.get_culture_mut(faction_id) {
                Some(c) => c,
                None => return,
            };

            if let Some(member) = culture.get_member_mut(member_id) {
                member.stress = new_stress;
                member.fervor = new_fervor;
                member.tenure += 1; // Increment tenure
            }
        }

        // Publish stress/fervor events if changed
        if (new_stress - old_stress).abs() > 0.001 {
            let reason = match &alignment {
                super::types::Alignment::Misaligned { reason, .. } => reason.clone(),
                super::types::Alignment::Aligned { .. } => "Aligned with culture".to_string(),
                super::types::Alignment::Neutral => "Neutral alignment".to_string(),
            };

            self.publish_event(
                StressAccumulatedEvent {
                    faction_id: faction_id.to_string(),
                    member_id: member_id.to_string(),
                    old_stress,
                    new_stress,
                    reason,
                },
                resources,
            )
            .await;

            self.hook
                .on_stress_accumulated(faction_id, member_id, new_stress, resources)
                .await;
        }

        if (new_fervor - old_fervor).abs() > 0.001 {
            self.publish_event(
                FervorIncreasedEvent {
                    faction_id: faction_id.to_string(),
                    member_id: member_id.to_string(),
                    old_fervor,
                    new_fervor,
                },
                resources,
            )
            .await;

            self.hook
                .on_fervor_increased(faction_id, member_id, new_fervor, resources)
                .await;
        }

        // Check for breakdown or fanaticism
        let member_clone = {
            let state = match resources.get_mut::<CultureState>().await {
                Some(s) => s,
                None => return,
            };

            let culture = match state.get_culture(faction_id) {
                Some(c) => c,
                None => return,
            };

            match culture.get_member(member_id) {
                Some(m) => m.clone(),
                None => return,
            }
        };

        // Check for breakdown (high stress)
        if CultureService::is_stressed_out(&member_clone, config) {
            self.publish_event(
                MemberBreakdownEvent {
                    faction_id: faction_id.to_string(),
                    member_id: member_id.to_string(),
                    stress_level: member_clone.stress,
                },
                resources,
            )
            .await;

            let should_remove = self
                .hook
                .on_member_breakdown(faction_id, &member_clone, resources)
                .await;

            if should_remove {
                let mut state = match resources.get_mut::<CultureState>().await {
                    Some(s) => s,
                    None => return,
                };

                if let Some(culture) = state.get_culture_mut(faction_id) {
                    culture.remove_member(member_id);

                    self.publish_event(
                        MemberRemovedEvent {
                            faction_id: faction_id.to_string(),
                            member_id: member_id.to_string(),
                        },
                        resources,
                    )
                    .await;
                }
            }
        }

        // Check for fanaticism (high fervor)
        if CultureService::is_fanatical(&member_clone, config) {
            self.publish_event(
                MemberFanaticizedEvent {
                    faction_id: faction_id.to_string(),
                    member_id: member_id.to_string(),
                    fervor_level: member_clone.fervor,
                },
                resources,
            )
            .await;

            self.hook
                .on_member_fanaticized(faction_id, &member_clone, resources)
                .await;
        }
    }

    /// Process add member request
    async fn process_add_member_request(
        &mut self,
        request: MemberAddRequested,
        resources: &mut ResourceContext,
    ) {
        let faction_id = request.faction_id.clone();
        let member = request.member;
        let member_id = member.id.clone();

        // Add member to state
        {
            let mut state = match resources.get_mut::<CultureState>().await {
                Some(s) => s,
                None => return,
            };

            if let Some(culture) = state.get_culture_mut(&faction_id) {
                culture.add_member(member);

                self.publish_event(
                    MemberAddedEvent {
                        faction_id,
                        member_id,
                    },
                    resources,
                )
                .await;
            }
        }
    }

    /// Process remove member request
    async fn process_remove_member_request(
        &mut self,
        request: MemberRemoveRequested,
        resources: &mut ResourceContext,
    ) {
        let faction_id = request.faction_id.clone();
        let member_id = request.member_id.clone();

        // Remove member from state
        {
            let mut state = match resources.get_mut::<CultureState>().await {
                Some(s) => s,
                None => return,
            };

            if let Some(culture) = state.get_culture_mut(&faction_id) {
                if culture.remove_member(&member_id).is_some() {
                    self.publish_event(
                        MemberRemovedEvent {
                            faction_id,
                            member_id,
                        },
                        resources,
                    )
                    .await;
                }
            }
        }
    }

    /// Process add culture tag request
    async fn process_add_culture_tag_request(
        &mut self,
        request: CultureTagAddRequested,
        resources: &mut ResourceContext,
    ) {
        let faction_id = request.faction_id.clone();
        let tag = request.tag.clone();

        // Check if hook allows adding this tag
        let can_add = self
            .hook
            .can_add_culture_tag(&faction_id, &tag, resources)
            .await;

        if !can_add {
            return;
        }

        // Add tag to state
        {
            let mut state = match resources.get_mut::<CultureState>().await {
                Some(s) => s,
                None => return,
            };

            if let Some(culture) = state.get_culture_mut(&faction_id) {
                culture.add_culture_tag(tag.clone());

                self.publish_event(
                    CultureTagAddedEvent {
                        faction_id: faction_id.clone(),
                        tag: tag.clone(),
                    },
                    resources,
                )
                .await;

                self.hook
                    .on_culture_tag_added(&faction_id, &tag, resources)
                    .await;
            }
        }
    }

    /// Process remove culture tag request
    async fn process_remove_culture_tag_request(
        &mut self,
        request: CultureTagRemoveRequested,
        resources: &mut ResourceContext,
    ) {
        let faction_id = request.faction_id.clone();
        let tag = request.tag.clone();

        // Remove tag from state
        {
            let mut state = match resources.get_mut::<CultureState>().await {
                Some(s) => s,
                None => return,
            };

            if let Some(culture) = state.get_culture_mut(&faction_id) {
                if culture.remove_culture_tag(&tag) {
                    self.publish_event(
                        CultureTagRemovedEvent {
                            faction_id: faction_id.clone(),
                            tag: tag.clone(),
                        },
                        resources,
                    )
                    .await;

                    self.hook
                        .on_culture_tag_removed(&faction_id, &tag, resources)
                        .await;
                }
            }
        }
    }

    /// Publish an event to the event bus
    async fn publish_event<T: crate::event::Event + Clone + serde::Serialize + 'static>(
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
    use crate::plugin::culture::hook::DefaultCultureHook;
    use crate::plugin::culture::types::{CultureTag, Member, PersonalityTrait};

    fn setup_test_resources() -> ResourceContext {
        let mut resources = ResourceContext::new();

        // Setup config
        let config = CultureConfig::default();
        resources.insert(config);

        // Setup state
        let mut state = CultureState::new();
        state.register_faction("faction_a");
        resources.insert(state);

        // Setup event bus
        let bus = EventBus::new();
        resources.insert(bus);

        resources
    }

    #[tokio::test]
    async fn test_add_member() {
        let mut system = CultureSystem::new(DefaultCultureHook);
        let mut resources = setup_test_resources();

        // Publish add member request
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(MemberAddRequested {
                faction_id: "faction_a".to_string(),
                member: Member::new("m1", "Alice"),
            });
            bus.dispatch();
        }

        // Process events
        system.process_events(&mut resources).await;

        // Verify member was added
        let state = resources.get_mut::<CultureState>().await.unwrap();
        let culture = state.get_culture(&"faction_a".to_string()).unwrap();
        assert_eq!(culture.member_count(), 1);
        assert!(culture.has_member(&"m1".to_string()));
    }

    #[tokio::test]
    async fn test_remove_member() {
        let mut system = CultureSystem::new(DefaultCultureHook);
        let mut resources = setup_test_resources();

        // Add member first
        {
            let mut state = resources.get_mut::<CultureState>().await.unwrap();
            if let Some(culture) = state.get_culture_mut(&"faction_a".to_string()) {
                culture.add_member(Member::new("m1", "Alice"));
            }
        }

        // Publish remove member request
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(MemberRemoveRequested {
                faction_id: "faction_a".to_string(),
                member_id: "m1".to_string(),
            });
            bus.dispatch();
        }

        // Process events
        system.process_events(&mut resources).await;

        // Verify member was removed
        let state = resources.get_mut::<CultureState>().await.unwrap();
        let culture = state.get_culture(&"faction_a".to_string()).unwrap();
        assert_eq!(culture.member_count(), 0);
    }

    #[tokio::test]
    async fn test_add_culture_tag() {
        let mut system = CultureSystem::new(DefaultCultureHook);
        let mut resources = setup_test_resources();

        // Publish add culture tag request
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(CultureTagAddRequested {
                faction_id: "faction_a".to_string(),
                tag: CultureTag::RiskTaking,
            });
            bus.dispatch();
        }

        // Process events
        system.process_events(&mut resources).await;

        // Verify tag was added
        let state = resources.get_mut::<CultureState>().await.unwrap();
        let culture = state.get_culture(&"faction_a".to_string()).unwrap();
        assert_eq!(culture.culture_tag_count(), 1);
        assert!(culture.has_culture_tag(&CultureTag::RiskTaking));
    }

    #[tokio::test]
    async fn test_remove_culture_tag() {
        let mut system = CultureSystem::new(DefaultCultureHook);
        let mut resources = setup_test_resources();

        // Add tag first
        {
            let mut state = resources.get_mut::<CultureState>().await.unwrap();
            if let Some(culture) = state.get_culture_mut(&"faction_a".to_string()) {
                culture.add_culture_tag(CultureTag::RiskTaking);
            }
        }

        // Publish remove culture tag request
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(CultureTagRemoveRequested {
                faction_id: "faction_a".to_string(),
                tag: CultureTag::RiskTaking,
            });
            bus.dispatch();
        }

        // Process events
        system.process_events(&mut resources).await;

        // Verify tag was removed
        let state = resources.get_mut::<CultureState>().await.unwrap();
        let culture = state.get_culture(&"faction_a".to_string()).unwrap();
        assert_eq!(culture.culture_tag_count(), 0);
    }

    #[tokio::test]
    async fn test_alignment_check_stress_accumulation() {
        let mut system = CultureSystem::new(DefaultCultureHook);
        let mut resources = setup_test_resources();

        // Setup member with misaligned personality
        {
            let mut state = resources.get_mut::<CultureState>().await.unwrap();
            if let Some(culture) = state.get_culture_mut(&"faction_a".to_string()) {
                // Cautious personality + RiskTaking culture = Misaligned
                culture.add_member(
                    Member::new("m1", "Alice")
                        .with_trait(PersonalityTrait::Cautious)
                        .with_stress(0.5),
                );
                culture.add_culture_tag(CultureTag::RiskTaking);
            }
        }

        // Publish alignment check request
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(AlignmentCheckRequested { delta_turns: 1 });
            bus.dispatch();
        }

        // Process events
        system.process_events(&mut resources).await;

        // Verify stress increased
        let state = resources.get_mut::<CultureState>().await.unwrap();
        let culture = state.get_culture(&"faction_a".to_string()).unwrap();
        let member = culture.get_member(&"m1".to_string()).unwrap();
        assert!(member.stress > 0.5); // Stress should have increased
    }

    #[tokio::test]
    async fn test_alignment_check_fervor_increase() {
        let mut system = CultureSystem::new(DefaultCultureHook);
        let mut resources = setup_test_resources();

        // Setup member with aligned personality
        {
            let mut state = resources.get_mut::<CultureState>().await.unwrap();
            if let Some(culture) = state.get_culture_mut(&"faction_a".to_string()) {
                // Bold personality + RiskTaking culture = Aligned
                culture.add_member(
                    Member::new("m1", "Alice")
                        .with_trait(PersonalityTrait::Bold)
                        .with_fervor(0.5),
                );
                culture.add_culture_tag(CultureTag::RiskTaking);
            }
        }

        // Publish alignment check request
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(AlignmentCheckRequested { delta_turns: 1 });
            bus.dispatch();
        }

        // Process events
        system.process_events(&mut resources).await;

        // Verify fervor increased
        let state = resources.get_mut::<CultureState>().await.unwrap();
        let culture = state.get_culture(&"faction_a".to_string()).unwrap();
        let member = culture.get_member(&"m1".to_string()).unwrap();
        assert!(member.fervor > 0.5); // Fervor should have increased
    }
}
