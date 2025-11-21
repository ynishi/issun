//! Reputation system for processing events

use crate::context::ResourceContext;
use crate::event::EventBus;

use super::config::ReputationConfig;
use super::events::*;
use super::hook::ReputationHook;
use super::state::ReputationState;

/// System for processing reputation events
///
/// This system:
/// 1. Listens for `ReputationChangeRequested` and `ReputationSetRequested` events
/// 2. Validates changes via hook
/// 3. Calculates effective delta via hook
/// 4. Updates `ReputationState`
/// 5. Calls hook callbacks (`on_reputation_changed`, `on_threshold_crossed`)
/// 6. Publishes state events (`ReputationChangedEvent`, `ReputationThresholdCrossedEvent`)
pub struct ReputationSystem<H: ReputationHook> {
    hook: H,
}

impl<H: ReputationHook> ReputationSystem<H> {
    /// Create a new reputation system with a custom hook
    pub fn new(hook: H) -> Self {
        Self { hook }
    }

    /// Process all pending reputation events
    ///
    /// This should be called once per game tick/frame.
    pub async fn process_events(&mut self, resources: &mut ResourceContext) {
        // Collect events from EventBus
        let change_requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<ReputationChangeRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        let set_requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<ReputationSetRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        // Process change requests
        for request in change_requests {
            self.process_change_request(request, resources).await;
        }

        // Process set requests
        for request in set_requests {
            self.process_set_request(request, resources).await;
        }
    }

    /// Process a single reputation change request
    async fn process_change_request(
        &mut self,
        request: ReputationChangeRequested,
        resources: &mut ResourceContext,
    ) {
        let subject_id = request.subject_id;
        let category = request.category.as_deref();

        // 1. Validate change via hook
        if let Err(_reason) = self
            .hook
            .validate_change(&subject_id, request.delta, category, resources)
            .await
        {
            // Validation failed, silently skip this request
            return;
        }

        // 2. Calculate effective delta via hook
        let effective_delta = self
            .hook
            .calculate_delta(&subject_id, request.delta, category, resources)
            .await;

        // 3. Get old score and threshold
        let (old_score, old_threshold_name) = {
            let config = match resources.get::<ReputationConfig>().await {
                Some(c) => c,
                None => return,
            };
            let state = match resources.get::<ReputationState>().await {
                Some(s) => s,
                None => return,
            };

            let old_score = match category {
                Some(cat) => state.get_category(&subject_id, cat).unwrap_or(config.default_score),
                None => state.get(&subject_id).unwrap_or(config.default_score),
            };

            let old_threshold = config.get_threshold(old_score).map(|t| t.name.clone());

            (old_score, old_threshold)
        };

        // 4. Update state
        let new_score = {
            let config = resources.get::<ReputationConfig>().await.unwrap();
            let mut state = resources.get_mut::<ReputationState>().await.unwrap();

            let (_, new_score) = match category {
                Some(cat) => state.adjust_category(
                    &subject_id,
                    cat.to_string(),
                    effective_delta,
                    config.default_score,
                ),
                None => state.adjust(&subject_id, effective_delta, config.default_score),
            };

            // Apply auto-clamping if enabled
            if config.auto_clamp {
                if let Some((min, max)) = config.score_range {
                    match category {
                        Some(cat) => {
                            state.clamp_score_category(&subject_id, cat, min, max);
                        }
                        None => {
                            state.clamp_score(&subject_id, min, max);
                        }
                    }
                }
            }

            // Get final score after clamping
            match category {
                Some(cat) => state.get_category(&subject_id, cat).unwrap_or(new_score),
                None => state.get(&subject_id).unwrap_or(new_score),
            }
        };

        // 5. Call hook: on_reputation_changed
        self.hook
            .on_reputation_changed(
                &subject_id,
                old_score,
                new_score,
                effective_delta,
                category,
                resources,
            )
            .await;

        // 6. Check for threshold crossing
        let new_threshold_name = {
            let config = resources.get::<ReputationConfig>().await.unwrap();
            config.get_threshold(new_score).map(|t| t.name.clone())
        };

        if old_threshold_name != new_threshold_name {
            if let Some(new_threshold_name) = &new_threshold_name {
                // Call hook: on_threshold_crossed
                let config = resources.get::<ReputationConfig>().await.unwrap();
                let old_threshold = old_threshold_name
                    .as_ref()
                    .and_then(|name| config.thresholds.iter().find(|t| &t.name == name));
                let new_threshold = config.thresholds.iter().find(|t| &t.name == new_threshold_name);

                if let Some(new_threshold) = new_threshold {
                    self.hook
                        .on_threshold_crossed(&subject_id, old_threshold, new_threshold, resources)
                        .await;
                }

                // Publish threshold crossed event
                let mut bus = resources.get_mut::<EventBus>().await.unwrap();
                bus.publish(ReputationThresholdCrossedEvent {
                    subject_id: subject_id.clone(),
                    old_threshold: old_threshold_name.clone(),
                    new_threshold: new_threshold_name.clone(),
                    score: new_score,
                    category: category.map(|s| s.to_string()),
                });
            }
        }

        // 7. Publish reputation changed event
        let mut bus = resources.get_mut::<EventBus>().await.unwrap();
        bus.publish(ReputationChangedEvent {
            subject_id,
            old_score,
            new_score,
            delta: effective_delta,
            category: request.category,
            reason: request.reason,
        });
    }

