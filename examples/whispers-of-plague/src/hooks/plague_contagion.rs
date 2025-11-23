use crate::models::CityMap;
use async_trait::async_trait;
use issun::plugin::contagion::{
    Contagion, ContagionContent, ContagionHook, DiseaseLevel, NodeId, PropagationEdge,
};
use issun::prelude::ResourceContext;

/// Custom hook for plague-specific contagion behavior
#[derive(Clone)]
pub struct PlagueContagionHook;

#[async_trait]
impl ContagionHook for PlagueContagionHook {
    /// Modify transmission rate based on panic levels
    async fn modify_transmission_rate(
        &self,
        base_rate: f32,
        edge: &PropagationEdge,
        _contagion: &Contagion,
    ) -> f32 {
        // Get panic level from source district
        let resources = ResourceContext::new();
        if let Some(city) = resources.try_get::<CityMap>() {
            // Find district matching edge.from
            if let Some(district) = city.districts.iter().find(|d| d.id == edge.from) {
                // Panic increases disease spread
                return base_rate * (1.0 + district.panic_level * 0.5);
            }
        }

        base_rate
    }

    /// Handle spread events (logging, updates)
    async fn on_contagion_spread(
        &self,
        _contagion: &Contagion,
        _from_node: &NodeId,
        _to_node: &NodeId,
    ) {
        // Note: In TUI applications, we should NOT use println! as it corrupts the display.
        // Instead, spread events are logged in GameScene's update() method.
    }

    /// Custom mutation logic for diseases and rumors
    async fn mutate_custom_content(
        &self,
        content: &ContagionContent,
        noise_level: f32,
    ) -> Option<ContagionContent> {
        match content {
            // Disease mutation: Higher noise = more severe mutations
            ContagionContent::Disease { severity, location } => {
                if noise_level > 0.15 {
                    let new_severity = severity.increase();
                    if new_severity != *severity {
                        // Note: Mutation events are logged via GameScene's mutation_count tracking
                        return Some(ContagionContent::Disease {
                            severity: new_severity,
                            location: location.clone(),
                        });
                    }
                }
                None
            }

            // Rumor mutation: Exaggeration increases (telephone game effect)
            ContagionContent::Political { faction, claim } => {
                if noise_level > 0.1 {
                    // Rumors get more exaggerated with each hop
                    let exaggerated_claim = if claim.contains("conspiracy") {
                        format!("{} [EXAGGERATED]", claim)
                    } else if claim.contains("cure") {
                        format!("{} [UNVERIFIED]", claim)
                    } else {
                        format!("{} [RUMOR]", claim)
                    };

                    Some(ContagionContent::Political {
                        faction: faction.clone(),
                        claim: exaggerated_claim,
                    })
                } else {
                    None
                }
            }

            _ => None,
        }
    }
}

impl Default for PlagueContagionHook {
    fn default() -> Self {
        Self
    }
}
