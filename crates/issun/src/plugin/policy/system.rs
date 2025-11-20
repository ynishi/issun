//! Policy management system

use crate::context::{Context, ResourceContext, ServiceContext};
use crate::event::EventBus;
use crate::system::System;
use async_trait::async_trait;
use std::any::Any;
use std::sync::Arc;

use super::events::*;
use super::hook::PolicyHook;
use super::registry::PolicyRegistry;

/// System that processes policy events with hooks
///
/// This system:
/// 1. Processes policy activation requests
/// 2. Processes policy deactivation requests
/// 3. Processes policy cycling requests
/// 4. Calls hooks for custom behavior
/// 5. Publishes state change events for network replication
///
/// # Feedback Loop
///
/// ```text
/// Command Event → Validation (Hook) → Registry Update → Hook Call → State Event
/// ```
pub struct PolicySystem {
    hook: Arc<dyn PolicyHook>,
}

impl PolicySystem {
    /// Create a new PolicySystem with a custom hook
    pub fn new(hook: Arc<dyn PolicyHook>) -> Self {
        Self { hook }
    }

    /// Process policy activation requests
    ///
    /// Listens for `PolicyActivateRequested` events and:
    /// 1. Validates activation (via hook)
    /// 2. Activates policy and updates registry
    /// 3. Calls hook
    /// 4. Publishes `PolicyActivatedEvent`
    pub async fn process_activations(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        // Collect activation requests
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<PolicyActivateRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Get policy for validation
            let policy = {
                if let Some(registry) = resources.get::<PolicyRegistry>().await {
                    match registry.get(&request.policy_id) {
                        Some(p) => p.clone(),
                        None => continue, // Policy not found, skip
                    }
                } else {
                    continue;
                }
            };

            // Validate activation via hook (read-only resources access)
            {
                let resources_ref = resources as &ResourceContext;
                match self
                    .hook
                    .validate_activation(&policy, resources_ref)
                    .await
                {
                    Ok(()) => {},
                    Err(_) => continue, // Hook rejected activation
                }
            }

            // Get previous policy (before activation)
            let previous_policy = {
                if let Some(registry) = resources.get::<PolicyRegistry>().await {
                    registry.active_policy().cloned()
                } else {
                    None
                }
            };

            let previous_policy_id = previous_policy.as_ref().map(|p| p.id.clone());

            // Activate policy (update registry)
            {
                if let Some(mut registry) = resources.get_mut::<PolicyRegistry>().await {
                    if registry.config().allow_multiple_active {
                        if let Err(_) = registry.activate_multi(&request.policy_id) {
                            continue; // Failed to activate
                        }
                    } else {
                        if let Err(_) = registry.activate(&request.policy_id) {
                            continue; // Failed to activate
                        }
                    }
                } else {
                    continue;
                }
            }

            // Call hook (synchronous, immediate, local only)
            self.hook
                .on_policy_activated(&policy, previous_policy.as_ref(), resources)
                .await;

            // Publish event (asynchronous, for other systems and network)
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(PolicyActivatedEvent {
                    policy_id: policy.id.clone(),
                    policy_name: policy.name.clone(),
                    effects: policy.effects.clone(),
                    previous_policy_id,
                });
            }
        }
    }

    /// Process policy deactivation requests
    ///
    /// Listens for `PolicyDeactivateRequested` events and:
    /// 1. Deactivates policy and updates registry
    /// 2. Calls hook
    /// 3. Publishes `PolicyDeactivatedEvent`
    pub async fn process_deactivations(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        // Collect deactivation requests
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<PolicyDeactivateRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Get current policy (before deactivation)
            let (policy, policy_id) = {
                if let Some(registry) = resources.get::<PolicyRegistry>().await {
                    if let Some(specific_id) = &request.policy_id {
                        // Multi-active mode: deactivate specific policy
                        match registry.get(specific_id) {
                            Some(p) => (Some(p.clone()), specific_id.clone()),
                            None => continue, // Policy not found
                        }
                    } else {
                        // Single-active mode: deactivate current policy
                        match registry.active_policy() {
                            Some(p) => (Some(p.clone()), p.id.clone()),
                            None => continue, // No active policy
                        }
                    }
                } else {
                    continue;
                }
            };

            // Deactivate policy (update registry)
            {
                if let Some(mut registry) = resources.get_mut::<PolicyRegistry>().await {
                    if registry.config().allow_multiple_active {
                        if let Err(_) = registry.deactivate_multi(&policy_id) {
                            continue; // Failed to deactivate
                        }
                    } else {
                        registry.deactivate();
                    }
                } else {
                    continue;
                }
            }

            // Call hook if policy was found
            if let Some(ref policy) = policy {
                self.hook.on_policy_deactivated(policy, resources).await;
            }

            // Publish event (asynchronous, for other systems and network)
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(PolicyDeactivatedEvent {
                    policy_id,
                    policy_name: policy.map(|p| p.name).unwrap_or_default(),
                });
            }
        }
    }

    /// Process policy cycling requests
    ///
    /// Listens for `PolicyCycleRequested` events and:
    /// 1. Cycles to the next policy in the registry
    /// 2. Calls activation hook
    /// 3. Publishes `PolicyActivatedEvent`
    pub async fn process_cycles(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        // Collect cycle requests
        let has_request = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<PolicyCycleRequested>();
                !reader.iter().collect::<Vec<_>>().is_empty()
            } else {
                false
            }
        };

        if !has_request {
            return;
        }

        // Get previous policy (before cycle)
        let previous_policy = {
            if let Some(registry) = resources.get::<PolicyRegistry>().await {
                registry.active_policy().cloned()
            } else {
                None
            }
        };

        let previous_policy_id = previous_policy.as_ref().map(|p| p.id.clone());

        // Cycle policy (update registry)
        {
            if let Some(mut registry) = resources.get_mut::<PolicyRegistry>().await {
                if let Err(_) = registry.cycle() {
                    return; // Failed to cycle
                }
            } else {
                return;
            }
        }

        // Get newly activated policy
        let policy = {
            if let Some(registry) = resources.get::<PolicyRegistry>().await {
                match registry.active_policy() {
                    Some(p) => p.clone(),
                    None => return, // No active policy after cycle
                }
            } else {
                return;
            }
        };

        // Call hook (synchronous, immediate, local only)
        self.hook
            .on_policy_activated(&policy, previous_policy.as_ref(), resources)
            .await;

        // Publish event (asynchronous, for other systems and network)
        if let Some(mut bus) = resources.get_mut::<EventBus>().await {
            bus.publish(PolicyActivatedEvent {
                policy_id: policy.id.clone(),
                policy_name: policy.name.clone(),
                effects: policy.effects.clone(),
                previous_policy_id,
            });
        }
    }
}

