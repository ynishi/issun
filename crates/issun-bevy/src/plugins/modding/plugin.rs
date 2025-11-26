//! Modding Plugin
//!
//! Provides asset loading and hot-reload for data mods.

use bevy::prelude::*;

use crate::IssunSet;

use super::components::{DiscoveredMods, LoadedModScenes};
use super::systems::{apply_mod_scenes, discover_mods, load_mod_scenes};

/// Plugin for modding system support
pub struct ModdingPlugin;

impl Plugin for ModdingPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register types
            .register_type::<DiscoveredMods>()
            .register_type::<LoadedModScenes>()
            // Initialize resources
            .init_resource::<DiscoveredMods>()
            .init_resource::<LoadedModScenes>()
            // Add systems
            .add_systems(Startup, discover_mods)
            .add_systems(
                Update,
                (load_mod_scenes, apply_mod_scenes)
                    .chain()
                    .in_set(IssunSet::Logic),
            );
    }
}
