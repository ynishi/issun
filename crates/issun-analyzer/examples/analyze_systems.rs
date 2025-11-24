//! Example: Analyze Systems to extract structure and dependencies

use issun_analyzer::prelude::*;
use std::path::PathBuf;

fn main() -> Result<()> {
    println!("🔍 ISSUN System Analyzer - Extract System Structures");
    println!("====================================================\n");

    // Path to CombatSystem source file
    let combat_system_path = PathBuf::from("crates/issun/src/plugin/combat/system.rs");

    if !combat_system_path.exists() {
        eprintln!("❌ File not found: {}", combat_system_path.display());
        eprintln!("   Please run this example from the project root directory.");
        return Ok(());
    }

    // Create analyzer
    let analyzer = Analyzer::new(".");

    // Analyze systems in the file
    println!("📂 Analyzing: {}", combat_system_path.display());
    let systems = analyzer.analyze_systems(&combat_system_path)?;

    // Display results
    println!("\n📊 Analysis Results:");
    println!("   Systems found: {}", systems.len());

    for system in &systems {
        println!("\n🔧 System: {}", system.name);
        println!("   Module: {}", system.module_path);
        println!("   File: {}", system.file_path);

        if !system.hooks.is_empty() {
            println!("   Hooks:");
            for hook in &system.hooks {
                println!("      - {}", hook);
            }
        }

        if !system.states.is_empty() {
            println!("   States:");
            for state in &system.states {
                println!("      - {}", state);
            }
        }
    }

    // Export as JSON
    if !systems.is_empty() {
        let json_output = serde_json::to_string_pretty(&systems)?;
        println!("\n📝 JSON Output:");
        println!("{}", json_output);
    }

    println!("\n✅ Analysis complete!");

    Ok(())
}
