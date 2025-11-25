// Debug Mode MOD
// For testing and debugging

fn get_metadata() {
    #{
        name: "Debug Mode",
        version: "1.0.0",
        author: "ISSUN Team",
        description: "Development mode - Very high HP, no difficulty, unlimited inventory"
    }
}

fn on_init() {
    log("🔧 Debug Mode activated!");

    // Enable all systems
    enable_plugin("combat");
    enable_plugin("inventory");

    // God mode stats
    set_plugin_param("combat", "max_hp", 9999);
    set_plugin_param("combat", "difficulty", 0.1);

    log("   ⚡ Combat: Max HP = 9999, Difficulty = 0.1x (God Mode)");

    // Unlimited inventory
    set_plugin_param("inventory", "max_slots", 999);
    set_plugin_param("inventory", "allow_stacking", true);

    log("   🎒 Inventory: 999 slots, stacking enabled");
}

fn on_shutdown() {
    log("👋 Debug Mode deactivated");
}
