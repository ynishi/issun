//! Reputation system for processing events

use crate::context::ResourceContext;
use crate::event::EventBus;

use super::events::*;
use super::hook::ReputationHook;
use super::registry::ReputationRegistry;

/// System for processing reputation events
///
/// This system:
/// 1. Listens for `ReputationChangeRequested` and `ReputationSetRequested` events
/// 2. Validates changes via hook
/// 3. Calculates effective delta via hook
/// 4. Updates `ReputationRegistry`
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
            let mut registry = match resources.get_mut::<ReputationRegistry>().await {
                Some(r) => r,
                None => return,
            };

            let entry = match category {
                Some(cat) => registry.get_or_create_category(subject_id.clone(), cat.to_string()),
                None => registry.get_or_create(subject_id.clone()),
            };

            let old_score = entry.score;
            let old_threshold = registry.get_threshold(old_score).map(|t| t.name.clone());

            (old_score, old_threshold)
        };

        // 4. Update registry
        let new_score = {
            let mut registry = resources.get_mut::<ReputationRegistry>().await.unwrap();

            match category {
                Some(cat) => {
                    registry.adjust_category(subject_id.clone(), cat.to_string(), effective_delta);
                    registry
                        .get_category(&subject_id, cat)
                        .map(|e| e.score)
                        .unwrap_or(old_score)
                }
                None => {
                    registry.adjust(subject_id.clone(), effective_delta);
                    registry.get(&subject_id).map(|e| e.score).unwrap_or(old_score)
                }
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
            let registry = resources.get::<ReputationRegistry>().await.unwrap();
            registry.get_threshold(new_score).map(|t| t.name.clone())
        };

        if old_threshold_name != new_threshold_name {
            if let Some(new_threshold_name) = &new_threshold_name {
                // Call hook: on_threshold_crossed
                let registry = resources.get::<ReputationRegistry>().await.unwrap();
                let old_threshold = old_threshold_name
                    .as_ref()
                    .and_then(|name| registry.thresholds().iter().find(|t| &t.name == name));
                let new_threshold = registry
                    .thresholds()
                    .iter()
                    .find(|t| &t.name == new_threshold_name);

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
            let registry = match resources.get::<ReputationRegistry>().await {
                Some(r) => r,
                None => return,
            };

            match category {
                Some(cat) => registry
                    .get_category(&subject_id, cat)
                    .map(|e| e.score)
                    .unwrap_or(registry.config().default_score),
                None => registry
                    .get(&subject_id)
                    .map(|e| e.score)
                    .unwrap_or(registry.config().default_score),
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
            let registry = resources.get::<ReputationRegistry>().await.unwrap();
            registry.get_threshold(old_score).map(|t| t.name.clone())
        };

        // 3. Update registry
        {
            let mut registry = resources.get_mut::<ReputationRegistry>().await.unwrap();

            match category {
                Some(cat) => {
                    registry.set_category(subject_id.clone(), cat.to_string(), request.score);
                }
                None => {
                    registry.set(subject_id.clone(), request.score);
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
            let registry = resources.get::<ReputationRegistry>().await.unwrap();
            registry.get_threshold(new_score).map(|t| t.name.clone())
        };

        if old_threshold_name != new_threshold_name {
            if let Some(new_threshold_name) = &new_threshold_name {
                // Call hook: on_threshold_crossed
                let registry = resources.get::<ReputationRegistry>().await.unwrap();
                let old_threshold = old_threshold_name
                    .as_ref()
                    .and_then(|name| registry.thresholds().iter().find(|t| &t.name == name));
                let new_threshold = registry
                    .thresholds()
                    .iter()
                    .find(|t| &t.name == new_threshold_name);

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
        resources.insert(ReputationRegistry::new());

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
        }

        // Process events
        system.process_events(&mut resources).await;

        // Check registry
        let registry = resources.get::<ReputationRegistry>().await.unwrap();
        let entry = registry.get(&SubjectId::new("player", "kingdom")).unwrap();
        assert_eq!(entry.score, 15.0);

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
        resources.insert(ReputationRegistry::new());

        let mut system = ReputationSystem::new(DefaultReputationHook);

        // Publish set request
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(ReputationSetRequested {
                subject_id: SubjectId::new("player", "kingdom"),
                score: 75.0,
                category: None,
            });
        }

        // Process events
        system.process_events(&mut resources).await;

        // Check registry
        let registry = resources.get::<ReputationRegistry>().await.unwrap();
        let entry = registry.get(&SubjectId::new("player", "kingdom")).unwrap();
        assert_eq!(entry.score, 75.0);
    }

    #[tokio::test]
    async fn test_system_threshold_crossing() {
        let mut resources = ResourceContext::new();
        resources.insert(EventBus::new());

        // Setup registry with thresholds
        let mut registry = ReputationRegistry::new();
        registry.add_threshold(ReputationThreshold::new("Neutral", -10.0, 10.0));
        registry.add_threshold(ReputationThreshold::new("Friendly", 10.0, 50.0));
        resources.insert(registry);

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
        }

        // Process events
        system.process_events(&mut resources).await;

        // Check threshold crossed event was published
        let mut bus = resources.get_mut::<EventBus>().await.unwrap();
        let reader = bus.reader::<ReputationThresholdCrossedEvent>();
        let events: Vec<_> = reader.iter().cloned().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].old_threshold, Some("Neutral".into()));
        assert_eq!(events[0].new_threshold, "Friendly");
    }
}
