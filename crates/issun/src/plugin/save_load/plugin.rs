//! Save/Load plugin implementation

use super::hook::{DefaultSaveLoadHook, SaveLoadHook};
use super::system::SaveLoadSystem;
use crate::plugin::{Plugin, PluginBuilder, PluginBuilderExt};
use crate::resources::Resource;
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;

/// Configuration for the SaveLoadPlugin
#[derive(Debug, Clone)]
pub struct SaveLoadConfig {
    /// Directory where save files will be stored
    pub save_directory: PathBuf,
    /// Save file format to use
    pub format: SaveFormat,
    /// Whether to enable auto-save functionality
    pub enable_auto_save: bool,
    /// Auto-save interval in seconds (if auto-save is enabled)
    pub auto_save_interval: u64,
}

impl Resource for SaveLoadConfig {}

impl Default for SaveLoadConfig {
    fn default() -> Self {
        Self {
            save_directory: PathBuf::from("saves"),
            format: SaveFormat::Json,
            enable_auto_save: true,
            auto_save_interval: 300, // 5 minutes
        }
    }
}

/// Supported save file formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveFormat {
    /// JSON format (human-readable, widely compatible)
    Json,
    /// RON format (Rust Object Notation - more compact and Rust-native)
    Ron,
}

/// Built-in save/load plugin for ISSUN
///
/// This plugin provides comprehensive save/load functionality including:
/// - Manual save/load operations
/// - Auto-save functionality
/// - Multiple save formats (JSON, RON)
/// - Customizable save hooks
/// - Save file management (list, delete, metadata)
///
/// # Features
///
/// - **Multiple Formats**: JSON and RON save formats
/// - **Auto-Save**: Configurable automatic saving
/// - **Hook System**: Customize save/load behavior
/// - **File Management**: List, delete, and inspect save files
/// - **Error Handling**: Comprehensive error reporting
/// - **Metadata**: Rich save file metadata including timestamps and versions
///
/// # Example
///
/// ```ignore
/// use issun::builder::GameBuilder;
/// use issun::plugin::save_load::{SaveLoadPlugin, SaveLoadConfig, SaveFormat};
/// use std::path::PathBuf;
///
/// let config = SaveLoadConfig {
///     save_directory: PathBuf::from("my_game_saves"),
///     format: SaveFormat::Ron,
///     enable_auto_save: true,
///     auto_save_interval: 180, // 3 minutes
/// };
///
/// let game = GameBuilder::new()
///     .with_plugin(
///         SaveLoadPlugin::new()
///             .with_config(config)
///     )
///     .build()
///     .await?;
/// ```
///
/// # Custom Hooks
///
/// ```ignore
/// use issun::plugin::save_load::{SaveLoadPlugin, SaveLoadHook};
/// use async_trait::async_trait;
///
/// struct CloudSaveHook;
///
/// #[async_trait]
/// impl SaveLoadHook for CloudSaveHook {
///     async fn after_save(
///         &self,
///         save_data: &SaveData,
///         metadata: &SaveMetadata,
///         resources: &mut ResourceContext,
///     ) {
///         // Upload to cloud storage
///         cloud_backup(save_data).await;
///     }
/// }
///
/// let game = GameBuilder::new()
///     .with_plugin(
///         SaveLoadPlugin::new()
///             .with_hook(CloudSaveHook)
///     )
///     .build()
///     .await?;
/// ```
pub struct SaveLoadPlugin {
    hook: Arc<dyn SaveLoadHook>,
    config: SaveLoadConfig,
}

impl SaveLoadPlugin {
    /// Create a new SaveLoadPlugin with default configuration
    ///
    /// Uses the default hook (no custom behavior) and default config:
    /// - Save directory: "./saves"
    /// - Format: JSON
    /// - Auto-save enabled with 5-minute interval
    pub fn new() -> Self {
        Self {
            hook: Arc::new(DefaultSaveLoadHook),
            config: SaveLoadConfig::default(),
        }
    }

