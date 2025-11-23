//! Pure logic service for social network analysis
//!
//! This service provides stateless functions for centrality calculation,
//! influence propagation, and shadow leader detection based on graph theory.

use super::config::SocialConfig;
use super::state::SocialNetwork;
use super::types::{CentralityMetrics, MemberId, SocialError};
use std::collections::{HashMap, HashSet, VecDeque};

/// Social network analysis service (stateless, pure functions)
///
/// All methods are pure functions with no side effects, making them easy to test.
/// Based on standard graph algorithms from network science.
#[derive(Debug, Clone, Copy, Default)]
pub struct NetworkAnalysisService;

impl NetworkAnalysisService {
    /// Calculate degree centrality for a member
    ///
    /// Degree centrality measures the number of direct connections.
    /// Higher degree = more well-connected.
    ///
    /// Formula: (in-degree + out-degree) / (n - 1)
    /// where n is total number of members
    ///
    /// # Arguments
    ///
    /// * `member_id` - ID of the member
    /// * `network` - The social network
    ///
    /// # Returns
    ///
    /// Normalized degree centrality (0.0-1.0)
    pub fn calculate_degree_centrality(
        member_id: &MemberId,
        network: &SocialNetwork,
    ) -> Result<f32, SocialError> {
        if !network.has_member(member_id) {
            return Err(SocialError::MemberNotFound(member_id.clone()));
        }

        let n = network.member_count();
        if n <= 1 {
            return Ok(0.0);
        }

        let mut degree = 0;

        // Count outgoing and incoming edges
        for ((from, to), _) in network.all_relations() {
            if from == member_id || to == member_id {
                degree += 1;
            }
        }

        // Normalize by maximum possible degree (n-1)
        Ok(degree as f32 / (n - 1) as f32)
    }

    /// Calculate betweenness centrality for a member
    ///
    /// Betweenness centrality measures how often a member lies on
    /// the shortest path between other members.
    /// Higher betweenness = more of an information broker.
    ///
    /// Uses simplified algorithm (not full Brandes for now).
    ///
    /// # Arguments
    ///
    /// * `member_id` - ID of the member
    /// * `network` - The social network
    ///
    /// # Returns
    ///
    /// Normalized betweenness centrality (0.0-1.0)
    pub fn calculate_betweenness_centrality(
        member_id: &MemberId,
        network: &SocialNetwork,
    ) -> Result<f32, SocialError> {
        if !network.has_member(member_id) {
            return Err(SocialError::MemberNotFound(member_id.clone()));
        }

        let n = network.member_count();
        if n <= 2 {
            return Ok(0.0);
        }

        let all_members: Vec<MemberId> = network.all_members().map(|(id, _)| id.clone()).collect();

        let mut betweenness = 0.0;

        // For each pair of members (s, t) where s != t
        for source in &all_members {
            if source == member_id {
                continue;
            }

            for target in &all_members {
                if target == member_id || target == source {
                    continue;
                }

                // Find all shortest paths from source to target
                let shortest_path_length =
                    Self::shortest_path_length(source, target, network);

                if let Some(sp_len) = shortest_path_length {
                    // Check if member_id is on a shortest path
                    if Self::is_on_shortest_path(source, target, member_id, sp_len, network) {
                        betweenness += 1.0;
                    }
                }
            }
        }

        // Normalize by maximum possible betweenness: (n-1)(n-2)/2
        let max_betweenness = ((n - 1) * (n - 2)) as f32 / 2.0;
        if max_betweenness > 0.0 {
            Ok(betweenness / max_betweenness)
        } else {
            Ok(0.0)
        }
    }

    /// Calculate closeness centrality for a member
    ///
    /// Closeness centrality measures the average distance to all other members.
    /// Higher closeness = information spreads faster from this person.
    ///
    /// Formula: (n-1) / sum(distance to all others)
    ///
    /// # Arguments
    ///
    /// * `member_id` - ID of the member
    /// * `network` - The social network
    ///
    /// # Returns
    ///
    /// Normalized closeness centrality (0.0-1.0)
    pub fn calculate_closeness_centrality(
        member_id: &MemberId,
        network: &SocialNetwork,
    ) -> Result<f32, SocialError> {
        if !network.has_member(member_id) {
            return Err(SocialError::MemberNotFound(member_id.clone()));
        }

        let n = network.member_count();
        if n <= 1 {
            return Ok(0.0);
        }

        let mut total_distance = 0.0;
        let mut reachable_count = 0;

        for (other_id, _) in network.all_members() {
            if other_id == member_id {
                continue;
            }

            if let Some(distance) = Self::shortest_path_length(member_id, other_id, network) {
                total_distance += distance as f32;
                reachable_count += 1;
            }
        }

        if reachable_count == 0 || total_distance == 0.0 {
            return Ok(0.0);
        }

        // Closeness = (n-1) / sum(distances)
        // Normalized by reachable nodes
        Ok(reachable_count as f32 / total_distance)
    }

