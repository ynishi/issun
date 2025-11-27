//! ISSUN Bevy Plugins
//!
//! Unit-testable game logic plugins for Bevy ECS.

use bevy::prelude::*;

pub mod plugins;

/// ISSUN common system execution order (Phase 1: Simple 4-stage pipeline)
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum IssunSet {
    /// Input processing (user input, network receive)
    Input,

    /// Main logic (combat, economy, AI)
    /// ⚠️ Phase 1: Most processing goes here
    Logic,

    /// Post-logic processing (death checks, cleanup)
    PostLogic,

    /// Visual sync (TUI updates, network send)
    Visual,
}

/// Marker resource indicating IssunCorePlugin is installed
///
/// Used by other issun-bevy plugins to verify IssunCorePlugin dependency.
#[derive(Resource)]
pub struct IssunCorePluginMarker;

/// ISSUN Core Plugin (configures SystemSet ordering once)
///
/// ## Usage Example
///
/// ```no_run
/// use bevy::prelude::*;
/// use issun_bevy::IssunCorePlugin;
///
/// App::new()
///     .add_plugins(IssunCorePlugin)  // ← Add first
///     .run();
/// ```
pub struct IssunCorePlugin;

impl Plugin for IssunCorePlugin {
    fn build(&self, app: &mut App) {
        // Register marker resource for dependency checking
        app.insert_resource(IssunCorePluginMarker);

        // Configure SystemSet ordering
        app.configure_sets(
            Update,
            (
                IssunSet::Input,
                IssunSet::Logic,
                IssunSet::PostLogic,
                IssunSet::Visual,
            )
                .chain(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issun_core_plugin() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(IssunCorePlugin);

        // Verify SystemSet configuration
        app.update();
    }

    #[test]
    fn test_system_set_ordering() {
        use std::sync::{Arc, Mutex};

        let execution_order = Arc::new(Mutex::new(Vec::new()));

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(IssunCorePlugin);

        // Add systems for each SystemSet
        let order1 = execution_order.clone();
        app.add_systems(
            Update,
            (move || {
                order1.lock().unwrap().push("Input");
            })
            .in_set(IssunSet::Input),
        );

        let order2 = execution_order.clone();
        app.add_systems(
            Update,
            (move || {
                order2.lock().unwrap().push("Logic");
            })
            .in_set(IssunSet::Logic),
        );

        let order3 = execution_order.clone();
        app.add_systems(
            Update,
            (move || {
                order3.lock().unwrap().push("PostLogic");
            })
            .in_set(IssunSet::PostLogic),
        );

        let order4 = execution_order.clone();
        app.add_systems(
            Update,
            (move || {
                order4.lock().unwrap().push("Visual");
            })
            .in_set(IssunSet::Visual),
        );

        // Execute
        app.update();

        // Verify execution order
        let order = execution_order.lock().unwrap();
        assert_eq!(
            *order,
            vec!["Input", "Logic", "PostLogic", "Visual"],
            "SystemSet execution order is incorrect"
        );
    }
}
