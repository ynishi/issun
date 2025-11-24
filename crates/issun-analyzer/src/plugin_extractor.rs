//! Plugin structure inference from directory layout

use crate::types::PluginInfo;
use std::path::Path;

/// Infer plugin structure from directory layout
///
/// Assumes plugin directory structure like:
/// ```text
/// plugin/
/// ├── combat/
/// │   ├── mod.rs
/// │   ├── system.rs      <- System impl
/// │   ├── hook.rs        <- Hook trait
/// │   ├── events.rs      <- Event types
/// │   ├── config.rs
/// │   └── state.rs
/// ```
pub fn infer_plugins_from_directory<P: AsRef<Path>>(
    plugin_dir: P,
) -> crate::Result<Vec<PluginInfo>> {
    let plugin_dir = plugin_dir.as_ref();
    let mut plugins = Vec::new();

    if !plugin_dir.exists() || !plugin_dir.is_dir() {
        return Ok(plugins);
    }

    // Iterate through subdirectories
    for entry in
        std::fs::read_dir(plugin_dir).map_err(|e| crate::error::AnalyzerError::FileReadError {
            path: plugin_dir.display().to_string(),
            source: e,
        })?
    {
        let entry = entry.map_err(|e| crate::error::AnalyzerError::FileReadError {
            path: plugin_dir.display().to_string(),
            source: e,
        })?;

        let path = entry.path();

        if path.is_dir() {
            if let Some(plugin) = analyze_plugin_directory(&path)? {
                plugins.push(plugin);
            }
        }
    }

    Ok(plugins)
}

/// Analyze a single plugin directory
fn analyze_plugin_directory(dir: &Path) -> crate::Result<Option<PluginInfo>> {
    let plugin_name = dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Check if mod.rs exists (indicates this is a module)
    let mod_file = dir.join("mod.rs");
    if !mod_file.exists() {
        return Ok(None);
    }

    let mut plugin = PluginInfo {
        name: plugin_name,
        path: dir.to_string_lossy().to_string(),
        system: None,
        hooks: Vec::new(),
        events: Vec::new(),
        hook_details: Vec::new(),
    };

    // Try to find system.rs
    let system_file = dir.join("system.rs");
    if system_file.exists() {
        let analyzer = crate::analyzer::Analyzer::new(".");
        if let Ok(systems) = analyzer.analyze_systems(&system_file) {
            if let Some(system) = systems.into_iter().next() {
                plugin.system = Some(system);
            }
        }
    }

    // Try to find hook.rs
    let hook_file = dir.join("hook.rs");
    if hook_file.exists() {
        if let Ok(hooks) = extract_hook_traits(&hook_file) {
            plugin.hooks = hooks;
        }

        // Extract detailed hook information
        if let Ok(hook_details) = extract_hook_details(&hook_file) {
            plugin.hook_details = hook_details;
        }
    }

    // Try to find events.rs
    let events_file = dir.join("events.rs");
    if events_file.exists() {
        if let Ok(events) = extract_event_types(&events_file) {
            plugin.events = events;
        }
    }

    Ok(Some(plugin))
}

/// Extract hook trait names from hook.rs
fn extract_hook_traits(hook_file: &Path) -> crate::Result<Vec<String>> {
    use syn::{File, Item};

    let content = std::fs::read_to_string(hook_file).map_err(|e| {
        crate::error::AnalyzerError::FileReadError {
            path: hook_file.display().to_string(),
            source: e,
        }
    })?;

    let syntax_tree: File =
        syn::parse_file(&content).map_err(|e| crate::error::AnalyzerError::ParseError {
            path: hook_file.display().to_string(),
            source: e,
        })?;

    let mut hooks = Vec::new();

    for item in &syntax_tree.items {
        if let Item::Trait(item_trait) = item {
            let trait_name = item_trait.ident.to_string();
            // Only include traits that end with "Hook"
            if trait_name.ends_with("Hook") {
                hooks.push(trait_name);
            }
        }
    }

    Ok(hooks)
}

/// Extract detailed hook information from hook.rs
fn extract_hook_details(hook_file: &Path) -> crate::Result<Vec<crate::types::HookInfo>> {
    use syn::File;

    let content = std::fs::read_to_string(hook_file).map_err(|e| {
        crate::error::AnalyzerError::FileReadError {
            path: hook_file.display().to_string(),
            source: e,
        }
    })?;

    let syntax_tree: File =
        syn::parse_file(&content).map_err(|e| crate::error::AnalyzerError::ParseError {
            path: hook_file.display().to_string(),
            source: e,
        })?;

    let file_path = hook_file.display().to_string();
    let hook_infos = crate::hook_extractor::extract_hook_traits(&file_path, &syntax_tree);

    Ok(hook_infos)
}

/// Extract event type names from events.rs
fn extract_event_types(events_file: &Path) -> crate::Result<Vec<String>> {
    use syn::{File, Item};

    let content = std::fs::read_to_string(events_file).map_err(|e| {
        crate::error::AnalyzerError::FileReadError {
            path: events_file.display().to_string(),
            source: e,
        }
    })?;

    let syntax_tree: File =
        syn::parse_file(&content).map_err(|e| crate::error::AnalyzerError::ParseError {
            path: events_file.display().to_string(),
            source: e,
        })?;

    let mut events = Vec::new();

    for item in &syntax_tree.items {
        match item {
            Item::Struct(item_struct) => {
                let struct_name = item_struct.ident.to_string();
                // Events typically end with "Event" or "Requested"
                if struct_name.ends_with("Event")
                    || struct_name.ends_with("Requested")
                    || struct_name.ends_with("Command")
                {
                    events.push(struct_name);
                }
            }
            Item::Enum(item_enum) => {
                let enum_name = item_enum.ident.to_string();
                if enum_name.ends_with("Event")
                    || enum_name.ends_with("Requested")
                    || enum_name.ends_with("Command")
                {
                    events.push(enum_name);
                }
            }
            _ => {}
        }
    }

    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_hook_traits() {
        let code = r#"
            pub trait CombatHook: Send + Sync {
                fn before_combat(&self) {}
            }

            pub trait OtherTrait {
                fn method(&self);
            }
        "#;

        // Create a temp file
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_hook.rs");
        std::fs::write(&test_file, code).unwrap();

        let hooks = extract_hook_traits(&test_file).unwrap();
        std::fs::remove_file(&test_file).ok();

        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0], "CombatHook");
    }

    #[test]
    fn test_extract_event_types() {
        let code = r#"
            #[derive(Clone)]
            pub struct CombatStartRequested {
                pub battle_id: String,
            }

            #[derive(Clone)]
            pub struct CombatStartedEvent {
                pub battle_id: String,
            }

            pub enum CombatCommand {
                Start,
                End,
            }
        "#;

        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_events.rs");
        std::fs::write(&test_file, code).unwrap();

        let events = extract_event_types(&test_file).unwrap();
        std::fs::remove_file(&test_file).ok();

        assert_eq!(events.len(), 3);
        assert!(events.contains(&"CombatStartRequested".to_string()));
        assert!(events.contains(&"CombatStartedEvent".to_string()));
        assert!(events.contains(&"CombatCommand".to_string()));
    }
}
