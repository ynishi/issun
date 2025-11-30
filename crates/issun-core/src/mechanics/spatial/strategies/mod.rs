//! Concrete strategy implementations for spatial policies.

mod euclidean_distance;
mod fixed_distance;
mod graph_topology;
mod grid_topology;
mod manhattan_distance;

pub use euclidean_distance::EuclideanDistance;
pub use fixed_distance::FixedDistance;
pub use graph_topology::GraphTopology;
pub use grid_topology::GridTopology;
pub use manhattan_distance::ManhattanDistance;
