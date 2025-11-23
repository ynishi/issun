use issun::plugin::contagion::{ContagionNode, GraphTopology, NodeType, PropagationEdge};

/// Build graph topology for city districts
pub fn build_city_topology() -> GraphTopology {
    let mut topology = GraphTopology::new();

    // Add district nodes
    topology
        .add_node(
            ContagionNode::new("downtown", NodeType::City, 100000)
                .with_resistance(0.1), // Low resistance (dense population)
        )
        .add_node(
            ContagionNode::new("industrial", NodeType::Custom("Industrial".into()), 80000)
                .with_resistance(0.2),
        )
        .add_node(
            ContagionNode::new("residential", NodeType::City, 150000)
                .with_resistance(0.15),
        )
        .add_node(
            ContagionNode::new("suburbs", NodeType::Village, 120000)
                .with_resistance(0.3), // Higher resistance (spread out)
        )
        .add_node(
            ContagionNode::new("harbor", NodeType::Custom("Harbor".into()), 90000)
                .with_resistance(0.05), // Very low resistance (high traffic)
        );

    // Add edges (population flow routes)
    // Downtown connections (central hub)
    topology
        .add_edge(
            PropagationEdge::new("downtown_industrial", "downtown", "industrial", 0.6)
                .with_noise(0.1),
        )
        .add_edge(
            PropagationEdge::new("downtown_residential", "downtown", "residential", 0.7)
                .with_noise(0.05),
        )
        .add_edge(
            PropagationEdge::new("downtown_harbor", "downtown", "harbor", 0.5)
                .with_noise(0.15),
        );

    // Industrial connections
    topology
        .add_edge(
            PropagationEdge::new("industrial_downtown", "industrial", "downtown", 0.6)
                .with_noise(0.1),
        )
        .add_edge(
            PropagationEdge::new("industrial_residential", "industrial", "residential", 0.4)
                .with_noise(0.08),
        );

    // Residential connections
    topology
        .add_edge(
            PropagationEdge::new("residential_downtown", "residential", "downtown", 0.7)
                .with_noise(0.05),
        )
        .add_edge(
            PropagationEdge::new("residential_suburbs", "residential", "suburbs", 0.8)
                .with_noise(0.03),
        );

    // Suburbs connections
    topology
        .add_edge(
            PropagationEdge::new("suburbs_residential", "suburbs", "residential", 0.8)
                .with_noise(0.03),
        )
        .add_edge(
            PropagationEdge::new("suburbs_harbor", "suburbs", "harbor", 0.3)
                .with_noise(0.12),
        );

    // Harbor connections (high traffic)
    topology
        .add_edge(
            PropagationEdge::new("harbor_downtown", "harbor", "downtown", 0.5)
                .with_noise(0.15),
        )
        .add_edge(
            PropagationEdge::new("harbor_industrial", "harbor", "industrial", 0.4)
                .with_noise(0.2),
        );

    topology
}
