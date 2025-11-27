//! Ratatui UI rendering

use bevy::prelude::*;
use issun_bevy::plugins::{action::ActionPoints, contagion::*};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;

use crate::{
    events::EventLog,
    game_rules::{DefeatType, GameState, GameStats, VictoryType},
    player::{CureResearch, EmergencyBudget, Player},
    world::{get_total_population, CITIES},
};

pub type Tui = Terminal<CrosstermBackend<io::Stdout>>;

/// Initialize terminal
pub fn init_terminal() -> io::Result<Tui> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

/// Restore terminal
pub fn restore_terminal(terminal: &mut Tui) -> io::Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

/// Render frame from Bevy World
pub fn render_frame(terminal: &mut Tui, world: &World) -> io::Result<()> {
    let game_state = *world.resource::<GameState>();

    terminal.draw(|frame| match game_state {
        GameState::Playing => render_game_scene(frame, world),
        GameState::Victory(victory_type) => {
            let stats = world.resource::<GameStats>();
            render_victory_scene(frame, victory_type, stats)
        }
        GameState::Defeat(defeat_type) => {
            let stats = world.resource::<GameStats>();
            render_defeat_scene(frame, defeat_type, stats)
        }
    })?;

    Ok(())
}

/// Render game scene
fn render_game_scene(frame: &mut Frame, world: &World) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(7), // Global status
            Constraint::Length(5), // Player resources
            Constraint::Min(8),    // City status
            Constraint::Length(7), // Events
            Constraint::Length(11), // Actions
        ])
        .split(area);

    render_header(frame, chunks[0], world);
    render_global_status(frame, chunks[1], world);
    render_player_resources(frame, chunks[2], world);
    render_city_status(frame, chunks[3], world);
    render_events(frame, chunks[4], world);
    render_actions(frame, chunks[5], world);
}

/// Render header
fn render_header(frame: &mut Frame, area: Rect, world: &World) {
    let stats = world.resource::<GameStats>();

    let header = Paragraph::new(format!(
        "🦠 PANDEMIC CRISIS - Turn {} | [1-6]: Actions | [7]: End Turn | [Q]: Quit",
        stats.current_turn
    ))
    .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(header, area);
}

