// Hard Mode MOD
// Challenge for experienced players

fn get_metadata() {
    #{
        name: "Hard Mode",
        version: "1.0.0",
        author: "ISSUN Team",
        description: "Challenge for experienced players - Low HP, high difficulty, limited inventory"
    }
}

fn on_init() {
    log("💀 Hard Mode activated!");

    // Enable combat system
    enable_plugin("combat");

    // Low HP, high difficulty
    set_plugin_param("combat", "max_hp", 50);
    set_plugin_param("combat", "difficulty", 2.0);

    log("   ⚔️ Combat: Max HP = 50, Difficulty = 2.0x");

    // Enable inventory system
    enable_plugin("inventory");

    // Small inventory, no stacking
    set_plugin_param("inventory", "max_slots", 5);
    set_plugin_param("inventory", "allow_stacking", false);

    log("   🎒 Inventory: 5 slots, stacking disabled");
}

fn on_shutdown() {
    log("👋 Hard Mode deactivated");
}