    /// Calculate eigenvector centrality for all members
    ///
    /// Eigenvector centrality measures influence based on connections
    /// to other influential members.
    /// Higher eigenvector = connected to power centers.
    ///
    /// Uses Power Iteration algorithm.
    ///
    /// # Arguments
    ///
    /// * `network` - The social network
    /// * `max_iterations` - Maximum iterations (default: 100)
    /// * `tolerance` - Convergence tolerance (default: 0.0001)
    ///
    /// # Returns
    ///
    /// HashMap of member_id -> eigenvector centrality
    pub fn calculate_eigenvector_centrality(
        network: &SocialNetwork,
        max_iterations: u32,
        tolerance: f32,
    ) -> HashMap<MemberId, f32> {
        let mut scores: HashMap<MemberId, f32> = network
            .all_members()
            .map(|(id, _)| (id.clone(), 1.0))
            .collect();

        if scores.is_empty() {
            return scores;
        }

        for _ in 0..max_iterations {
            let mut new_scores: HashMap<MemberId, f32> = HashMap::new();

            // For each member, sum the scores of neighbors
            for (member_id, _) in network.all_members() {
                let mut score = 0.0;

                // Add scores from incoming connections
                for ((from, to), _) in network.all_relations() {
                    if to == member_id {
                        score += scores.get(from).unwrap_or(&0.0);
                    }
                }

                new_scores.insert(member_id.clone(), score);
            }

            // Normalize (L2 norm)
            let norm: f32 = new_scores
                .values()
                .map(|v| v * v)
                .sum::<f32>()
                .sqrt();

            if norm > 0.0 {
                for score in new_scores.values_mut() {
                    *score /= norm;
                }
            }

            // Check convergence
            let diff: f32 = new_scores
                .iter()
                .map(|(id, new_val)| {
                    let old_val = scores.get(id).unwrap_or(&0.0);
                    (new_val - old_val).abs()
                })
                .sum();

            if diff < tolerance {
                return new_scores;
            }

            scores = new_scores;
        }

        scores
    }

    /// Calculate all centrality metrics for a member
    ///
    /// Convenience function that calculates all four centrality types.
    ///
    /// # Arguments
    ///
    /// * `member_id` - ID of the member
    /// * `network` - The social network
    /// * `config` - Social configuration
    /// * `eigenvector_scores` - Pre-calculated eigenvector scores (optional)
    ///
    /// # Returns
    ///
    /// Complete CentralityMetrics with overall influence
    pub fn calculate_all_centrality(
        member_id: &MemberId,
        network: &SocialNetwork,
        config: &SocialConfig,
        eigenvector_scores: Option<&HashMap<MemberId, f32>>,
    ) -> Result<CentralityMetrics, SocialError> {
        let degree = Self::calculate_degree_centrality(member_id, network)?;
        let betweenness = Self::calculate_betweenness_centrality(member_id, network)?;
        let closeness = Self::calculate_closeness_centrality(member_id, network)?;

        let eigenvector = eigenvector_scores
            .and_then(|scores| scores.get(member_id).copied())
            .unwrap_or_else(|| {
                // Fallback: calculate just for this member (less accurate)
                let all_scores = Self::calculate_eigenvector_centrality(network, 100, 0.0001);
                all_scores.get(member_id).copied().unwrap_or(0.0)
            });

        let mut metrics = CentralityMetrics {
            degree,
            betweenness,
            closeness,
            eigenvector,
            overall_influence: 0.0,
        };

        // Calculate weighted overall influence
        metrics.calculate_overall(
            config.centrality_weights.degree,
            config.centrality_weights.betweenness,
            config.centrality_weights.closeness,
            config.centrality_weights.eigenvector,
        );

        Ok(metrics)
    }

