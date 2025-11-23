//! System orchestration for contagion propagation

use super::config::ContagionConfig;
use super::hook::ContagionHook;
use super::service::ContagionService;
use super::state::{Contagion, ContagionState};
use super::topology::GraphTopology;
use super::types::{ContagionId, NodeId};
use crate::context::ResourceContext;
use crate::system::System;
use async_trait::async_trait;
use rand::Rng;
use std::any::Any;
use std::sync::Arc;

/// System for orchestrating contagion propagation
#[derive(Clone)]
pub struct ContagionSystem {
    hook: Arc<dyn ContagionHook>,
}

impl ContagionSystem {
    /// Create a new contagion system with a hook
    pub fn new(hook: Arc<dyn ContagionHook>) -> Self {
        Self { hook }
    }

    /// Propagate all active contagions through the graph
    ///
    /// Returns the number of new spreads that occurred
    pub async fn propagate_contagions(
        &self,
        resources: &mut ResourceContext,
    ) -> Result<PropagationReport, String> {
        let config = resources
            .get::<ContagionConfig>()
            .await
            .ok_or("ContagionConfig not found")?;

        let topology = resources
            .get::<GraphTopology>()
            .await
            .ok_or("GraphTopology not found")?;

        let mut state = resources
            .get_mut::<ContagionState>()
            .await
            .ok_or("ContagionState not found")?;

        let mut new_spreads: Vec<SpreadEvent> = Vec::new();
        let mut rng = rand::thread_rng();

        // Collect spread events (can't mutate while iterating)
        for (contagion_id, contagion) in state.all_contagions() {
            for node_id in contagion.spread.clone() {
                let outgoing_edges = topology.get_outgoing_edges(&node_id);

                for edge in outgoing_edges {
                    // Skip if already spread to target
                    if contagion.has_reached(&edge.to) {
                        continue;
                    }

                    let target_node = topology
                        .get_node(&edge.to)
                        .ok_or_else(|| format!("Target node {} not found", edge.to))?;

                    // Modify transmission rate via hook
                    let modified_rate = self
                        .hook
                        .modify_transmission_rate(edge.transmission_rate, edge, contagion)
                        .await;

                    // Create modified edge for propagation check
                    let mut modified_edge = edge.clone();
                    modified_edge.transmission_rate = modified_rate;

                    // Check if propagation occurs
                    if ContagionService::should_propagate(
                        contagion,
                        &modified_edge,
                        target_node,
                        &config,
                        &mut rng,
                    ) {
                        // Check for mutation
                        if let Some(mutated) = ContagionService::mutate_contagion(
                            contagion,
                            edge.noise_level,
                            &mut rng,
                        ) {
                            // Mutated version
                            new_spreads.push(SpreadEvent::Mutated {
                                original_id: contagion_id.clone(),
                                mutated_contagion: mutated,
                                from_node: node_id.clone(),
                                to_node: edge.to.clone(),
                            });
                        } else {
                            // Normal spread
                            new_spreads.push(SpreadEvent::Normal {
                                contagion_id: contagion_id.clone(),
                                from_node: node_id.clone(),
                                to_node: edge.to.clone(),
                            });
                        }

                        // Call hook
                        self.hook
                            .on_contagion_spread(contagion, &node_id, &edge.to)
                            .await;
                    }
                }
            }
        }

        // Apply all spreads
        let mut spread_count = 0;
        let mut mutation_count = 0;
        let mut spread_details = Vec::new();

        for event in new_spreads {
            match event {
                SpreadEvent::Normal {
                    contagion_id,
                    from_node,
                    to_node,
                } => {
                    if let Some(contagion) = state.get_contagion_mut(&contagion_id) {
                        contagion.add_spread(to_node.clone());
                        spread_count += 1;

                        spread_details.push(SpreadDetail {
                            from_node,
                            to_node,
                            contagion_id,
                            is_mutation: false,
                            original_id: None,
                        });
                    }
                }
                SpreadEvent::Mutated {
                    original_id,
                    mut mutated_contagion,
                    from_node,
                    to_node,
                } => {
                    // Generate new ID for mutated version (use timestamp + random)
                    let new_id = format!(
                        "contagion_{}_{}",
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_nanos(),
                        rng.gen::<u64>()
                    );
                    mutated_contagion.id = new_id.clone();
                    mutated_contagion.add_spread(to_node.clone());

                    state.spawn_contagion(mutated_contagion);
                    spread_count += 1;
                    mutation_count += 1;

                    spread_details.push(SpreadDetail {
                        from_node,
                        to_node,
                        contagion_id: new_id,
                        is_mutation: true,
                        original_id: Some(original_id),
                    });
                }
            }
        }

        Ok(PropagationReport {
            spread_count,
            mutation_count,
            spread_details,
        })
    }

    /// Decay credibility and remove dead contagions
    ///
    /// Returns the number of contagions removed
    pub async fn decay_contagions(
        &self,
        resources: &mut ResourceContext,
        elapsed_turns: u64,
    ) -> Result<usize, String> {
        let config = resources
            .get::<ContagionConfig>()
            .await
            .ok_or("ContagionConfig not found")?;

        let mut state = resources
            .get_mut::<ContagionState>()
            .await
            .ok_or("ContagionState not found")?;

        // Collect all IDs and credibilities first (read-only phase)
        let contagion_data: Vec<(ContagionId, f32)> = state
            .all_contagions()
            .map(|(id, contagion)| {
                let new_credibility = ContagionService::decay_credibility(
                    contagion.credibility,
                    elapsed_turns,
                    config.lifetime_turns,
                );
                (id.clone(), new_credibility)
            })
            .collect();

        // Determine which to remove and which to update
        let mut to_remove = Vec::new();
        let mut to_update = Vec::new();

        for (id, new_credibility) in contagion_data {
            if new_credibility < config.min_credibility {
                to_remove.push(id);
            } else {
                to_update.push((id, new_credibility));
            }
        }

        // Apply decay to remaining contagions (mutation phase)
        for (id, new_credibility) in to_update {
            if let Some(c) = state.get_contagion_mut(&id) {
                c.credibility = new_credibility;
            }
        }

        // Remove dead contagions
        let removed_count = to_remove.len();
        for id in to_remove {
            state.remove_contagion(&id);
        }

        Ok(removed_count)
    }

