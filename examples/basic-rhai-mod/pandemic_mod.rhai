// Pandemic MOD for ISSUN
// Controls the contagion plugin dynamically based on game state

fn get_metadata() {
    #{
        name: "Pandemic Controller",
        version: "1.0.0",
        author: "ISSUN Team",
        description: "Dynamically adjusts infection rates based on player actions"
    }
}

// Called when the MOD is loaded
fn on_init() {
    log("🦠 Pandemic MOD initialized!");
    enable_plugin("contagion");
    set_plugin_param("contagion", "infection_rate", 0.05);
    log("Initial infection rate: 5%");
}

// Called each game turn
fn on_tick(turn_number) {
    log(`Turn ${turn_number}: Checking pandemic status...`);

    if turn_number == 50 {
        log("⚠️  Pandemic entering critical phase!");
        set_plugin_param("contagion", "infection_rate", 0.10);
    }

    if turn_number == 100 {
        log("🔴 PANDEMIC OUTBREAK! Infection rate increased!");
        set_plugin_param("contagion", "infection_rate", 0.15);
    }

    if turn_number == 200 {
        log("✅ Vaccine developed! Infection rate decreasing...");
        set_plugin_param("contagion", "infection_rate", 0.03);
    }
}

// Called when a plugin control command is issued
fn on_control_plugin(plugin_name, action) {
    log(`MOD controlling ${plugin_name}: ${action}`);
}

// Called when the MOD is unloaded
fn on_shutdown() {
    log("Pandemic MOD shutting down...");
}

// Custom function: Calculate risk level
fn calculate_risk(infection_count, population) {
    let risk = infection_count / population;

    if risk < 0.1 {
        "LOW"
    } else if risk < 0.3 {
        "MODERATE"
    } else if risk < 0.6 {
        "HIGH"
    } else {
        "CRITICAL"
    }
}
