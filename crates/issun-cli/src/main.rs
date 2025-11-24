//! ISSUN CLI - Command-line interface for ISSUN game framework

mod commands;
mod config;
mod error;

use clap::{Parser, Subcommand};
use commands::AnalyzeCommand;
use config::Config;
use error::Result;

/// ISSUN - A mini game engine for logic-focused games
#[derive(Parser, Debug)]
#[command(name = "issun")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to project root directory
    #[arg(short = 'C', long, default_value = ".")]
    project_root: String,

    /// Plugin directory (relative to project root)
    #[arg(long, default_value = "crates/issun/src/plugin")]
    plugin_dir: String,

    /// Output directory for generated files
    #[arg(short, long, default_value = ".")]
    output: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Analyze plugin architecture and event flows
    Analyze(AnalyzeCommand),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Build configuration
    let config = Config::new()
        .with_project_root(&cli.project_root)
        .with_plugin_dir(&cli.plugin_dir)
        .with_output_dir(&cli.output);

    // Execute subcommand
    match &cli.command {
        Commands::Analyze(cmd) => cmd.execute(&config)?,
    }

    Ok(())
}
