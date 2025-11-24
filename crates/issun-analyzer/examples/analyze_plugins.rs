//! Example: Infer plugin structures from directory layout

use issun_analyzer::plugin_extractor::infer_plugins_from_directory;
use issun_analyzer::prelude::*;
use std::path::PathBuf;

fn main() -> Result<()> {
    println!("🔍 ISSUN Plugin Analyzer - Infer Plugin Structures");
    println!("===================================================\n");

    // Path to plugin directory
    let plugin_dir = PathBuf::from("crates/issun/src/plugin");

    if !plugin_dir.exists() {
        eprintln!("❌ Directory not found: {}", plugin_dir.display());
        eprintln!("   Please run this example from the project root directory.");
        return Ok(());
    }

    // Infer plugins from directory structure
    println!("📂 Analyzing plugins in: {}", plugin_dir.display());
    let plugins = infer_plugins_from_directory(&plugin_dir)?;

    // Display results
    println!("\n📊 Analysis Results:");
    println!("   Plugins found: {}", plugins.len());

    for plugin in &plugins {
        println!("\n📦 Plugin: {}", plugin.name);
        println!("   Path: {}", plugin.path);

        if let Some(ref system) = plugin.system {
            println!("   System: {}", system.name);

            if !system.subscribes.is_empty() {
                println!("   Subscribes:");
                for event in &system.subscribes {
                    println!("      📨 {}", event);
                }
            }

            if !system.publishes.is_empty() {
                println!("   Publishes:");
                for event in &system.publishes {
                    println!("      📤 {}", event);
                }
            }

            if !system.hooks.is_empty() {
                println!("   Hooks:");
                for hook in &system.hooks {
                    println!("      🪝 {}", hook);
                }
            }
        }

        if !plugin.hooks.is_empty() {
            println!("   Hook Traits:");
            for hook in &plugin.hooks {
                println!("      🔧 {}", hook);
            }
        }

        if !plugin.events.is_empty() {
            println!("   Event Types:");
            for event in &plugin.events {
                println!("      📋 {}", event);
            }
        }
    }

    // Export as JSON (first plugin only for brevity)
    if let Some(plugin) = plugins.first() {
        let json_output = serde_json::to_string_pretty(&plugin)?;
        println!("\n📝 JSON Output (first plugin):");
        println!("{}", json_output);
    }

    println!("\n✅ Analysis complete!");

    Ok(())
}
