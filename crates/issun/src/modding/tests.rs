//! Unit tests for MOD system

use super::*;
use std::path::Path;

/// Mock loader for testing
struct MockLoader {
    mods: Vec<ModHandle>,
}

impl MockLoader {
    fn new() -> Self {
        Self { mods: Vec::new() }
    }
}

impl ModLoader for MockLoader {
    fn load(&mut self, path: &Path) -> ModResult<ModHandle> {
        let id = path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| ModError::InvalidFormat("Invalid path".to_string()))?
            .to_string();

        let handle = ModHandle {
            id: id.clone(),
            metadata: ModMetadata {
                name: format!("Test Mod: {}", id),
                version: "1.0.0".to_string(),
                author: Some("Test Author".to_string()),
                description: Some("Test Description".to_string()),
            },
            backend: ModBackend::Rhai,
        };

        self.mods.push(handle.clone());
        Ok(handle)
    }

    fn unload(&mut self, handle: &ModHandle) -> ModResult<()> {
        self.mods.retain(|m| m.id != handle.id);
        Ok(())
    }

    fn control_plugin(&mut self, _handle: &ModHandle, _control: &PluginControl) -> ModResult<()> {
        // Mock implementation
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn ModLoader> {
        Box::new(Self::new())
    }
}

#[test]
fn test_mod_metadata_serialization() {
    let metadata = ModMetadata {
        name: "Test Mod".to_string(),
        version: "1.0.0".to_string(),
        author: Some("Author".to_string()),
        description: Some("Description".to_string()),
    };

    let json = serde_json::to_string(&metadata).unwrap();
    let deserialized: ModMetadata = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.name, "Test Mod");
    assert_eq!(deserialized.version, "1.0.0");
}

#[test]
fn test_mod_backend_display() {
    assert_eq!(ModBackend::Rhai.to_string(), "rhai");
    assert_eq!(ModBackend::Wasm.to_string(), "wasm");
}

#[test]
fn test_plugin_control_builders() {
    let control = PluginControl::enable("combat");
    assert_eq!(control.plugin_name, "combat");
    assert!(matches!(control.action, PluginAction::Enable));

    let control = PluginControl::disable("economy");
    assert_eq!(control.plugin_name, "economy");
    assert!(matches!(control.action, PluginAction::Disable));

    let control = PluginControl::set_param("contagion", "rate", serde_json::json!(0.5));
    assert_eq!(control.plugin_name, "contagion");
    if let PluginAction::SetParameter { key, value } = &control.action {
        assert_eq!(key, "rate");
        assert_eq!(value, &serde_json::json!(0.5));
    } else {
        panic!("Expected SetParameter action");
    }
}

#[test]
fn test_mock_loader_load() {
    let mut loader = MockLoader::new();
    let path = Path::new("test_mod.rhai");

    let handle = loader.load(path).unwrap();
    assert_eq!(handle.id, "test_mod");
    assert_eq!(handle.metadata.name, "Test Mod: test_mod");
    assert_eq!(handle.backend, ModBackend::Rhai);
    assert_eq!(loader.mods.len(), 1);
}

#[test]
fn test_mock_loader_unload() {
    let mut loader = MockLoader::new();
    let path = Path::new("test_mod.rhai");

    let handle = loader.load(path).unwrap();
    assert_eq!(loader.mods.len(), 1);

    loader.unload(&handle).unwrap();
    assert_eq!(loader.mods.len(), 0);
}

#[test]
fn test_mock_loader_control_plugin() {
    let mut loader = MockLoader::new();
    let path = Path::new("test_mod.rhai");

    let handle = loader.load(path).unwrap();
    let control = PluginControl::enable("combat");

    // Should not error
    loader.control_plugin(&handle, &control).unwrap();
}

#[test]
fn test_mod_system_config_default() {
    let config = ModSystemConfig::default();
    assert_eq!(config.mod_dir, "mods");
    assert!(!config.hot_reload);
    assert!(config.auto_load);
}

#[test]
fn test_mod_error_display() {
    let err = ModError::LoadFailed("test error".to_string());
    assert_eq!(err.to_string(), "Failed to load MOD: test error");

    let err = ModError::PluginNotFound("combat".to_string());
    assert_eq!(err.to_string(), "Plugin not found: combat");
}
