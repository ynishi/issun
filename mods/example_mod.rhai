// Example MOD for ISSUN
//
// This MOD demonstrates the Plugin Control API

/// MOD metadata
fn get_metadata() {
    #{
        name: "Example MOD",
        version: "1.0.0",
        author: "Claude",
        description: "Demonstrates MOD system features"
    }
}

/// Called when MOD is loaded
fn on_init() {
    log("=== Example MOD Initialized ===");
    log("This MOD will enable the combat plugin and configure it");

    // Enable plugins
    enable_plugin("combat");
    enable_plugin("inventory");

    // Configure plugin parameters
    set_plugin_param("combat", "max_hp", 100);
    set_plugin_param("combat", "difficulty", 1.5);
    set_plugin_param("inventory", "max_slots", 20);

    log("Example MOD setup complete!");
}

/// Called when MOD is unloaded
fn on_shutdown() {
    log("=== Example MOD Shutting Down ===");

    // Optionally disable plugins
    disable_plugin("combat");

    log("Example MOD unloaded");
}

/// Called when plugin control is requested
fn on_control_plugin(plugin_name, action) {
    log("Control request: " + plugin_name + " -> " + action);
}
