//! Example: Analyze Hook traits and method categorization

use issun_analyzer::plugin_extractor::infer_plugins_from_directory;
use issun_analyzer::prelude::*;
use issun_analyzer::HookMethod;
use std::path::PathBuf;

fn main() -> Result<()> {
    println!("🔍 ISSUN Hook Analyzer - Extract Hook Traits");
    println!("==============================================\n");

    // Path to plugin directory
    let plugin_dir = PathBuf::from("crates/issun/src/plugin");

    if !plugin_dir.exists() {
        eprintln!("❌ Directory not found: {}", plugin_dir.display());
        eprintln!("   Please run this example from the project root directory.");
        return Ok(());
    }

    // Infer plugins from directory structure
    println!("📂 Analyzing hooks in: {}", plugin_dir.display());
    let plugins = infer_plugins_from_directory(&plugin_dir)?;

    // Filter plugins that have hooks
    let plugins_with_hooks: Vec<_> = plugins
        .iter()
        .filter(|p| !p.hook_details.is_empty())
        .collect();

    println!("\n📊 Analysis Results:");
    println!("   Plugins with hooks: {}", plugins_with_hooks.len());

    // Display first few plugins with detailed hook information
    for plugin in plugins_with_hooks.iter().take(3) {
        println!("\n📦 Plugin: {}", plugin.name);

        for hook_info in &plugin.hook_details {
            println!("   🪝 {}", hook_info.trait_name);
            println!("      Module: {}", hook_info.module_path);
            println!("      Methods: {}", hook_info.methods.len());

            // Group methods by category
            use std::collections::HashMap;
            let mut by_category: HashMap<String, Vec<&HookMethod>> = HashMap::new();

            for method in &hook_info.methods {
                let category = format!("{:?}", method.category);
                by_category.entry(category).or_default().push(method);
            }

            for (category, methods) in &by_category {
                println!("\n      {} ({}):", category, methods.len());
                for method in methods {
                    let params_str = if method.params.is_empty() {
                        String::new()
                    } else {
                        format!("({})", method.params.join(", "))
                    };

                    let default_marker = if method.has_default_impl {
                        " [default]"
                    } else {
                        ""
                    };

                    println!(
                        "         • {}{} -> {}{}",
                        method.name, params_str, method.return_type, default_marker
                    );
                }
            }
        }
    }

    // Export first hook as JSON
    if let Some(plugin) = plugins_with_hooks.first() {
        if let Some(hook_info) = plugin.hook_details.first() {
            let json_output = serde_json::to_string_pretty(&hook_info)?;
            println!("\n📝 JSON Output (first hook):");
            println!("{}", json_output);
        }
    }

    println!("\n✅ Analysis complete!");

    Ok(())
}
