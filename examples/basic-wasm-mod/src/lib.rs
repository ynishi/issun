//! Basic WebAssembly MOD for ISSUN
//!
//! This is a guest module that implements the mod-guest WIT interface.

use wit_bindgen::generate;

// Generate guest bindings from WIT
generate!({
    world: "mod-guest",
    path: "../../crates/issun-mod-wasm/wit/issun.wit",
});

// Export the MOD implementation
struct PandemicMod;

impl Guest for PandemicMod {
    fn get_metadata() -> Metadata {
        Metadata {
            name: "Wasm Pandemic Controller".to_string(),
            version: "1.0.0".to_string(),
            author: Some("ISSUN Team".to_string()),
            description: Some("WebAssembly-based pandemic simulation controller".to_string()),
        }
    }

    fn on_init() {
        issun::mod_::api::log("🦠 Wasm Pandemic MOD initialized!");
        issun::mod_::api::enable_plugin("contagion");
        issun::mod_::api::set_plugin_param("contagion", "infection_rate", "0.05");
        issun::mod_::api::log("Initial infection rate: 5%");
    }

    fn on_shutdown() {
        issun::mod_::api::log("Wasm Pandemic MOD shutting down...");
    }

    fn on_control_plugin(plugin_name: String, action: String) {
        let msg = format!("Controlling plugin: {} - {}", plugin_name, action);
        issun::mod_::api::log(&msg);
    }

    fn call_custom(fn_name: String, args: Vec<String>) -> String {
        match fn_name.as_str() {
            "calculate_risk" => {
                // Parse arguments
                let infection_count: f64 = args.get(0)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0.0);
                let population: f64 = args.get(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(1.0);

                let risk = infection_count / population;
                let risk_level = if risk < 0.1 {
                    "LOW"
                } else if risk < 0.3 {
                    "MODERATE"
                } else if risk < 0.6 {
                    "HIGH"
                } else {
                    "CRITICAL"
                };

                // Return as JSON string
                serde_json::json!({
                    "risk_level": risk_level,
                    "risk_value": risk
                }).to_string()
            }
            "tick" => {
                let turn: u32 = args.get(0)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);

                if turn == 50 {
                    issun::mod_::api::log("⚠️  Pandemic entering critical phase!");
                    issun::mod_::api::set_plugin_param("contagion", "infection_rate", "0.10");
                } else if turn == 100 {
                    issun::mod_::api::log("🔴 PANDEMIC OUTBREAK!");
                    issun::mod_::api::set_plugin_param("contagion", "infection_rate", "0.15");
                } else if turn == 200 {
                    issun::mod_::api::log("✅ Vaccine developed!");
                    issun::mod_::api::set_plugin_param("contagion", "infection_rate", "0.03");
                }

                serde_json::json!({ "turn": turn }).to_string()
            }
            _ => {
                serde_json::json!({ "error": "Unknown function" }).to_string()
            }
        }
    }
}

// Export the implementation
export!(PandemicMod);
