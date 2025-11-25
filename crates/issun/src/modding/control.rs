//! Plugin control primitives
//!
//! Defines the API for MODs to control ISSUN plugins at runtime.

use serde::{Deserialize, Serialize};

/// Actions that can be performed on plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PluginAction {
    /// Enable a plugin
    Enable,

    /// Disable a plugin
    Disable,

    /// Set plugin parameter
    SetParameter {
        key: String,
        value: serde_json::Value,
    },

    /// Trigger a custom hook
    TriggerHook {
        hook_name: String,
        data: serde_json::Value,
    },
}

/// Plugin control command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginControl {
    pub plugin_name: String,
    pub action: PluginAction,
}

impl PluginControl {
    /// Create a control command to enable a plugin
    pub fn enable(plugin_name: impl Into<String>) -> Self {
        Self {
            plugin_name: plugin_name.into(),
            action: PluginAction::Enable,
        }
    }

    /// Create a control command to disable a plugin
    pub fn disable(plugin_name: impl Into<String>) -> Self {
        Self {
            plugin_name: plugin_name.into(),
            action: PluginAction::Disable,
        }
    }

    /// Create a control command to set a parameter
    pub fn set_param(
        plugin_name: impl Into<String>,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        Self {
            plugin_name: plugin_name.into(),
            action: PluginAction::SetParameter {
                key: key.into(),
                value: value.into(),
            },
        }
    }

    /// Create a control command to trigger a hook
    pub fn trigger_hook(
        plugin_name: impl Into<String>,
        hook_name: impl Into<String>,
        data: serde_json::Value,
    ) -> Self {
        Self {
            plugin_name: plugin_name.into(),
            action: PluginAction::TriggerHook {
                hook_name: hook_name.into(),
                data,
            },
        }
    }
}
