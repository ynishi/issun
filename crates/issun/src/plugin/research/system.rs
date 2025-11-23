//! Research management system

use crate::context::{ResourceContext, ServiceContext};
use crate::event::EventBus;
use crate::system::System;
use async_trait::async_trait;
use std::any::Any;
use std::sync::Arc;

use super::config::ResearchConfig;
use super::events::*;
use super::hook::ResearchHook;
use super::research_projects::ResearchProjects;
use super::state::ResearchState;
use super::types::*;

/// System that processes research events with hooks
///
/// This system:
/// 1. Processes research queue requests
/// 2. Processes research start/cancel requests
/// 3. Processes progress updates (manual or auto)
/// 4. Calls hooks for custom behavior
/// 5. Publishes state change events for network replication
///
/// # Feedback Loop
///
/// ```text
/// Command Event → Validation (Hook) → State Update → Hook Call → State Event
/// ```
#[derive(Clone)]
pub struct ResearchSystem {
    hook: Arc<dyn ResearchHook>,
}

impl ResearchSystem {
    /// Create a new ResearchSystem with a custom hook
    pub fn new(hook: Arc<dyn ResearchHook>) -> Self {
        Self { hook }
    }

    /// Process all research events
    pub async fn process_events(
        &mut self,
        services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        self.process_queue_requests(services, resources).await;
        self.process_start_requests(services, resources).await;
        self.process_cancel_requests(services, resources).await;
        self.process_progress_requests(services, resources).await;
        self.process_complete_requests(services, resources).await;

        // Auto-advance progress if enabled
        self.auto_advance_progress(resources).await;
    }

