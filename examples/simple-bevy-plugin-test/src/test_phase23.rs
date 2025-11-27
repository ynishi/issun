//! Test Phase 2.3: Plugin Dependency Checking
//!
//! Tests:
//! - IssunCorePlugin dependency check (auto_require_core = true)
//! - Custom plugin dependencies (requires attribute)
//! - Disabling IssunCorePlugin check (auto_require_core = false)

use bevy::prelude::*;
use issun_bevy::IssunCorePlugin;
use issun_macros::IssunBevyPlugin;

#[derive(Resource, Clone, Debug, Default)]
pub struct DependencyTestData {
    pub value: u32,
}

// Test 9: Plugin with auto_require_core = true (default)
#[derive(Default, IssunBevyPlugin)]
#[plugin(name = "requires_core_test")]
pub struct RequiresCorePlugin {
    #[resource]
    pub data: DependencyTestData,
}

// Test 10: Plugin that requires another plugin
#[derive(Default, IssunBevyPlugin)]
#[plugin(name = "dependent_test", requires = [RequiresCorePlugin])]
pub struct DependentPlugin {
    #[resource]
    pub data: DependencyTestData,
}

// Test 11: Plugin with auto_require_core = false
#[derive(Default, IssunBevyPlugin)]
#[plugin(name = "no_core_test", auto_require_core = false)]
pub struct NoCoreRequirePlugin {
    #[resource]
    pub data: DependencyTestData,
}

pub fn run_phase23_tests() {
    test_issun_core_plugin_required();
    test_custom_plugin_dependency();
    test_auto_require_core_disabled();
}

fn test_issun_core_plugin_required() {
    println!("\nTest 9: IssunCorePlugin Required (Phase 2.3)");

    // Test 9a: Should panic without IssunCorePlugin
    let result = std::panic::catch_unwind(|| {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RequiresCorePlugin::default()); // Should panic!
    });

    assert!(
        result.is_err(),
        "Plugin should panic when IssunCorePlugin is missing"
    );
    println!("  ✅ Correctly panics when IssunCorePlugin is missing");

    // Test 9b: Should succeed with IssunCorePlugin
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(IssunCorePlugin); // Add required plugin
    app.add_plugins(RequiresCorePlugin::default());

    app.update();

    // Verify plugin was added successfully
    assert!(
        app.world().get_resource::<DependencyTestData>().is_some(),
        "Plugin should be added successfully when IssunCorePlugin is present"
    );

    // Verify marker was registered
    assert!(
        app.world()
            .get_resource::<issun_bevy::IssunCorePluginMarker>()
            .is_some(),
        "IssunCorePluginMarker should be registered"
    );

    println!("  ✅ Succeeds when IssunCorePlugin is present");
    println!("✅ IssunCorePlugin dependency check works correctly");
}

fn test_custom_plugin_dependency() {
    println!("\nTest 10: Custom Plugin Dependency (Phase 2.3)");

    // Test 10a: Should panic without RequiresCorePlugin
    let result = std::panic::catch_unwind(|| {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(IssunCorePlugin);
        app.add_plugins(DependentPlugin::default()); // Should panic! (no RequiresCorePlugin)
    });

    assert!(
        result.is_err(),
        "Plugin should panic when required plugin is missing"
    );
    println!("  ✅ Correctly panics when required plugin is missing");

    // Test 10b: Should succeed with RequiresCorePlugin
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(IssunCorePlugin);
    app.add_plugins(RequiresCorePlugin::default()); // Add required plugin first
    app.add_plugins(DependentPlugin::default());

    app.update();

    // Verify both plugins were added successfully
    assert!(
        app.world().get_resource::<DependencyTestData>().is_some(),
        "Plugin should be added successfully when dependencies are satisfied"
    );

    println!("  ✅ Succeeds when required plugin is present");
    println!("✅ Custom plugin dependency check works correctly");
}

fn test_auto_require_core_disabled() {
    println!("\nTest 11: auto_require_core = false (Phase 2.3)");

    // Should succeed WITHOUT IssunCorePlugin when auto_require_core = false
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(NoCoreRequirePlugin::default()); // No IssunCorePlugin, but should work

    app.update();

    // Verify plugin was added successfully
    assert!(
        app.world().get_resource::<DependencyTestData>().is_some(),
        "Plugin should be added successfully when auto_require_core = false"
    );

    println!("  ✅ Succeeds without IssunCorePlugin when auto_require_core = false");
    println!("✅ auto_require_core = false works correctly");
}
