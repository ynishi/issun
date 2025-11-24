//! Example: Generate Hook Flow Graph in Mermaid format

use issun_analyzer::plugin_extractor::infer_plugins_from_directory;
use issun_analyzer::prelude::*;
use std::path::PathBuf;

fn main() -> Result<()> {
    println!("🔍 ISSUN Hook Flow Graph Generator");
    println!("===================================\n");

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
        "   Plugins with hooks: {}",
        result
            .plugins
            .iter()
            .filter(|p| !p.hook_details.is_empty())
            .count()
    );

    // Generate Hook Flow Graph (first 5 plugins)
    println!("\n📈 Generating Hook Flow Graph (first 5 plugins)...\n");

    let options = GraphOptions {
        filter_plugins: result
            .plugins
            .iter()
            .filter(|p| !p.hook_details.is_empty())
            .take(5)
            .map(|p| p.name.clone())
            .collect(),
        ..Default::default()
    };

    let graph_gen = HookFlowGraphGenerator::with_options(&result, options);
    let mermaid_graph = graph_gen.generate();

    println!("```mermaid");
    println!("{}", mermaid_graph);
    println!("```");

    // Save to file
    let output_path = PathBuf::from("hook_flow.mmd");
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
