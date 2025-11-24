//! Analyze command - Static analysis of ISSUN plugins

use crate::config::Config;
use crate::error::Result;
use clap::Args;
use issun_analyzer::plugin_extractor::infer_plugins_from_directory;
use issun_analyzer::prelude::*;
use std::path::PathBuf;

/// Analyze plugin architecture and event flows
#[derive(Args, Debug)]
pub struct AnalyzeCommand {
    /// Generate event flow graph
    #[arg(long)]
    pub event_flow: bool,

    /// Generate hook flow graph
    #[arg(long)]
    pub hook_flow: bool,

    /// Generate combined event + hook flow graph
    #[arg(long)]
    pub combined_flow: bool,

    /// Validate event consistency
    #[arg(long)]
    pub validate: bool,

    /// List all plugins
    #[arg(long)]
    pub list_plugins: bool,

    /// Show plugin details
    #[arg(long)]
    pub plugin_details: bool,

    /// Filter by specific plugins (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub plugins: Option<Vec<String>>,

    /// Output file path (default: stdout or <type>.mmd for graphs)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Maximum number of plugins to show in graphs
    #[arg(long, default_value = "5")]
    pub max_plugins: usize,
}

impl AnalyzeCommand {
    pub fn execute(&self, config: &Config) -> Result<()> {
        println!("🔍 ISSUN Static Analyzer");
        println!("========================\n");

        // Validate configuration
        config.validate()?;

        let plugin_dir = config.plugin_dir_absolute();
        println!("📂 Analyzing plugins in: {}\n", plugin_dir.display());

        // Infer plugins from directory
        let plugins = infer_plugins_from_directory(&plugin_dir)?;

        let mut result = AnalysisResult::new();
        for plugin in plugins {
            result.add_plugin(plugin);
        }

        println!("📊 Analysis Summary:");
        println!("   Total plugins: {}", result.plugins.len());
        println!(
            "   Plugins with systems: {}",
            result.plugins.iter().filter(|p| p.system.is_some()).count()
        );
        println!(
            "   Plugins with hooks: {}",
            result
                .plugins
                .iter()
                .filter(|p| !p.hook_details.is_empty())
                .count()
        );
        println!();

        // Execute requested operations
        let mut executed = false;

        if self.list_plugins {
            self.list_plugins_info(&result)?;
            executed = true;
        }

        if self.plugin_details {
            self.show_plugin_details(&result)?;
            executed = true;
        }

        if self.event_flow {
            self.generate_event_flow_graph(&result, config)?;
            executed = true;
        }

        if self.hook_flow {
            self.generate_hook_flow_graph(&result, config)?;
            executed = true;
        }

        if self.combined_flow {
            self.generate_combined_flow_graph(&result, config)?;
            executed = true;
        }

        if self.validate {
            self.validate_event_flow(&result)?;
            executed = true;
        }

        if !executed {
            println!("ℹ️  No operation specified. Use --help to see available options.");
            println!("   Example: issun analyze --list-plugins --validate");
        }

        Ok(())
    }

    fn list_plugins_info(&self, result: &AnalysisResult) -> Result<()> {
        println!("📦 Plugin List:\n");

        for plugin in &result.plugins {
            let system_info = if let Some(system) = &plugin.system {
                format!(
                    "System: {} (subs: {}, pubs: {}, hooks: {})",
                    system.name,
                    system.subscribes.len(),
                    system.publishes.len(),
                    system.hooks.len()
                )
            } else {
                "No system".to_string()
            };

            let hooks_info = if !plugin.hook_details.is_empty() {
                format!("{} hook traits", plugin.hook_details.len())
            } else {
                "No hooks".to_string()
            };

            println!("  • {} - {} | {}", plugin.name, system_info, hooks_info);
        }

        println!();
        Ok(())
    }

