//! Example: Generate Event Flow Graph in Mermaid format

use issun_analyzer::prelude::*;
use std::path::PathBuf;

fn main() -> Result<()> {
    println!("🔍 ISSUN Event Flow Graph Generator");
    println!("====================================\n");

    // Analyze Combat plugin
    let combat_system_path = PathBuf::from("crates/issun/src/plugin/combat/system.rs");

    if !combat_system_path.exists() {
        eprintln!("❌ File not found: {}", combat_system_path.display());
        eprintln!("   Please run this example from the project root directory.");
        return Ok(());
    }

    println!("📂 Analyzing: {}", combat_system_path.display());

    let analyzer = Analyzer::new(".");
    let mut result = AnalysisResult::new();

    // Analyze file
    let file_analysis = analyzer.analyze_file(&combat_system_path)?;
    result.add_file(file_analysis);

    // Analyze systems
    let systems = analyzer.analyze_systems(&combat_system_path)?;
    for system in systems {
        result.add_system(system);
    }

    println!("\n📊 Analysis Results:");
    println!("   Events subscribed: {}", result.all_subscriptions().len());
    println!("   Events published: {}", result.all_publications().len());
    println!("   Systems: {}", result.systems.len());

    // Generate Event Flow Graph
    println!("\n📈 Generating Event Flow Graph...\n");

    let graph_gen = EventFlowGraphGenerator::new(&result);
    let mermaid_graph = graph_gen.generate();

    println!("```mermaid");
    println!("{}", mermaid_graph);
    println!("```");

    // Save to file
    let output_path = PathBuf::from("event_flow.mmd");
    std::fs::write(&output_path, &mermaid_graph).map_err(|e| {
        issun_analyzer::AnalyzerError::FileWriteError {
            path: output_path.display().to_string(),
            source: e,
        }
    })?;

    println!("\n✅ Graph saved to: {}", output_path.display());
    println!("   You can visualize it at: https://mermaid.live");

    Ok(())
}
