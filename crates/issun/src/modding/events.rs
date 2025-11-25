//! Event types for MOD system communication
//!
//! These events enable decoupled communication between MOD scripts,
//! the MOD system, and ISSUN plugins.

use crate::event::Event;
use crate::modding::{ModHandle, PluginControl};
use std::path::PathBuf;

/// Dynamic event from MOD scripts
///
/// This event carries arbitrary event data from MOD scripts.
/// MODs can publish and subscribe to these events using string event types.
///
/// # Example
/// ```ignore
/// // In MOD script:
/// publish_event("CustomEvent", #{
///     message: "Hello",
///     value: 42
/// });
///
/// subscribe_event("CustomEvent", |event| {
///     log("Received: " + event.message);
/// });
/// ```
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DynamicEvent {
    /// Event type identifier (e.g., "PlayerDamaged", "CustomEvent")
    pub event_type: String,
    /// Event data as JSON
    pub data: serde_json::Value,
}

impl Event for DynamicEvent {}

/// Request to load a MOD from a file path
///
/// Published by user code to trigger MOD loading.
/// Consumed by `ModLoadSystem`.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ModLoadRequested {
    pub path: PathBuf,
}

impl Event for ModLoadRequested {}

/// MOD successfully loaded
///
/// Published by `ModLoadSystem` after successful load.
/// Can be consumed by other systems to react to new MODs.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ModLoadedEvent {
    pub handle: ModHandle,
}

impl Event for ModLoadedEvent {}

/// MOD failed to load
///
/// Published by `ModLoadSystem` when load fails.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ModLoadFailedEvent {
    pub path: PathBuf,
    pub error: String,
}

impl Event for ModLoadFailedEvent {}

/// Request to unload a MOD
///
/// Published by user code to trigger MOD unloading.
/// Consumed by `ModLoadSystem`.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ModUnloadRequested {
    pub mod_id: String,
}

impl Event for ModUnloadRequested {}

/// MOD successfully unloaded
///
/// Published by `ModLoadSystem` after successful unload.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ModUnloadedEvent {
    pub mod_id: String,
}

impl Event for ModUnloadedEvent {}

/// Request to control a plugin from MOD
///
/// Published by `PluginControlSystem` after draining commands from MODs.
/// Consumed by `PluginControlSystem` to execute actions.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PluginControlRequested {
    pub control: PluginControl,
    /// Which MOD issued this command (optional)
    pub source_mod: Option<String>,
}

impl Event for PluginControlRequested {}

/// Plugin was enabled at runtime
///
/// Published by `PluginControlSystem` after enabling a plugin.
/// Plugins should listen to this and activate themselves.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PluginEnabledEvent {
    pub plugin_name: String,
}

impl Event for PluginEnabledEvent {}

/// Plugin was disabled at runtime
///
/// Published by `PluginControlSystem` after disabling a plugin.
/// Plugins should listen to this and deactivate themselves.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PluginDisabledEvent {
    pub plugin_name: String,
}

impl Event for PluginDisabledEvent {}

/// Plugin parameter changed
///
/// Published by `PluginControlSystem` after parameter change.
/// Plugins should listen to this and update their configuration.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PluginParameterChangedEvent {
    pub plugin_name: String,
    pub key: String,
    pub value: serde_json::Value,
}

impl Event for PluginParameterChangedEvent {}

/// Plugin hook triggered
///
/// Published by `PluginControlSystem` when a MOD triggers a custom hook.
/// Plugins should listen to this and execute hook logic.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PluginHookTriggeredEvent {
    pub plugin_name: String,
    pub hook_name: String,
    pub data: serde_json::Value,
}

impl Event for PluginHookTriggeredEvent {}