    /// Process research queue requests
    async fn process_queue_requests(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        // Collect queue requests
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<ResearchQueueRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Get project for validation
            let project = {
                if let Some(projects) = resources.get::<ResearchProjects>().await {
                    match projects.get(&request.project_id) {
                        Some(p) => p.clone(),
                        None => continue,
                    }
                } else {
                    continue;
                }
            };

            // Validate prerequisites via hook
            {
                let resources_ref = resources as &ResourceContext;
                match self
                    .hook
                    .validate_prerequisites(&project, resources_ref)
                    .await
                {
                    Ok(()) => {}
                    Err(_) => continue,
                }
            }

            // Calculate cost via hook
            let cost = {
                let resources_ref = resources as &ResourceContext;
                match self
                    .hook
                    .calculate_research_cost(&project, resources_ref)
                    .await
                {
                    Ok(c) => c,
                    Err(_) => continue,
                }
            };

            // Queue research (update state)
            {
                if let Some(mut state) = resources.get_mut::<ResearchState>().await {
                    if state.queue(&request.project_id).is_err() {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            // Auto-activate if slots available
            {
                let config = resources.get::<ResearchConfig>().await;
                let state = resources.get_mut::<ResearchState>().await;

                if let (Some(cfg), Some(mut st)) = (config, state) {
                    let max_slots = if cfg.allow_parallel_research {
                        cfg.max_parallel_slots
                    } else {
                        1
                    };
                    st.activate_next_queued(max_slots);
                }
            }

            // Call hook
            self.hook.on_research_queued(&project, resources).await;

            // Check if project was immediately started
            let was_started = {
                if let Some(state) = resources.get::<ResearchState>().await {
                    state.get_status(&request.project_id) == ResearchStatus::InProgress
                } else {
                    false
                }
            };

            // Publish events
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(ResearchQueuedEvent {
                    project_id: project.id.clone(),
                    project_name: project.name.clone(),
                    cost,
                });

                if was_started {
                    bus.publish(ResearchStartedEvent {
                        project_id: project.id.clone(),
                        project_name: project.name.clone(),
                    });
                }
            }
        }
    }

    /// Process research start requests
    async fn process_start_requests(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        let _requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<ResearchStartRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        // Manual start not implemented - auto-activated by queue
    }

    /// Process research cancel requests
    async fn process_cancel_requests(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<ResearchCancelRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Get project
            let project = {
                if let Some(projects) = resources.get::<ResearchProjects>().await {
                    match projects.get(&request.project_id) {
                        Some(p) => p.clone(),
                        None => continue,
                    }
                } else {
                    continue;
                }
            };

            // Cancel (update state)
            {
                if let Some(mut state) = resources.get_mut::<ResearchState>().await {
                    if state.cancel(&request.project_id).is_err() {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            // Auto-activate next queued
            {
                let config = resources.get::<ResearchConfig>().await;
                let state = resources.get_mut::<ResearchState>().await;

                if let (Some(cfg), Some(mut st)) = (config, state) {
                    let max_slots = if cfg.allow_parallel_research {
                        cfg.max_parallel_slots
                    } else {
                        1
                    };
                    st.activate_next_queued(max_slots);
                }
            }

            // Call hook
            self.hook.on_research_failed(&project, resources).await;

            // Publish event
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(ResearchCancelledEvent {
                    project_id: project.id.clone(),
                    project_name: project.name.clone(),
                });
            }
        }
    }

    /// Process research progress requests
    async fn process_progress_requests(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<ResearchProgressRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Add progress
            {
                if let Some(mut state) = resources.get_mut::<ResearchState>().await {
                    state.add_progress(&request.project_id, request.amount);
                } else {
                    continue;
                }
            }

            // Check completion
            let (completed, progress) = {
                if let Some(state) = resources.get::<ResearchState>().await {
                    let prog = state.get_progress(&request.project_id);
                    (prog >= 1.0, prog)
                } else {
                    continue;
                }
            };

            if completed {
                // Complete project
                if let Some(mut state) = resources.get_mut::<ResearchState>().await {
                    let _ = state.complete(&request.project_id);
                }

                // Auto-activate next
                {
                    let config = resources.get::<ResearchConfig>().await;
                    let state = resources.get_mut::<ResearchState>().await;

                    if let (Some(cfg), Some(mut st)) = (config, state) {
                        let max_slots = if cfg.allow_parallel_research {
                            cfg.max_parallel_slots
                        } else {
                            1
                        };
                        st.activate_next_queued(max_slots);
                    }
                }

                // Get project for events
                let project = {
                    if let Some(projects) = resources.get::<ResearchProjects>().await {
                        projects.get(&request.project_id).cloned()
                    } else {
                        None
                    }
                };

                if let Some(proj) = project {
                    let result = ResearchResult {
                        project_id: proj.id.clone(),
                        success: true,
                        final_metrics: proj.metrics.clone(),
                        metadata: proj.metadata.clone(),
                    };

                    // Call hook
                    self.hook
                        .on_research_completed(&proj, &result, resources)
                        .await;

                    // Publish event
                    if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                        bus.publish(ResearchCompletedEvent {
                            project_id: proj.id.clone(),
                            project_name: proj.name.clone(),
                            result,
                        });
                    }
                }
            } else {
                // Publish progress event
                if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                    bus.publish(ResearchProgressUpdatedEvent {
                        project_id: request.project_id.clone(),
                        progress,
                    });
                }
            }
        }
    }

    /// Process research complete requests (manual)
    async fn process_complete_requests(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<ResearchCompleteRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Get project
            let project = {
                if let Some(projects) = resources.get::<ResearchProjects>().await {
                    match projects.get(&request.project_id) {
                        Some(p) => p.clone(),
                        None => continue,
                    }
                } else {
                    continue;
                }
            };

            // Complete (update state)
            {
                if let Some(mut state) = resources.get_mut::<ResearchState>().await {
                    if state.complete(&request.project_id).is_err() {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            // Auto-activate next
            {
                let config = resources.get::<ResearchConfig>().await;
                let state = resources.get_mut::<ResearchState>().await;

                if let (Some(cfg), Some(mut st)) = (config, state) {
                    let max_slots = if cfg.allow_parallel_research {
                        cfg.max_parallel_slots
                    } else {
                        1
                    };
                    st.activate_next_queued(max_slots);
                }
            }

            let result = ResearchResult {
                project_id: project.id.clone(),
                success: true,
                final_metrics: project.metrics.clone(),
                metadata: project.metadata.clone(),
            };

            // Call hook
            self.hook
                .on_research_completed(&project, &result, resources)
                .await;

            // Publish event
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(ResearchCompletedEvent {
                    project_id: project.id.clone(),
                    project_name: project.name.clone(),
                    result,
                });
            }
        }
    }

    /// Auto-advance progress for active research
    async fn auto_advance_progress(&mut self, resources: &mut ResourceContext) {
        // Check if auto-advance is enabled
        let (enabled, progress_per_turn) = {
            if let Some(config) = resources.get::<ResearchConfig>().await {
                (config.auto_advance, config.base_progress_per_turn)
            } else {
                return;
            }
        };

        if !enabled {
            return;
        }

        // Get active projects
        let active_ids = {
            if let Some(state) = resources.get::<ResearchState>().await {
                state.active_projects()
            } else {
                return;
            }
        };

        let mut completed = Vec::new();

        // Advance progress for each
        {
            if let Some(mut state) = resources.get_mut::<ResearchState>().await {
                for id in &active_ids {
                    state.add_progress(id, progress_per_turn);

                    if state.get_progress(id) >= 1.0 {
                        let _ = state.complete(id);
                        completed.push(id.clone());
                    }
                }
            }
        }

        // Auto-activate next after completions
        if !completed.is_empty() {
            let config = resources.get::<ResearchConfig>().await;
            let state = resources.get_mut::<ResearchState>().await;

            if let (Some(cfg), Some(mut st)) = (config, state) {
                let max_slots = if cfg.allow_parallel_research {
                    cfg.max_parallel_slots
                } else {
                    1
                };
                st.activate_next_queued(max_slots);
            }
        }

        // Publish completion events
        for project_id in completed {
            let project = {
                if let Some(projects) = resources.get::<ResearchProjects>().await {
                    projects.get(&project_id).cloned()
                } else {
                    None
                }
            };

            if let Some(proj) = project {
                let result = ResearchResult {
                    project_id: proj.id.clone(),
                    success: true,
                    final_metrics: proj.metrics.clone(),
                    metadata: proj.metadata.clone(),
                };

                self.hook
                    .on_research_completed(&proj, &result, resources)
                    .await;

                if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                    bus.publish(ResearchCompletedEvent {
                        project_id: proj.id.clone(),
                        project_name: proj.name.clone(),
                        result,
                    });
                }
            }
        }
    }
}

#[async_trait]
impl System for ResearchSystem {
    fn name(&self) -> &'static str {
        "research_system"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