    fn show_plugin_details(&self, result: &AnalysisResult) -> Result<()> {
        println!("📋 Plugin Details:\n");

        let plugins_to_show: Vec<_> = if let Some(filter) = &self.plugins {
            result
                .plugins
                .iter()
                .filter(|p| filter.contains(&p.name))
                .collect()
        } else {
            result.plugins.iter().take(self.max_plugins).collect()
        };

        for plugin in plugins_to_show {
            println!("📦 Plugin: {}", plugin.name);
            println!("   Path: {}", plugin.path);

            if let Some(system) = &plugin.system {
                println!("\n   ⚙️  System: {}", system.name);
                println!("      Module: {}", system.module_path);

                if !system.subscribes.is_empty() {
                    println!("\n      📨 Subscribes to:");
                    for event in &system.subscribes {
                        println!("         • {}", event);
                    }
                }

                if !system.publishes.is_empty() {
                    println!("\n      📤 Publishes:");
                    for event in &system.publishes {
                        println!("         • {}", event);
                    }
                }

                if !system.hooks.is_empty() {
                    println!("\n      🪝 Uses Hooks:");
                    for hook in &system.hooks {
                        println!("         • {}", hook);
                    }
                }
            }

            if !plugin.hook_details.is_empty() {
                println!("\n   🪝 Hook Traits:");
                for hook in &plugin.hook_details {
                    println!(
                        "      • {} ({} methods)",
                        hook.trait_name,
                        hook.methods.len()
                    );
                }
            }

            println!();
        }

        Ok(())
    }

    fn generate_event_flow_graph(&self, result: &AnalysisResult, config: &Config) -> Result<()> {
        println!("📈 Generating Event Flow Graph...");

        let graph_gen = EventFlowGraphGenerator::new(result);
        let mermaid = graph_gen.generate();

        let output_path = self
            .output
            .clone()
            .unwrap_or_else(|| config.output_dir_absolute().join("event_flow.mmd"));

        std::fs::write(&output_path, mermaid)?;

        println!("   ✅ Saved to: {}", output_path.display());
        println!("   View at: https://mermaid.live\n");

        Ok(())
    }

    fn generate_hook_flow_graph(&self, result: &AnalysisResult, config: &Config) -> Result<()> {
        println!("📈 Generating Hook Flow Graph...");

        let filter_plugins = if let Some(plugins) = &self.plugins {
            plugins.clone()
        } else {
            result
                .plugins
                .iter()
                .filter(|p| !p.hook_details.is_empty())
                .take(self.max_plugins)
                .map(|p| p.name.clone())
                .collect()
        };

        let options = GraphOptions {
            filter_plugins,
            ..Default::default()
        };

        let graph_gen = HookFlowGraphGenerator::with_options(result, options);
        let mermaid = graph_gen.generate();

        let output_path = self
            .output
            .clone()
            .unwrap_or_else(|| config.output_dir_absolute().join("hook_flow.mmd"));

        std::fs::write(&output_path, mermaid)?;

        println!("   ✅ Saved to: {}", output_path.display());
        println!("   View at: https://mermaid.live\n");

        Ok(())
    }

    fn generate_combined_flow_graph(&self, result: &AnalysisResult, config: &Config) -> Result<()> {
        println!("📈 Generating Combined Flow Graph...");

        let filter_plugins = if let Some(plugins) = &self.plugins {
            plugins.clone()
        } else {
            result
                .plugins
                .iter()
                .filter(|p| p.system.is_some())
                .take(self.max_plugins)
                .map(|p| p.name.clone())
                .collect()
        };

        let options = GraphOptions {
            filter_plugins,
            show_hooks: true,
            ..Default::default()
        };

        let graph_gen = CombinedFlowGraphGenerator::with_options(result, options);
        let mermaid = graph_gen.generate();

        let output_path = self
            .output
            .clone()
            .unwrap_or_else(|| config.output_dir_absolute().join("combined_flow.mmd"));

        std::fs::write(&output_path, mermaid)?;

        println!("   ✅ Saved to: {}", output_path.display());
        println!("   View at: https://mermaid.live\n");

        Ok(())
    }

    fn validate_event_flow(&self, result: &AnalysisResult) -> Result<()> {
        println!("🔎 Validating Event Flow...\n");

        let validator = Validator::new(result);
        let validation = validator.validate();

        validation.print_report();

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
        }

        println!();
        Ok(())
    }
}
