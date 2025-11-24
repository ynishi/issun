//! Example: Validate Event Flow for consistency issues

use issun_analyzer::plugin_extractor::infer_plugins_from_directory;
use issun_analyzer::prelude::*;
use std::path::PathBuf;

fn main() -> Result<()> {
    println!("🔍 ISSUN Event Flow Validator");
    println!("==============================\n");

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

    println!("\n📊 Analysis Summary:");
    println!("   Plugins analyzed: {}", result.plugins.len());
    println!(
        "   Systems with events: {}",
        result
            .plugins
            .iter()
            .filter(
                |p| p.system.as_ref().is_some_and(|s| !s.subscribes.is_empty()
                    || !s.publishes.is_empty())
            )
            .count()
    );

    // Count total events
    let total_subscriptions = result
        .plugins
        .iter()
        .filter_map(|p| p.system.as_ref())
        .map(|s| s.subscribes.len())
        .sum::<usize>();

    let total_publications = result
        .plugins
        .iter()
        .filter_map(|p| p.system.as_ref())
        .map(|s| s.publishes.len())
        .sum::<usize>();

    println!("   Total subscriptions: {}", total_subscriptions);
    println!("   Total publications: {}", total_publications);

    // Run validation
    println!("\n🔎 Running validation checks...\n");
    let validator = Validator::new(&result);
    let validation = validator.validate();

    // Print report
    validation.print_report();

    // Print summary
    println!("\n📋 Validation Summary:");
    println!(
        "   🔴 High severity: {}",
        validation.warnings_by_severity(WarningSeverity::High).len()
    );
    println!(
        "   🟡 Medium severity: {}",
        validation
            .warnings_by_severity(WarningSeverity::Medium)
            .len()
    );
    println!(
        "   🟢 Low severity: {}",
        validation.warnings_by_severity(WarningSeverity::Low).len()
    );

    if validation.has_high_severity_warnings() {
        println!("\n⚠️  High severity warnings found! Please review event flows.");
    } else if validation.warnings.is_empty() {
        println!("\n✅ All validation checks passed!");
    }

    Ok(())
}
