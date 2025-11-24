//! CLI configuration

use crate::error::{CliError, Result};
use std::path::{Path, PathBuf};

/// CLI configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Project root directory
    pub project_root: PathBuf,
    /// Plugin directory (relative to project root)
    pub plugin_dir: PathBuf,
    /// Output directory for generated files
    pub output_dir: PathBuf,
}

impl Config {
    /// Create a new configuration with defaults
    pub fn new() -> Self {
        Self {
            project_root: PathBuf::from("."),
            plugin_dir: PathBuf::from("crates/issun/src/plugin"),
            output_dir: PathBuf::from("."),
        }
    }

    /// Set project root directory
    pub fn with_project_root<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.project_root = path.as_ref().to_path_buf();
        self
    }

    /// Set plugin directory
    pub fn with_plugin_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.plugin_dir = path.as_ref().to_path_buf();
        self
    }

    /// Set output directory
    pub fn with_output_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.output_dir = path.as_ref().to_path_buf();
        self
    }

    /// Get absolute path to plugin directory
    pub fn plugin_dir_absolute(&self) -> PathBuf {
        if self.plugin_dir.is_absolute() {
            self.plugin_dir.clone()
        } else {
            self.project_root.join(&self.plugin_dir)
        }
    }

    /// Get absolute path to output directory
    pub fn output_dir_absolute(&self) -> PathBuf {
        if self.output_dir.is_absolute() {
            self.output_dir.clone()
        } else {
            self.project_root.join(&self.output_dir)
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        let plugin_dir = self.plugin_dir_absolute();
        if !plugin_dir.exists() {
            return Err(CliError::ConfigError(format!(
                "Plugin directory does not exist: {}",
                plugin_dir.display()
            )));
        }

        if !plugin_dir.is_dir() {
            return Err(CliError::ConfigError(format!(
                "Plugin directory is not a directory: {}",
                plugin_dir.display()
            )));
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