/// Render global status
fn render_global_status(frame: &mut Frame, area: Rect, world: &World) {
    let stats = world.resource::<GameStats>();
    let total_pop = get_total_population();
    let infection_rate = stats.infection_rate() * 100.0;
    let active_rate = stats.active_rate() * 100.0;

    let text = vec![
        Line::from(Span::styled(
            "📊 Global Status",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(format!("   Total Population:  {:>10}", format_number(total_pop))),
        Line::from(vec![
            Span::raw("   Infected:          "),
            Span::styled(
                format!("{:>10} ({:.1}%)", format_number(stats.total_infected), infection_rate),
                get_infection_color(infection_rate),
            ),
        ]),
        Line::from(vec![
            Span::raw("   Active Cases:      "),
            Span::styled(
                format!("{:>10} ({:.1}%)", format_number(stats.total_active), active_rate),
                get_infection_color(active_rate),
            ),
        ]),
        Line::from(format!("   Recovered:         {:>10}", format_number(stats.total_recovered))),
    ];

    let paragraph = Paragraph::new(text).block(Block::default().borders(Borders::ALL));
    frame.render_widget(paragraph, area);
}

/// Render player resources
fn render_player_resources(frame: &mut Frame, area: Rect, world: &World) {
    let player_entity = world.resource::<Player>().entity;
    let ap = world.get::<ActionPoints>(player_entity);
    let cure = world.resource::<CureResearch>();
    let _budget = world.resource::<EmergencyBudget>();
    let stats = world.resource::<GameStats>();

    let mut lines = vec![
        Line::from(Span::styled(
            "💰 Player Resources",
            Style::default().add_modifier(Modifier::BOLD),
        )),
    ];

    if let Some(ap) = ap {
        lines.push(Line::from(format!("   Action Points:     {}/15", ap.available)));
    }

    lines.push(Line::from(vec![
        Span::raw("   Cure Progress:     "),
        Span::styled(
            format!("{:.0}%", cure.progress * 100.0),
            if cure.progress >= 1.0 {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            },
        ),
    ]));

    if cure.deployed {
        if cure.deployment_complete(stats.current_turn) {
            lines.push(Line::from(Span::styled(
                "   Cure Status:       ✅ DEPLOYED",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            )));
        } else {
            let turns_left = cure.deployment_turn.unwrap() + 3 - stats.current_turn;
            lines.push(Line::from(format!(
                "   Cure Status:       🚀 Deploying ({} turns)",
                turns_left
            )));
        }
    }

    let paragraph = Paragraph::new(lines).block(Block::default().borders(Borders::ALL));
    frame.render_widget(paragraph, area);
}

/// Render city status
fn render_city_status(frame: &mut Frame, area: Rect, world: &World) {
    let stats = world.resource::<GameStats>();

    // Use stats data for display (simplified for now)
    // In a full implementation, we'd need proper query access
    let total_infected = stats.total_infected;
    let total_active = stats.total_active;
    let total_recovered = stats.total_recovered;

    // Show top cities with simplified data
    let mut city_infections: Vec<(String, usize, usize, usize)> = vec![];

    // For now, show overall stats as we don't have mutable query access
    if total_active > 0 || total_infected > 0 {
        city_infections.push((
            "Global".to_string(),
            total_active,
            total_infected.saturating_sub(total_active + total_recovered),
            total_recovered
        ));
    }

    city_infections.sort_by(|a, b| b.1.cmp(&a.1));

    let items: Vec<ListItem> = city_infections
        .iter()
        .take(5)
        .map(|(name, active, incubating, recovered)| {
            let total = active + incubating + recovered;
            let rate = (total as f32 / 1_000_000.0) * 100.0;
            let style = get_infection_color(rate);

            ListItem::new(format!(
                "  {:<12} Active: {:>6}  Incubating: {:>6}  Recovered: {:>6}",
                name, active, incubating, recovered
            ))
            .style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title("🌍 City Status (Top Infected)")
            .borders(Borders::ALL),
    );

    frame.render_widget(list, area);
}

/// Render events
fn render_events(frame: &mut Frame, area: Rect, world: &World) {
    let event_log = world.resource::<EventLog>();
    let recent_events = event_log.recent(5);

    let items: Vec<ListItem> = recent_events
        .iter()
        .map(|msg| ListItem::new(format!("  {}", msg)))
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title("📰 Recent Events")
            .borders(Borders::ALL),
    );

    frame.render_widget(list, area);
}

/// Render actions
fn render_actions(frame: &mut Frame, area: Rect, world: &World) {
    let player_entity = world.resource::<Player>().entity;
    let ap = world.get::<ActionPoints>(player_entity);

    let mut text = vec![
        Line::from(Span::styled(
            "🎮 Available Actions",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  [1] Quarantine City        (3 AP) - Block transmissions"),
        Line::from("  [2] Increase Awareness     (2 AP) - Boost resistance"),
        Line::from("  [3] Develop Cure Research  (5 AP) - Advance cure +10%"),
        Line::from("  [4] Emergency Healthcare   (4 AP) - Major boost (limited)"),
        Line::from("  [5] Travel Ban             (2 AP) - Reduce transmissions"),
        Line::from("  [6] Monitor City           (1 AP) - View details"),
        Line::from("  [7] End Turn               (0 AP) - Next turn"),
    ];

    if let Some(ap) = ap {
        text.push(Line::from(""));
        text.push(Line::from(vec![
            Span::raw("  Current AP: "),
            Span::styled(
                format!("{}/15", ap.available),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// Render victory scene
fn render_victory_scene(frame: &mut Frame, victory_type: VictoryType, stats: &GameStats) {
    let area = frame.area();

    let (title, message) = match victory_type {
        VictoryType::CureDeployed => (
            "🎉 VICTORY - Cure Deployed! 🎉",
            "You successfully developed and deployed the cure!\nThe pandemic has been contained through scientific triumph.",
        ),
        VictoryType::NaturalContainment => (
            "🎉 VICTORY - Natural Containment! 🎉",
            "Through careful management, the disease burned out naturally!\nHumanity survived through your strategic interventions.",
        ),
    };

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            title,
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(message),
        Line::from(""),
        Line::from(format!("Turns Survived:     {}", stats.current_turn)),
        Line::from(format!(
            "Total Infected:     {}",
            format_number(stats.total_infected)
        )),
        Line::from(format!(
            "Final Infection:    {:.1}%",
            stats.infection_rate() * 100.0
        )),
        Line::from(""),
        Line::from("Press ENTER to exit"),
    ];

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Green));

    frame.render_widget(paragraph, area);
}

/// Render defeat scene
fn render_defeat_scene(frame: &mut Frame, defeat_type: DefeatType, stats: &GameStats) {
    let area = frame.area();

    let (title, message) = match defeat_type {
        DefeatType::GlobalPandemic => (
            "☠️  DEFEAT - Global Pandemic ☠️",
            "The disease has spread to 70% of the global population.\nHealthcare systems have collapsed. Humanity faces extinction.",
        ),
        DefeatType::CriticalMutations => (
            "☠️  DEFEAT - Critical Mutations ☠️",
            "The disease has mutated into multiple critical strains.\nMedical responses are overwhelmed. Hope is lost.",
        ),
        DefeatType::EconomicCollapse => (
            "☠️  DEFEAT - Economic Collapse ☠️",
            "Too many cities under quarantine for too long.\nThe global economy has collapsed. Society breaks down.",
        ),
    };

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            title,
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(message),
        Line::from(""),
        Line::from(format!("Turns Survived:     {}", stats.current_turn)),
        Line::from(format!(
            "Total Infected:     {}",
            format_number(stats.total_infected)
        )),
        Line::from(format!(
            "Final Infection:    {:.1}%",
            stats.infection_rate() * 100.0
        )),
        Line::from(""),
        Line::from("Press ENTER to exit"),
    ];

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Red));

    frame.render_widget(paragraph, area);
}

// Helper functions

fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }
    result
}

fn get_infection_color(rate: f32) -> Style {
    if rate > 50.0 {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else if rate > 20.0 {
        Style::default().fg(Color::Yellow)
    } else if rate > 5.0 {
        Style::default().fg(Color::Blue)
    } else {
        Style::default().fg(Color::Green)
    }
}
