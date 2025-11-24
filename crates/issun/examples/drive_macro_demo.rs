//! Demo of the drive! macro for simplified component rendering
//!
//! This example shows how to use the drive! and drive_to! macros
//! to simplify UI rendering with components.

#![allow(dead_code)]

use issun::context::ResourceContext;
use issun::ui::core::{Component, MultiResourceComponent};
use issun::ui::layer::{UILayer, UILayoutPresets};
use issun::ui::ratatui::{
    DistrictData, DistrictsComponent, DistrictsProvider, HeaderComponent, HeaderContext,
    LogComponent, LogProvider, RatatuiLayer,
};
use issun::{drive, drive_to};
use ratatui::{
    widgets::{Block, Borders, Paragraph},
    Frame,
};

// Game context implementation
#[derive(Debug, Clone)]
struct GameContext {
    turn: u32,
    max_turns: u32,
    mode: GameMode,
}

#[derive(Debug, Clone)]
enum GameMode {
    Normal,
    Hard,
}

impl HeaderContext for GameContext {
    fn turn(&self) -> u32 {
        self.turn
    }

    fn max_turns(&self) -> u32 {
        self.max_turns
    }

    fn mode(&self) -> String {
        format!("{:?}", self.mode)
    }
}

// District implementation
#[derive(Debug, Clone)]
struct District {
    id: String,
    name: String,
    population: u32,
}

impl DistrictData for District {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn format_line(&self) -> String {
        format!("{}: {} citizens", self.name, self.population)
    }
}

#[derive(Debug, Clone)]
struct CityMap {
    districts: Vec<District>,
}

impl DistrictsProvider for CityMap {
    type District = District;

    fn districts(&self) -> &[Self::District] {
        &self.districts
    }
}

// Log implementation
#[derive(Debug, Clone)]
struct GameLog {
    messages: Vec<String>,
}

impl LogProvider for GameLog {
    fn log_messages(&self) -> &[String] {
        &self.messages
    }
}

// Example 1: Using drive! with automatic layout
fn render_with_drive(frame: &mut Frame, resources: &ResourceContext, selected: usize) {
    let header = HeaderComponent::<GameContext>::new();
    let districts = DistrictsComponent::<CityMap>::new();
    let log = LogComponent::<GameLog>::with_title("Event Log | [Q] Quit");

    // Drive macro handles layout splitting and error handling automatically
    drive! {
        frame: frame,
        layout: RatatuiLayer::three_panel().apply(frame.area()),
        [
            header.render(resources),
            districts.render_with_selection(resources, selected),
            log.render_multi(resources),
        ]
    }
}

// Example 2: Using drive_to! for custom layouts
fn render_with_drive_to(frame: &mut Frame, resources: &ResourceContext) {
    let area = frame.area();

    // Custom layout: two equal columns
    let layout = RatatuiLayer::two_column(50).apply(area);

    let header = HeaderComponent::<GameContext>::new();
    let log = LogComponent::<GameLog>::new();

    // Drive_to allows explicit area assignment
    drive_to! {
        frame: frame,
        [
            (layout[0], header.render(resources)),
            (layout[1], log.render_multi(resources)),
        ]
    }
}

// Example 3: Using drive_to! with fallback widgets
fn render_with_fallback(frame: &mut Frame, resources: &ResourceContext) {
    let area = frame.area();
    let layout = RatatuiLayer::two_column(50).apply(area);

    let header = HeaderComponent::<GameContext>::new();

    // Provide fallback widget when component returns None
    let fallback = Paragraph::new("Loading...")
        .block(Block::default().borders(Borders::ALL).title("Status"));

    drive_to! {
        frame: frame,
        [
            (layout[0], header.render(resources), fallback),
        ]
    }
}

fn main() {
    println!("Drive Macro Demo");
    println!("================");
    println!();
    println!("This example demonstrates the drive! and drive_to! macros.");
    println!();
    println!("Example 1: drive! with automatic layout");
    println!("  - Creates layout automatically");
    println!("  - Renders components sequentially");
    println!("  - Handles missing resources gracefully");
    println!();
    println!("Example 2: drive_to! for custom layouts");
    println!("  - Explicit area assignment");
    println!("  - Full control over positioning");
    println!();
    println!("Example 3: drive_to! with fallback");
    println!("  - Provides fallback widgets");
    println!("  - Shows 'Loading...' when resources missing");
    println!();
    println!("Benefits:");
    println!("  ✓ Less boilerplate code");
    println!("  ✓ Automatic error handling");
    println!("  ✓ Cleaner, more readable rendering code");
    println!("  ✓ Type-safe component composition");
}
