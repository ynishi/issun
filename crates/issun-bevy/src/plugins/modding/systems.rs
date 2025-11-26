//! Modding Plugin Systems

use bevy::prelude::*;
use std::fs;
use std::path::Path;

use super::components::{DiscoveredMods, LoadedModScenes};

/// Discover mods in the mods/ directory
///
/// Scans for .ron files and registers them in DiscoveredMods resource.
/// If mods/ directory doesn't exist, silently skips (no error).
pub fn discover_mods(mut discovered: ResMut<DiscoveredMods>) {
    let mods_dir = Path::new("mods");

    // Clear previous discoveries
    discovered.clear();

    // If directory doesn't exist, just return (no error)
    if !mods_dir.exists() {
        info!("mods/ directory not found, skipping mod discovery");
        return;
    }

    // Read directory entries
    let entries = match fs::read_dir(mods_dir) {
        Ok(entries) => entries,
        Err(e) => {
            warn!("Failed to read mods/ directory: {}", e);
            return;
        }
    };

    // Find all .ron files
    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "ron" {
                    info!("Discovered mod file: {:?}", path);
                    discovered.add_ron_file(path);
                }
            }
        }
    }

    info!(
        "Mod discovery complete: {} files found",
        discovered.ron_files.len()
    );
}

/// Load mod scenes from discovered .ron files
///
/// Loads DynamicScene assets using AssetServer for discovered mod files.
/// If AssetServer is not available (e.g., in minimal tests), this system does nothing.
pub fn load_mod_scenes(
    discovered: Res<DiscoveredMods>,
    mut loaded: ResMut<LoadedModScenes>,
    asset_server: Option<Res<AssetServer>>,
) {
    // If AssetServer is not available, skip (e.g., in minimal tests)
    let Some(asset_server) = asset_server else {
        return;
    };

    for path in &discovered.ron_files {
        // Skip if already loaded
        if loaded.scenes.iter().any(|(p, _)| p == path) {
            continue;
        }

        // Load DynamicScene asset
        let handle: Handle<DynamicScene> = asset_server.load(path.clone());
        info!("Loading mod scene: {:?}", path);

        loaded.add_scene(path.clone(), handle);
    }
}

/// Apply loaded mod scenes to the world
///
/// Spawns entities from loaded DynamicScenes into the world.
/// If Assets<DynamicScene> is not available (e.g., in minimal tests), this system does nothing.
pub fn apply_mod_scenes(
    mut loaded: ResMut<LoadedModScenes>,
    scenes: Option<Res<Assets<DynamicScene>>>,
    mut commands: Commands,
) {
    // If Assets<DynamicScene> is not available, skip (e.g., in minimal tests)
    let Some(scenes) = scenes else {
        return;
    };

    // Collect paths to mark as applied (to avoid borrow checker issues)
    let mut to_apply = Vec::new();

    for (path, handle) in &loaded.scenes {
        // Skip if already applied
        if loaded.is_applied(path) {
            continue;
        }

        // Check if scene is loaded
        if scenes.get(handle).is_some() {
            info!("Applying mod scene: {:?}", path);

            // Spawn scene into world
            commands.spawn(DynamicSceneRoot(handle.clone()));

            to_apply.push(path.clone());
        }
    }

    // Mark all applied scenes
    for path in to_apply {
        loaded.mark_applied(&path);
    }
}