    /// Get a reference to a specific node's contagions
    pub async fn get_node_contagions(
        &self,
        resources: &ResourceContext,
        node_id: &NodeId,
    ) -> Result<Vec<Contagion>, String> {
        let state = resources
            .get::<ContagionState>()
            .await
            .ok_or("ContagionState not found")?;

        Ok(state
            .get_contagions_at_node(node_id)
            .into_iter()
            .cloned()
            .collect())
    }
}

#[async_trait]
impl System for ContagionSystem {
    fn name(&self) -> &'static str {
        "contagion_system"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Default for ContagionSystem {
    fn default() -> Self {
        Self::new(Arc::new(super::hook::DefaultContagionHook))
    }
}

/// Event types for spreading
enum SpreadEvent {
    Normal {
        contagion_id: ContagionId,
        from_node: NodeId,
        to_node: NodeId,
    },
    Mutated {
        original_id: ContagionId,
        mutated_contagion: Contagion,
        from_node: NodeId,
        to_node: NodeId,
    },
}

/// Report of propagation results
#[derive(Debug, Clone)]
pub struct PropagationReport {
    pub spread_count: usize,
    pub mutation_count: usize,
    pub spread_details: Vec<SpreadDetail>,
}

/// Details of a single spread event
#[derive(Debug, Clone)]
pub struct SpreadDetail {
    pub from_node: NodeId,
    pub to_node: NodeId,
    pub contagion_id: ContagionId,
    pub is_mutation: bool,
    pub original_id: Option<ContagionId>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::contagion::types::{ContagionContent, DiseaseLevel};
    use crate::plugin::contagion::{ContagionNode, NodeType, PropagationEdge};

    #[tokio::test]
    async fn test_system_creation() {
        let _system = ContagionSystem::default();
        // Should not panic
    }

    #[tokio::test]
    async fn test_propagate_contagions() {
        let mut resources = ResourceContext::new();

        // Setup config
        let config = ContagionConfig::default();
        resources.insert(config);

        // Setup topology
        let mut topology = GraphTopology::new();
        topology.add_node(ContagionNode::new("london", NodeType::City, 100000));
        topology.add_node(ContagionNode::new("paris", NodeType::City, 80000));
        topology.add_edge(PropagationEdge::new("london_paris", "london", "paris", 1.0)); // High transmission
        resources.insert(topology);

        // Setup state with one contagion
        let mut state = ContagionState::new();
        let contagion = Contagion::new(
            "c1",
            ContagionContent::Disease {
                severity: DiseaseLevel::Moderate,
                location: "london".to_string(),
            },
            "london",
            0,
        );
        state.spawn_contagion(contagion);
        resources.insert(state);

        let system = ContagionSystem::default();

        // Run propagation multiple times (probabilistic)
        let mut spread_occurred = false;
        for _ in 0..20 {
            let report = system.propagate_contagions(&mut resources).await.unwrap();
            if report.spread_count > 0 {
                spread_occurred = true;

                // Assert spread details are populated
                assert_eq!(report.spread_details.len(), report.spread_count);
                assert!(report
                    .spread_details
                    .iter()
                    .all(|detail| { detail.from_node == "london" && detail.to_node == "paris" }));

                break;
            }
        }

        assert!(spread_occurred);
    }

    #[tokio::test]
    async fn test_decay_contagions() {
        let mut resources = ResourceContext::new();

        // Setup config with short lifetime
        let config = ContagionConfig::default().with_lifetime_turns(5);
        resources.insert(config);

        // Setup empty topology
        let topology = GraphTopology::new();
        resources.insert(topology);

        // Setup state
        let mut state = ContagionState::new();
        let contagion = Contagion::new(
            "c1",
            ContagionContent::Disease {
                severity: DiseaseLevel::Moderate,
                location: "london".to_string(),
            },
            "london",
            0,
        );
        state.spawn_contagion(contagion);
        resources.insert(state);

        let system = ContagionSystem::default();

        // Decay for entire lifetime
        let removed = system.decay_contagions(&mut resources, 5).await.unwrap();

        assert_eq!(removed, 1);

        // Verify state is empty
        let state = resources.get::<ContagionState>().await.unwrap();
        assert_eq!(state.contagion_count(), 0);
    }

    #[tokio::test]
    async fn test_get_node_contagions() {
        let mut resources = ResourceContext::new();

        let mut state = ContagionState::new();
        let contagion = Contagion::new(
            "c1",
            ContagionContent::Disease {
                severity: DiseaseLevel::Moderate,
                location: "london".to_string(),
            },
            "london",
            0,
        );
        state.spawn_contagion(contagion);
        resources.insert(state);

        let system = ContagionSystem::default();

        let contagions = system
            .get_node_contagions(&resources, &"london".to_string())
            .await
            .unwrap();

        assert_eq!(contagions.len(), 1);
    }
}
