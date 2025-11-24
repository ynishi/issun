//! Core types for analysis results

use serde::{Deserialize, Serialize};

/// Event subscription information (EventReader<E> usage)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventSubscription {
    /// Name of the struct that reads this event
    pub subscriber: String,
    /// Event type being subscribed to
    pub event_type: String,
    /// Source file path
    pub file_path: String,
    /// Line number where the field is defined
    pub line: usize,
}

/// Event publication information (EventBus::publish<E>() calls)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventPublication {
    /// Name of the function/method that publishes this event
    pub publisher: String,
    /// Event type being published
    pub event_type: String,
    /// Source file path
    pub file_path: String,
    /// Line number where publish is called
    pub line: usize,
}

/// Complete analysis result for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAnalysis {
    /// File path
    pub path: String,
    /// Event subscriptions found in this file
    pub subscriptions: Vec<EventSubscription>,
    /// Event publications found in this file
    pub publications: Vec<EventPublication>,
}

/// System information extracted from code
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemInfo {
    /// System struct name
    pub name: String,
    /// Module path (e.g., "plugin::combat::system")
    pub module_path: String,
    /// Source file path
    pub file_path: String,
    /// Event types this system subscribes to
    pub subscribes: Vec<String>,
    /// Event types this system publishes
    pub publishes: Vec<String>,
    /// Hook traits used by this system
    pub hooks: Vec<String>,
    /// State types accessed by this system
    pub states: Vec<String>,
}

/// Hook method category based on naming convention
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HookCategory {
    /// Notification hooks: on_* methods (void return)
    Notification,
    /// Validation hooks: can_* methods (Result return)
    Validation,
    /// Lifecycle hooks: before_*/after_* methods
    Lifecycle,
    /// Calculation hooks: calculate_* methods
    Calculation,
    /// Generation hooks: generate_* methods
    Generation,
    /// Other/Unknown category
    Other,
}

/// Hook method information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookMethod {
    /// Method name
    pub name: String,
    /// Method category
    pub category: HookCategory,
    /// Parameter types (simplified representation)
    pub params: Vec<String>,
    /// Return type (simplified representation)
    pub return_type: String,
    /// Whether this method has a default implementation
    pub has_default_impl: bool,
}

/// Hook trait information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookInfo {
    /// Trait name
    pub trait_name: String,
    /// Module path
    pub module_path: String,
    /// Source file path
    pub file_path: String,
    /// Methods defined in this trait
    pub methods: Vec<HookMethod>,
}

/// Hook call site information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookCall {
    /// Hook trait being called
    pub hook_trait: String,
    /// Method name being called
    pub method_name: String,
    /// Caller (function/method name)
    pub caller: String,
    /// Source file path
    pub file_path: String,
    /// Line number (placeholder: 0)
    pub line: usize,
}

/// Plugin information inferred from directory structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Plugin name (e.g., "combat")
    pub name: String,
    /// Plugin directory path
    pub path: String,
    /// System implementation (if found)
    pub system: Option<SystemInfo>,
    /// Hook trait names defined in this plugin
    pub hooks: Vec<String>,
    /// Event types defined in this plugin
    pub events: Vec<String>,
    /// Detailed hook information (if analyzed)
    pub hook_details: Vec<HookInfo>,
}

/// Complete analysis result for a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// All analyzed files
    pub files: Vec<FileAnalysis>,
    /// Detected systems
    pub systems: Vec<SystemInfo>,
    /// Detected plugins
    pub plugins: Vec<PluginInfo>,
}

impl AnalysisResult {
    /// Create a new empty analysis result
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            systems: Vec::new(),
            plugins: Vec::new(),
        }
    }

    /// Add a file analysis result
    pub fn add_file(&mut self, analysis: FileAnalysis) {
        self.files.push(analysis);
    }

    /// Add a system
    pub fn add_system(&mut self, system: SystemInfo) {
        self.systems.push(system);
    }

    /// Add a plugin
    pub fn add_plugin(&mut self, plugin: PluginInfo) {
        self.plugins.push(plugin);
    }

    /// Get all event subscriptions across all files
    pub fn all_subscriptions(&self) -> Vec<&EventSubscription> {
        self.files
            .iter()
            .flat_map(|f| f.subscriptions.iter())
            .collect()
    }

    /// Get all event publications across all files
    pub fn all_publications(&self) -> Vec<&EventPublication> {
        self.files
            .iter()
            .flat_map(|f| f.publications.iter())
            .collect()
    }

    /// Get all unique event types
    pub fn event_types(&self) -> Vec<String> {
        let mut types: Vec<String> = self
            .all_subscriptions()
            .iter()
            .map(|s| s.event_type.clone())
            .chain(self.all_publications().iter().map(|p| p.event_type.clone()))
            .collect();

        types.sort();
        types.dedup();
        types
    }
}

impl Default for AnalysisResult {
    fn default() -> Self {
        Self::new()
    }
}