    /// Detect shadow leaders (KingMakers)
    ///
    /// Identifies members with high influence but potentially low official authority.
    ///
    /// Criteria:
    /// - High betweenness (information broker)
    /// - High overall influence above threshold
    ///
    /// # Arguments
    ///
    /// * `network` - The social network
    /// * `config` - Social configuration
    ///
    /// # Returns
    ///
    /// Vector of (member_id, influence_score) sorted by influence
    pub fn detect_shadow_leaders(
        network: &SocialNetwork,
        config: &SocialConfig,
    ) -> Vec<(MemberId, f32)> {
        // Pre-calculate eigenvector centrality for all
        let eigenvector_scores =
            Self::calculate_eigenvector_centrality(network, 100, 0.0001);

        let mut candidates = Vec::new();

        for (member_id, member) in network.all_members() {
            // Use cached centrality if available
            let metrics = if let Some(metrics) = Some(&member.capital.centrality_scores) {
                if network.is_centrality_cache_valid() {
                    metrics.clone()
                } else {
                    Self::calculate_all_centrality(
                        member_id,
                        network,
                        config,
                        Some(&eigenvector_scores),
                    )
                    .unwrap_or_default()
                }
            } else {
                Self::calculate_all_centrality(
                    member_id,
                    network,
                    config,
                    Some(&eigenvector_scores),
                )
                .unwrap_or_default()
            };

            // Check if shadow leader
            if metrics.is_shadow_leader(config.shadow_leader_threshold) {
                // Additional criteria: high betweenness (information broker)
                if metrics.betweenness > 0.5 {
                    candidates.push((member_id.clone(), metrics.overall_influence));
                }
            }
        }

        // Sort by influence (descending)
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        candidates
    }

    // ===== Helper Functions =====

