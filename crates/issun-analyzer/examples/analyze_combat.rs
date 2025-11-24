//! Example: Analyze CombatSystem to extract event subscriptions and publications

use issun_analyzer::prelude::*;
use std::path::PathBuf;

fn main() -> Result<()> {
    println!("🔍 ISSUN Event Analyzer - CombatSystem Analysis");
    println!("=================================================\n");

    // Path to CombatSystem source file
    let combat_system_path = PathBuf::from("crates/issun/src/plugin/combat/system.rs");

    if !combat_system_path.exists() {
        eprintln!("❌ File not found: {}", combat_system_path.display());
        eprintln!("   Please run this example from the project root directory.");
        return Ok(());
    }

    // Create analyzer
    let analyzer = Analyzer::new(".");

    // Analyze the file
    println!("📂 Analyzing: {}", combat_system_path.display());
    let analysis = analyzer.analyze_file(&combat_system_path)?;

    // Display results
    println!("\n📊 Analysis Results:");
    println!("   File: {}", analysis.path);
    println!("   Subscriptions: {}", analysis.subscriptions.len());
    println!("   Publications: {}", analysis.publications.len());

    // Show event subscriptions
    if !analysis.subscriptions.is_empty() {
        println!("\n📨 Event Subscriptions (EventReader<E>):");
        for sub in &analysis.subscriptions {
            println!("   • {} subscribes to {}", sub.subscriber, sub.event_type);
        }
    }

    // Show event publications
    if !analysis.publications.is_empty() {
        println!("\n📤 Event Publications (EventBus::publish):");
        for pub_event in &analysis.publications {
            println!(
                "   • {} publishes {}",
                pub_event.publisher, pub_event.event_type
            );
        }
    }

    // Export as JSON
    let json_output = serde_json::to_string_pretty(&analysis)?;
    println!("\n📝 JSON Output:");
    println!("{}", json_output);

    println!("\n✅ Analysis complete!");

    Ok(())
}