#[async_trait]
impl System for PolicySystem {
    fn name(&self) -> &'static str {
        "policy_system"
    }

    async fn initialize(&mut self, _context: &mut Context) {}

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::ResourceContext;
    use crate::event::EventBus;
    use crate::plugin::policy::{DefaultPolicyHook, Policy, PolicyId};

    #[tokio::test]
    async fn test_system_creation() {
        let hook = Arc::new(DefaultPolicyHook);
        let system = PolicySystem::new(hook);
        assert_eq!(system.name(), "policy_system");
    }

    #[tokio::test]
    async fn test_process_activation() {
        let hook = Arc::new(DefaultPolicyHook);
        let mut system = PolicySystem::new(hook);
        let mut resources = ResourceContext::new();

        // Setup registry and event bus
        let mut registry = PolicyRegistry::new();
        let policy = Policy::new("test", "Test Policy", "Test");
        registry.add(policy);
        resources.insert(registry);
        resources.insert(EventBus::new());

        // Publish activation request
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(PolicyActivateRequested {
                policy_id: PolicyId::new("test"),
            });
        }

        // Process activations
        let services = ServiceContext::new();
        system.process_activations(&services, &mut resources).await;

        // Verify policy is activated
        {
            let registry = resources.get::<PolicyRegistry>().await.unwrap();
            assert_eq!(
                registry.active_policy().unwrap().id.as_str(),
                "test"
            );
        }

        // Verify event was published
        let mut bus = resources.get_mut::<EventBus>().await.unwrap();
        let reader = bus.reader::<PolicyActivatedEvent>();
        let events: Vec<_> = reader.iter().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].policy_id.as_str(), "test");
    }

    #[tokio::test]
    async fn test_process_deactivation() {
        let hook = Arc::new(DefaultPolicyHook);
        let mut system = PolicySystem::new(hook);
        let mut resources = ResourceContext::new();

        // Setup registry with active policy
        let mut registry = PolicyRegistry::new();
        let policy = Policy::new("test", "Test Policy", "Test");
        registry.add(policy);
        registry.activate(&PolicyId::new("test")).unwrap();
        resources.insert(registry);
        resources.insert(EventBus::new());

        // Publish deactivation request
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(PolicyDeactivateRequested { policy_id: None });
        }

        // Process deactivations
        let services = ServiceContext::new();
        system
            .process_deactivations(&services, &mut resources)
            .await;

        // Verify policy is deactivated
        {
            let registry = resources.get::<PolicyRegistry>().await.unwrap();
            assert!(registry.active_policy().is_none());
        }

        // Verify event was published
        let mut bus = resources.get_mut::<EventBus>().await.unwrap();
        let reader = bus.reader::<PolicyDeactivatedEvent>();
        let events: Vec<_> = reader.iter().collect();
        assert_eq!(events.len(), 1);
    }
}