    /// Process a single reputation set request
    async fn process_set_request(
        &mut self,
        request: ReputationSetRequested,
        resources: &mut ResourceContext,
    ) {
        let subject_id = request.subject_id;
        let category = request.category.as_deref();

        // Calculate delta for validation
        let old_score = {
            let config = match resources.get::<ReputationConfig>().await {
                Some(c) => c,
                None => return,
            };
            let state = match resources.get::<ReputationState>().await {
                Some(s) => s,
                None => return,
            };

            match category {
                Some(cat) => state.get_category(&subject_id, cat).unwrap_or(config.default_score),
                None => state.get(&subject_id).unwrap_or(config.default_score),
            }
        };

        let delta = request.score - old_score;

        // 1. Validate change via hook
        if let Err(_reason) = self
            .hook
            .validate_change(&subject_id, delta, category, resources)
            .await
        {
            // Validation failed, silently skip this request
            return;
        }

        // 2. Get old threshold
        let old_threshold_name = {
            let config = resources.get::<ReputationConfig>().await.unwrap();
            config.get_threshold(old_score).map(|t| t.name.clone())
        };

        // 3. Update state
        {
            let mut state = resources.get_mut::<ReputationState>().await.unwrap();

            match category {
                Some(cat) => {
                    state.set_category(&subject_id, cat.to_string(), request.score);
                }
                None => {
                    state.set(&subject_id, request.score);
                }
            }

            // Apply auto-clamping if enabled
            let config = resources.get::<ReputationConfig>().await.unwrap();
            if config.auto_clamp {
                if let Some((min, max)) = config.score_range {
                    match category {
                        Some(cat) => {
                            state.clamp_score_category(&subject_id, cat, min, max);
                        }
                        None => {
                            state.clamp_score(&subject_id, min, max);
                        }
                    }
                }
            }
        };

        let new_score = request.score;

        // 4. Call hook: on_reputation_changed
        self.hook
            .on_reputation_changed(&subject_id, old_score, new_score, delta, category, resources)
            .await;

        // 5. Check for threshold crossing
        let new_threshold_name = {
            let config = resources.get::<ReputationConfig>().await.unwrap();
            config.get_threshold(new_score).map(|t| t.name.clone())
        };

        if old_threshold_name != new_threshold_name {
            if let Some(new_threshold_name) = &new_threshold_name {
                // Call hook: on_threshold_crossed
                let config = resources.get::<ReputationConfig>().await.unwrap();
                let old_threshold = old_threshold_name
                    .as_ref()
                    .and_then(|name| config.thresholds.iter().find(|t| &t.name == name));
                let new_threshold = config.thresholds.iter().find(|t| &t.name == new_threshold_name);

                if let Some(new_threshold) = new_threshold {
                    self.hook
                        .on_threshold_crossed(&subject_id, old_threshold, new_threshold, resources)
                        .await;
                }

                // Publish threshold crossed event
                let mut bus = resources.get_mut::<EventBus>().await.unwrap();
                bus.publish(ReputationThresholdCrossedEvent {
                    subject_id: subject_id.clone(),
                    old_threshold: old_threshold_name.clone(),
                    new_threshold: new_threshold_name.clone(),
                    score: new_score,
                    category: category.map(|s| s.to_string()),
                });
            }
        }

        // 6. Publish reputation changed event
        let mut bus = resources.get_mut::<EventBus>().await.unwrap();
        bus.publish(ReputationChangedEvent {
            subject_id,
            old_score,
            new_score,
            delta,
            category: request.category,
            reason: None,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::reputation::hook::DefaultReputationHook;
    use crate::plugin::reputation::types::*;

    #[tokio::test]
    async fn test_system_process_change_request() {
        let mut resources = ResourceContext::new();
        resources.insert(EventBus::new());
        resources.insert(ReputationConfig::default());
        resources.insert(ReputationState::new());

        let mut system = ReputationSystem::new(DefaultReputationHook);

        // Publish change request
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(ReputationChangeRequested {
                subject_id: SubjectId::new("player", "kingdom"),
                delta: 15.0,
                category: None,
                reason: Some("Completed quest".into()),
            });
            bus.dispatch();
        }

        // Process events
        system.process_events(&mut resources).await;

        // Check state
        let state = resources.get::<ReputationState>().await.unwrap();
        let score = state.get(&SubjectId::new("player", "kingdom")).unwrap();
        assert_eq!(score, 15.0);

        // Dispatch to make events visible
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.dispatch();
        }

        // Check event was published
        let mut bus = resources.get_mut::<EventBus>().await.unwrap();
        let reader = bus.reader::<ReputationChangedEvent>();
        let events: Vec<_> = reader.iter().cloned().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].new_score, 15.0);
        assert_eq!(events[0].delta, 15.0);
    }

    #[tokio::test]
    async fn test_system_process_set_request() {
        let mut resources = ResourceContext::new();
        resources.insert(EventBus::new());
        resources.insert(ReputationConfig::default());
        resources.insert(ReputationState::new());

        let mut system = ReputationSystem::new(DefaultReputationHook);

        // Publish set request
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(ReputationSetRequested {
                subject_id: SubjectId::new("player", "kingdom"),
                score: 75.0,
                category: None,
            });
            bus.dispatch();
        }

        // Process events
        system.process_events(&mut resources).await;

        // Check state
        let state = resources.get::<ReputationState>().await.unwrap();
        let score = state.get(&SubjectId::new("player", "kingdom")).unwrap();
        assert_eq!(score, 75.0);
    }

    #[tokio::test]
    async fn test_system_threshold_crossing() {
        let mut resources = ResourceContext::new();
        resources.insert(EventBus::new());

        // Setup config with thresholds
        let mut config = ReputationConfig::default();
        config.add_threshold(ReputationThreshold::new("Neutral", -10.0, 10.0));
        config.add_threshold(ReputationThreshold::new("Friendly", 10.0, 50.0));
        resources.insert(config);
        resources.insert(ReputationState::new());

        let mut system = ReputationSystem::new(DefaultReputationHook);

        // Publish change that crosses threshold
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(ReputationChangeRequested {
                subject_id: SubjectId::new("player", "kingdom"),
                delta: 15.0,
                category: None,
                reason: None,
            });
            bus.dispatch();
        }

        // Process events
        system.process_events(&mut resources).await;

        // Dispatch to make events visible
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.dispatch();
        }

        // Check threshold crossed event was published
        let mut bus = resources.get_mut::<EventBus>().await.unwrap();
        let reader = bus.reader::<ReputationThresholdCrossedEvent>();
        let events: Vec<_> = reader.iter().cloned().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].old_threshold, Some("Neutral".into()));
        assert_eq!(events[0].new_threshold, "Friendly");
    }
}
