//! Modding Plugin Components and Resources

use bevy::prelude::*;
use std::path::PathBuf;

/// Configuration for modding system
#[derive(Resource, Reflect, Debug, Clone)]
#[reflect(Resource)]
pub struct ModdingConfig {
    /// Path to mods directory (default: "mods")
    pub mods_directory: PathBuf,
}

impl Default for ModdingConfig {
    fn default() -> Self {
        Self {
            mods_directory: PathBuf::from("mods"),
        }
    }
}

impl ModdingConfig {
    /// Create config with custom mods directory
    pub fn with_directory(path: impl Into<PathBuf>) -> Self {
        Self {
            mods_directory: path.into(),
        }
    }
}

/// Resource tracking discovered mod files
#[derive(Resource, Reflect, Debug, Default)]
#[reflect(Resource)]
pub struct DiscoveredMods {
    /// List of .ron file paths found in mods/ directory
    pub ron_files: Vec<PathBuf>,
}

impl DiscoveredMods {
    pub fn new() -> Self {
        Self {
            ron_files: Vec::new(),
        }
    }

    pub fn add_ron_file(&mut self, path: PathBuf) {
        self.ron_files.push(path);
    }

    pub fn clear(&mut self) {
        self.ron_files.clear();
    }
}

/// Resource tracking loaded mod scenes
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct LoadedModScenes {
    /// Map from file path to DynamicScene handle
    #[reflect(ignore)]
    pub scenes: Vec<(PathBuf, Handle<DynamicScene>)>,
    /// Tracks which scenes have been applied to the world
    pub applied: Vec<PathBuf>,
}

impl LoadedModScenes {
    pub fn new() -> Self {
        Self {
            scenes: Vec::new(),
            applied: Vec::new(),
        }
    }

    pub fn add_scene(&mut self, path: PathBuf, handle: Handle<DynamicScene>) {
        self.scenes.push((path, handle));
    }

    pub fn mark_applied(&mut self, path: &PathBuf) {
        if !self.applied.contains(path) {
            self.applied.push(path.clone());
        }
    }

    pub fn is_applied(&self, path: &PathBuf) -> bool {
        self.applied.contains(path)
    }
}