    /// Add a custom hook for save/load behavior
    ///
    /// The hook will be called at various points during save/load operations:
    /// - Before/after save operations
    /// - Before/after load operations
    /// - On operation failures
    /// - During auto-save decisions
    ///
    /// # Arguments
    ///
    /// * `hook` - Implementation of SaveLoadHook trait
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::save_load::{SaveLoadPlugin, SaveLoadHook};
    ///
    /// struct MyHook;
    ///
    /// #[async_trait]
    /// impl SaveLoadHook for MyHook {
    ///     async fn before_save(
    ///         &self,
    ///         save_data: &mut SaveData,
    ///         resources: &mut ResourceContext,
    ///     ) -> bool {
    ///         // Add custom validation logic
    ///         validate_save_data(save_data)
    ///     }
    /// }
    ///
    /// let plugin = SaveLoadPlugin::new().with_hook(MyHook);
    /// ```
    pub fn with_hook<H: SaveLoadHook + 'static>(mut self, hook: H) -> Self {
        self.hook = Arc::new(hook);
        self
    }

    /// Set custom save/load configuration
    ///
    /// # Arguments
    ///
    /// * `config` - SaveLoadConfig with custom settings
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::save_load::{SaveLoadPlugin, SaveLoadConfig, SaveFormat};
    /// use std::path::PathBuf;
    ///
    /// let config = SaveLoadConfig {
    ///     save_directory: PathBuf::from("custom_saves"),
    ///     format: SaveFormat::Ron,
    ///     enable_auto_save: false,
    ///     auto_save_interval: 0,
    /// };
    ///
    /// let plugin = SaveLoadPlugin::new().with_config(config);
    /// ```
    pub fn with_config(mut self, config: SaveLoadConfig) -> Self {
        self.config = config;
        self
    }

    /// Convenience method to set save directory
    ///
    /// # Arguments
    ///
    /// * `directory` - Path where save files will be stored
    ///
    /// # Example
    ///
    /// ```ignore
    /// use std::path::PathBuf;
    ///
    /// let plugin = SaveLoadPlugin::new()
    ///     .with_save_directory(PathBuf::from("game_data/saves"));
    /// ```
    pub fn with_save_directory(mut self, directory: PathBuf) -> Self {
        self.config.save_directory = directory;
        self
    }

    /// Convenience method to set save format
    ///
    /// # Arguments
    ///
    /// * `format` - Save file format to use
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::save_load::{SaveLoadPlugin, SaveFormat};
    ///
    /// let plugin = SaveLoadPlugin::new()
    ///     .with_format(SaveFormat::Ron);
    /// ```
    pub fn with_format(mut self, format: SaveFormat) -> Self {
        self.config.format = format;
        self
    }

    /// Convenience method to configure auto-save
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to enable auto-save
    /// * `interval_seconds` - Auto-save interval in seconds
    ///
    /// # Example
    ///
    /// ```ignore
    /// let plugin = SaveLoadPlugin::new()
    ///     .with_auto_save(true, 120); // Auto-save every 2 minutes
    /// ```
    pub fn with_auto_save(mut self, enabled: bool, interval_seconds: u64) -> Self {
        self.config.enable_auto_save = enabled;
        self.config.auto_save_interval = interval_seconds;
        self
    }
}

impl Default for SaveLoadPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for SaveLoadPlugin {
    fn name(&self) -> &'static str {
        "save_load_plugin"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        // Note: Repository creation is moved to initialize() since constructors are async

        // Register the SaveLoadConfig as a resource for other systems to access
        builder.register_resource(self.config.clone());

        // Register save/load system with hook
        builder.register_system(Box::new(SaveLoadSystem::new(
            self.hook.clone(),
            self.config.clone(),
        )));
    }

    async fn initialize(&mut self) {
        // Create save directory if it doesn't exist
        if let Err(e) = tokio::fs::create_dir_all(&self.config.save_directory).await {
            eprintln!(
                "Warning: Failed to create save directory {:?}: {}",
                self.config.save_directory, e
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::save_load::hook::SaveLoadHook;

    #[test]
    fn test_plugin_creation() {
        let plugin = SaveLoadPlugin::new();
        assert_eq!(plugin.name(), "save_load_plugin");
        assert_eq!(plugin.config.format, SaveFormat::Json);
        assert!(plugin.config.enable_auto_save);
    }

    #[test]
    fn test_plugin_with_config() {
        let config = SaveLoadConfig {
            save_directory: PathBuf::from("test_saves"),
            format: SaveFormat::Ron,
            enable_auto_save: false,
            auto_save_interval: 60,
        };

        let plugin = SaveLoadPlugin::new().with_config(config.clone());
        assert_eq!(plugin.config.save_directory, config.save_directory);
        assert_eq!(plugin.config.format, SaveFormat::Ron);
        assert!(!plugin.config.enable_auto_save);
        assert_eq!(plugin.config.auto_save_interval, 60);
    }

    #[test]
    fn test_plugin_with_hook() {
        struct CustomHook;

        #[async_trait::async_trait]
        impl SaveLoadHook for CustomHook {}

        let plugin = SaveLoadPlugin::new().with_hook(CustomHook);
        assert_eq!(plugin.name(), "save_load_plugin");
    }

    #[test]
    fn test_plugin_convenience_methods() {
        let plugin = SaveLoadPlugin::new()
            .with_save_directory(PathBuf::from("custom"))
            .with_format(SaveFormat::Ron)
            .with_auto_save(false, 42);

        assert_eq!(plugin.config.save_directory, PathBuf::from("custom"));
        assert_eq!(plugin.config.format, SaveFormat::Ron);
        assert!(!plugin.config.enable_auto_save);
        assert_eq!(plugin.config.auto_save_interval, 42);
    }

    #[test]
    fn test_default_config() {
        let config = SaveLoadConfig::default();
        assert_eq!(config.save_directory, PathBuf::from("saves"));
        assert_eq!(config.format, SaveFormat::Json);
        assert!(config.enable_auto_save);
        assert_eq!(config.auto_save_interval, 300);
    }
}
