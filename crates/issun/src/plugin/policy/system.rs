//! Policy management system

use crate::context::{Context, ResourceContext, ServiceContext};
use crate::event::EventBus;
use crate::system::System;
use async_trait::async_trait;
use std::any::Any;
use std::sync::Arc;

use super::config::PolicyConfig;
use super::events::*;
use super::hook::PolicyHook;
use super::policies::Policies;
use super::state::PolicyState;

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
/// Command Event → Validation (Hook) → State Update → Hook Call → State Event
/// ```
#[derive(Clone)]
pub struct PolicySystem {
    #[allow(dead_code)]
    hook: Arc<dyn PolicyHook>,
}

#[allow(dead_code)]
impl PolicySystem {
    /// Create a new PolicySystem with a custom hook
    pub fn new(hook: Arc<dyn PolicyHook>) -> Self {
        Self { hook }
    }

    /// Process all policy events
    pub async fn process_events(
        &mut self,
        services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        self.process_activations(services, resources).await;
        self.process_deactivations(services, resources).await;
        self.process_cycles(services, resources).await;
    }

    /// Process policy activation requests
    ///
    /// Listens for `PolicyActivateRequested` events and:
    /// 1. Validates activation (via hook)
    /// 2. Activates policy and updates state
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
                if let Some(policies) = resources.get::<Policies>().await {
                    match policies.get(&request.policy_id) {
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
                match self.hook.validate_activation(&policy, resources_ref).await {
                    Ok(()) => {}
                    Err(_) => continue, // Hook rejected activation
                }
            }

            // Get previous policy (before activation)
            let previous_policy_id = {
                let state = match resources.get::<PolicyState>().await {
                    Some(s) => s,
                    None => continue,
                };
                state.active_policy_id().cloned()
            };

            let previous_policy = {
                if let Some(id) = &previous_policy_id {
                    if let Some(policies) = resources.get::<Policies>().await {
                        policies.get(id).cloned()
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            // Activate policy (update state)
            {
                let config = match resources.get::<PolicyConfig>().await {
                    Some(c) => c,
                    None => continue,
                };
                let mut state = match resources.get_mut::<PolicyState>().await {
                    Some(s) => s,
                    None => continue,
                };

                if config.allow_multiple_active {
                    if !state.activate_multi(request.policy_id.clone()) {
                        continue; // Already active
                    }
                } else {
                    state.activate(request.policy_id.clone());
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
    /// 1. Deactivates policy and updates state
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
            // Determine which policy to deactivate
            let policy_id = {
                if let Some(specific_id) = &request.policy_id {
                    // Multi-active mode: deactivate specific policy
                    specific_id.clone()
                } else {
                    // Single-active mode: deactivate current policy
                    let state = match resources.get::<PolicyState>().await {
                        Some(s) => s,
                        None => continue,
                    };
                    match state.active_policy_id() {
                        Some(id) => id.clone(),
                        None => continue, // No active policy
                    }
                }
            };

            // Get policy for hook call
            let policy = {
                if let Some(policies) = resources.get::<Policies>().await {
                    policies.get(&policy_id).cloned()
                } else {
                    None
                }
            };

            // Deactivate policy (update state)
            {
                let config = match resources.get::<PolicyConfig>().await {
                    Some(c) => c,
                    None => continue,
                };
                let mut state = match resources.get_mut::<PolicyState>().await {
                    Some(s) => s,
                    None => continue,
                };

                if config.allow_multiple_active {
                    if !state.deactivate_multi(&policy_id) {
                        continue; // Was not active
                    }
                } else {
                    state.deactivate();
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

        // Check if cycling is enabled
        let cycling_enabled = {
            if let Some(config) = resources.get::<PolicyConfig>().await {
                config.enable_cycling
            } else {
                return;
            }
        };

        if !cycling_enabled {
            return;
        }

        // Get previous policy (before cycle)
        let previous_policy_id = {
            let state = match resources.get::<PolicyState>().await {
                Some(s) => s,
                None => return,
            };
            state.active_policy_id().cloned()
        };

        let previous_policy = {
            if let Some(id) = &previous_policy_id {
                if let Some(policies) = resources.get::<Policies>().await {
                    policies.get(id).cloned()
                } else {
                    None
                }
            } else {
                None
            }
        };

        // Cycle to next policy
        let next_policy_id = {
            let policies = match resources.get::<Policies>().await {
                Some(p) => p,
                None => return,
            };

            let policy_ids = policies.policy_ids();
            if policy_ids.is_empty() {
                return; // No policies to cycle through
            }

            if let Some(current_id) = &previous_policy_id {
                // Find current index and move to next
                if let Some(index) = policy_ids.iter().position(|id| id == current_id) {
                    let next_index = (index + 1) % policy_ids.len();
                    policy_ids[next_index].clone()
                } else {
                    // Current policy not found, activate first
                    policy_ids[0].clone()
                }
            } else {
                // No active policy, activate first
                policy_ids[0].clone()
            }
        };

        // Activate next policy
        {
            let mut state = match resources.get_mut::<PolicyState>().await {
                Some(s) => s,
                None => return,
            };
            state.activate(next_policy_id.clone());
        }

        // Get newly activated policy
        let policy = {
            if let Some(policies) = resources.get::<Policies>().await {
                match policies.get(&next_policy_id) {
                    Some(p) => p.clone(),
                    None => return, // Policy not found
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

        // Setup policies, config, state, and event bus
        let mut policies = Policies::new();
        let policy = Policy::new("test", "Test Policy", "Test");
        policies.add(policy);
        resources.insert(policies);
        resources.insert(PolicyConfig::default());
        resources.insert(PolicyState::new());
        resources.insert(EventBus::new());

        // Publish activation request
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(PolicyActivateRequested {
                policy_id: PolicyId::new("test"),
            });
            bus.dispatch();
        }

        // Process activations
        let services = ServiceContext::new();
        system.process_events(&services, &mut resources).await;

        // Verify policy is activated
        {
            let state = resources.get::<PolicyState>().await.unwrap();
            assert_eq!(state.active_policy_id().unwrap().as_str(), "test");
        }

        // Dispatch to make events visible
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.dispatch();
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

        // Setup with active policy
        let mut policies = Policies::new();
        let policy = Policy::new("test", "Test Policy", "Test");
        policies.add(policy);
        resources.insert(policies);
        resources.insert(PolicyConfig::default());

        let mut state = PolicyState::new();
        state.activate(PolicyId::new("test"));
        resources.insert(state);
        resources.insert(EventBus::new());

        // Publish deactivation request
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(PolicyDeactivateRequested { policy_id: None });
            bus.dispatch();
        }

        // Process deactivations
        let services = ServiceContext::new();
        system.process_events(&services, &mut resources).await;

        // Verify policy is deactivated
        {
            let state = resources.get::<PolicyState>().await.unwrap();
            assert!(state.active_policy_id().is_none());
        }

        // Dispatch to make events visible
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.dispatch();
        }

        // Verify event was published
        let mut bus = resources.get_mut::<EventBus>().await.unwrap();
        let reader = bus.reader::<PolicyDeactivatedEvent>();
        let events: Vec<_> = reader.iter().collect();
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn test_process_cycle() {
        let hook = Arc::new(DefaultPolicyHook);
        let mut system = PolicySystem::new(hook);
        let mut resources = ResourceContext::new();

        // Setup with multiple policies
        let mut policies = Policies::new();
        policies.add(Policy::new("policy1", "Policy 1", "Test"));
        policies.add(Policy::new("policy2", "Policy 2", "Test"));
        policies.add(Policy::new("policy3", "Policy 3", "Test"));
        resources.insert(policies);

        let config = PolicyConfig {
            enable_cycling: true,
            ..Default::default()
        };
        resources.insert(config);

        let mut state = PolicyState::new();
        state.activate(PolicyId::new("policy1"));
        resources.insert(state);
        resources.insert(EventBus::new());

        // Publish cycle request
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(PolicyCycleRequested);
            bus.dispatch();
        }

        // Process cycles
        let services = ServiceContext::new();
        system.process_events(&services, &mut resources).await;

        // Verify policy cycled to next (policy2)
        {
            let state = resources.get::<PolicyState>().await.unwrap();
            assert_eq!(state.active_policy_id().unwrap().as_str(), "policy2");
        }

        // Dispatch to make events visible
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.dispatch();
        }

        // Verify event was published
        let mut bus = resources.get_mut::<EventBus>().await.unwrap();
        let reader = bus.reader::<PolicyActivatedEvent>();
        let events: Vec<_> = reader.iter().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].policy_id.as_str(), "policy2");
    }
}