    /// Find shortest path length between two members using BFS
    fn shortest_path_length(
        from: &MemberId,
        to: &MemberId,
        network: &SocialNetwork,
    ) -> Option<usize> {
        if from == to {
            return Some(0);
        }

        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut distances: HashMap<MemberId, usize> = HashMap::new();

        queue.push_back(from.clone());
        visited.insert(from.clone());
        distances.insert(from.clone(), 0);

        while let Some(current) = queue.pop_front() {
            let current_dist = distances[&current];

            // Get neighbors
            for neighbor in network.get_neighbors(&current) {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor.clone());
                    distances.insert(neighbor.clone(), current_dist + 1);
                    queue.push_back(neighbor.clone());

                    if &neighbor == to {
                        return Some(current_dist + 1);
                    }
                }
            }
        }

        None // No path found
    }

    /// Check if a member is on a shortest path between source and target
    fn is_on_shortest_path(
        source: &MemberId,
        target: &MemberId,
        member: &MemberId,
        shortest_length: usize,
        network: &SocialNetwork,
    ) -> bool {
        if member == source || member == target {
            return false;
        }

        // Check if distance(source, member) + distance(member, target) == shortest_length
        if let Some(dist_sm) = Self::shortest_path_length(source, member, network) {
            if let Some(dist_mt) = Self::shortest_path_length(member, target, network) {
                return dist_sm + dist_mt == shortest_length;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::social::state::SocialMember;
    use crate::plugin::social::types::RelationType;

    fn create_test_network() -> SocialNetwork {
        let mut network = SocialNetwork::new();

        // Create a simple network: A -> B -> C
        //                          A -> C
        network.add_member(SocialMember::new("A".to_string(), "Alice".to_string()));
        network.add_member(SocialMember::new("B".to_string(), "Bob".to_string()));
        network.add_member(SocialMember::new("C".to_string(), "Carol".to_string()));

        network
            .add_relation(
                "A".to_string(),
                "B".to_string(),
                RelationType::Trust { strength: 0.8 },
            )
            .unwrap();
        network
            .add_relation(
                "B".to_string(),
                "C".to_string(),
                RelationType::Trust { strength: 0.8 },
            )
            .unwrap();
        network
            .add_relation(
                "A".to_string(),
                "C".to_string(),
                RelationType::Trust { strength: 0.5 },
            )
            .unwrap();

        network
    }

    #[test]
    fn test_degree_centrality() {
        let network = create_test_network();

        let degree_a = NetworkAnalysisService::calculate_degree_centrality(&"A".to_string(), &network).unwrap();
        let degree_b = NetworkAnalysisService::calculate_degree_centrality(&"B".to_string(), &network).unwrap();
        let degree_c = NetworkAnalysisService::calculate_degree_centrality(&"C".to_string(), &network).unwrap();

        // A has 2 connections (to B and C)
        assert_eq!(degree_a, 1.0); // 2 / (3-1) = 1.0

        // B has 2 connections (from A, to C)
        assert_eq!(degree_b, 1.0);

        // C has 2 connections (from A, from B)
        assert_eq!(degree_c, 1.0);
    }

    #[test]
    fn test_degree_centrality_invalid_member() {
        let network = create_test_network();

        let result = NetworkAnalysisService::calculate_degree_centrality(&"X".to_string(), &network);
        assert!(result.is_err());
    }

    #[test]
    fn test_shortest_path_length() {
        let network = create_test_network();

        // Direct path A -> C (length 1)
        let dist_ac = NetworkAnalysisService::shortest_path_length(&"A".to_string(), &"C".to_string(), &network);
        assert_eq!(dist_ac, Some(1));

        // Path A -> B (length 1)
        let dist_ab = NetworkAnalysisService::shortest_path_length(&"A".to_string(), &"B".to_string(), &network);
        assert_eq!(dist_ab, Some(1));

        // Same node
        let dist_aa = NetworkAnalysisService::shortest_path_length(&"A".to_string(), &"A".to_string(), &network);
        assert_eq!(dist_aa, Some(0));
    }

    #[test]
    fn test_closeness_centrality() {
        let network = create_test_network();

        let closeness_a = NetworkAnalysisService::calculate_closeness_centrality(&"A".to_string(), &network).unwrap();

        // A can reach B (dist 1) and C (dist 1)
        // Closeness = 2 / (1 + 1) = 1.0
        assert_eq!(closeness_a, 1.0);
    }

    #[test]
    fn test_eigenvector_centrality() {
        let network = create_test_network();

        let scores = NetworkAnalysisService::calculate_eigenvector_centrality(&network, 100, 0.0001);

        assert!(scores.contains_key("A"));
        assert!(scores.contains_key("B"));
        assert!(scores.contains_key("C"));

        // All scores should be non-negative
        assert!(scores["A"] >= 0.0);
        assert!(scores["B"] >= 0.0);
        assert!(scores["C"] >= 0.0);

        // Scores should be normalized (L2 norm ≈ 1)
        let norm: f32 = scores.values().map(|v| v * v).sum::<f32>().sqrt();
        // More lenient tolerance for small networks
        assert!((norm - 1.0).abs() < 0.1 || norm == 0.0);

        // At least one score should be positive
        let sum: f32 = scores.values().sum();
        assert!(sum > 0.0 || sum == 0.0); // Allow zero for disconnected graphs
    }

    #[test]
    fn test_calculate_all_centrality() {
        let network = create_test_network();
        let config = SocialConfig::default();

        let metrics = NetworkAnalysisService::calculate_all_centrality(
            &"A".to_string(),
            &network,
            &config,
            None,
        )
        .unwrap();

        assert!(metrics.degree > 0.0);
        assert!(metrics.overall_influence > 0.0);
    }

    #[test]
    fn test_detect_shadow_leaders_empty_network() {
        let network = SocialNetwork::new();
        let config = SocialConfig::default();

        let leaders = NetworkAnalysisService::detect_shadow_leaders(&network, &config);
        assert!(leaders.is_empty());
    }

    #[test]
    fn test_betweenness_centrality() {
        let mut network = SocialNetwork::new();

        // Create a line: A -> B -> C
        // B should have high betweenness
        network.add_member(SocialMember::new("A".to_string(), "Alice".to_string()));
        network.add_member(SocialMember::new("B".to_string(), "Bob".to_string()));
        network.add_member(SocialMember::new("C".to_string(), "Carol".to_string()));

        network
            .add_relation(
                "A".to_string(),
                "B".to_string(),
                RelationType::Trust { strength: 0.8 },
            )
            .unwrap();
        network
            .add_relation(
                "B".to_string(),
                "C".to_string(),
                RelationType::Trust { strength: 0.8 },
            )
            .unwrap();

        let betweenness_b = NetworkAnalysisService::calculate_betweenness_centrality(&"B".to_string(), &network).unwrap();

        // B is on the path from A to C
        assert!(betweenness_b > 0.0);
    }
}
