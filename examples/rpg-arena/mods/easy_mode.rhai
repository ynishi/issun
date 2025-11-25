// Easy Mode MOD
// Makes the game easier for beginners

fn get_metadata() {
    #{
        name: "Easy Mode",
        version: "1.0.0",
        author: "ISSUN Team",
        description: "Make the game easier for beginners - High HP, low difficulty, large inventory"
    }
}

fn on_init() {
    log("🌟 Easy Mode activated!");

    // Enable combat system
    enable_plugin("combat");

    // High HP, low difficulty
    set_plugin_param("combat", "max_hp", 200);
    set_plugin_param("combat", "difficulty", 0.5);

    log("   💪 Combat: Max HP = 200, Difficulty = 0.5x");

    // Enable inventory system
    enable_plugin("inventory");

    // Large inventory with stacking
    set_plugin_param("inventory", "max_slots", 30);
    set_plugin_param("inventory", "allow_stacking", true);

    log("   🎒 Inventory: 30 slots, stacking enabled");
}

fn on_shutdown() {
    log("👋 Easy Mode deactivated");
}
