//! Example: Generate Combined Event + Hook Flow Graph

use issun_analyzer::plugin_extractor::infer_plugins_from_directory;
use issun_analyzer::prelude::*;
use std::path::PathBuf;

fn main() -> Result<()> {
    println!("🔍 ISSUN Combined Flow Graph Generator");
    println!("========================================\n");

    // Path to plugin directory
    let plugin_dir = PathBuf::from("crates/issun/src/plugin");

    if !plugin_dir.exists() {
        eprintln!("❌ Directory not found: {}", plugin_dir.display());
        eprintln!("   Please run this example from the project root directory.");
        return Ok(());
    }

    println!("📂 Analyzing plugins in: {}", plugin_dir.display());

    // Infer plugins from directory
    let plugins = infer_plugins_from_directory(&plugin_dir)?;

    let mut result = AnalysisResult::new();
    for plugin in plugins {
        result.add_plugin(plugin);
    }

    println!("\n📊 Analysis Results:");
    println!("   Plugins: {}", result.plugins.len());
    println!(
        "   Plugins with systems: {}",
        result.plugins.iter().filter(|p| p.system.is_some()).count()
    );

    // Generate Combined Flow Graph (first 3 plugins with systems)
    println!("\n📈 Generating Combined Flow Graph (first 3 plugins)...\n");

    let filter_plugins: Vec<String> = result
        .plugins
        .iter()
        .filter(|p| p.system.is_some())
        .take(3)
        .map(|p| p.name.clone())
        .collect();

    let options = GraphOptions {
        filter_plugins: filter_plugins.clone(),
        show_hooks: true,
        ..Default::default()
    };

    let graph_gen = CombinedFlowGraphGenerator::with_options(&result, options);
    let mermaid_graph = graph_gen.generate();

    println!("```mermaid");
    println!("{}", mermaid_graph);
    println!("```");

    // Save to file
    let output_path = PathBuf::from("combined_flow.mmd");
    std::fs::write(&output_path, &mermaid_graph).map_err(|e| {
        issun_analyzer::AnalyzerError::FileWriteError {
            path: output_path.display().to_string(),
            source: e,
        }
    })?;

    println!("\n✅ Graph saved to: {}", output_path.display());
    println!("   You can visualize it at: https://mermaid.live");

    // Print summary
    println!("\n📋 Graph includes:");
    for plugin_name in &filter_plugins {
        if let Some(plugin) = result.plugins.iter().find(|p| &p.name == plugin_name) {
            if let Some(system) = &plugin.system {
                println!("\n   📦 {} Plugin", plugin.name);
                println!("      ⚙️  System: {}", system.name);
                println!("      📨 Subscribes: {}", system.subscribes.len());
                println!("      📤 Publishes: {}", system.publishes.len());
                println!("      🪝 Hooks: {}", system.hooks.len());
            }
        }
    }

    Ok(())
}
