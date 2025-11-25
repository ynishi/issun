//! UI rendering for RPG Arena

use crate::arena::Arena;
use crate::combat_state::CombatState;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

/// Render the main arena UI
pub fn render(frame: &mut Frame, arena: &Arena, loaded_mods: &[String]) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(7),  // Fighters
            Constraint::Length(8),  // Inventory
            Constraint::Min(8),     // Combat log
            Constraint::Length(6),  // Config info
            Constraint::Length(3),  // Controls
        ])
        .split(frame.area());

    render_title(frame, chunks[0]);
    render_fighters(frame, chunks[1], arena);
    render_inventory(frame, chunks[2], arena);
    render_combat_log(frame, chunks[3], arena);
    render_config_info(frame, chunks[4], arena, loaded_mods);
    render_controls(frame, chunks[5], &arena.combat.state);
}

fn render_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new("⚔️  RPG Arena - MOD System E2E Test")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, area);
}

fn render_fighters(frame: &mut Frame, area: Rect, arena: &Arena) {
    let player = &arena.player;
    let enemy = &arena.enemy;

    let hp_bar_width = 20;

    let content = vec![
        Line::from(vec![
            Span::styled(
                format!("{} HP: {}/{} ", player.name, player.current_hp, player.max_hp),
                Style::default().fg(Color::Green),
            ),
            Span::styled(
                player.hp_bar(hp_bar_width),
                Style::default().fg(if player.hp_percentage() > 0.5 {
                    Color::Green
                } else if player.hp_percentage() > 0.2 {
                    Color::Yellow
                } else {
                    Color::Red
                }),
            ),
        ]),
        Line::from(format!("   ATK: {}", player.base_attack)),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("{} HP: {}/{} ", enemy.name, enemy.current_hp, enemy.max_hp),
                Style::default().fg(Color::Red),
            ),
            Span::styled(
                enemy.hp_bar(hp_bar_width),
                Style::default().fg(if enemy.hp_percentage() > 0.5 {
                    Color::Red
                } else if enemy.hp_percentage() > 0.2 {
                    Color::Yellow
                } else {
                    Color::DarkGray
                }),
            ),
        ]),
        Line::from(format!("   ATK: {}", enemy.base_attack)),
    ];

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("⚔️ Fighters"));
    frame.render_widget(paragraph, area);
}

fn render_inventory(frame: &mut Frame, area: Rect, arena: &Arena) {
    let inv = &arena.inventory;

    let items: Vec<ListItem> = inv
        .items()
        .iter()
        .enumerate()
        .map(|(i, (item, count))| {
            let text = if *count > 1 {
                format!("[{}] {} {} x{}", i, item.icon(), item.name, count)
            } else {
                format!("[{}] {} {}", i, item.icon(), item.name)
            };
            ListItem::new(text)
        })
        .collect();

    let title = format!(
        "🎒 Inventory [{}/{}] {}",
        inv.item_count(),
        if inv.max_slots == 0 {
            "∞".to_string()
        } else {
            inv.max_slots.to_string()
        },
        if inv.allow_stacking {
            "(stacking)"
        } else {
            "(no stack)"
        }
    );

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(title));
    frame.render_widget(list, area);
}

fn render_combat_log(frame: &mut Frame, area: Rect, arena: &Arena) {
    let logs: Vec<ListItem> = arena
        .combat
        .recent_logs(10)
        .iter()
        .map(|entry| {
            let text = format!("[Turn {}] {}", entry.turn, entry.message);
            ListItem::new(text)
        })
        .collect();

    let status_text = match arena.combat.state {
        CombatState::Idle => "Press [N] to start new combat",
        CombatState::PlayerTurn => "⏳ Your turn",
        CombatState::EnemyTurn => "⏳ Enemy's turn...",
        CombatState::Victory => "🎉 Victory! Press [N] for new combat",
        CombatState::Defeat => "💀 Defeat... Press [N] for new combat",
    };

    let title = format!("📜 Combat Log - {}", status_text);
    let list = List::new(logs).block(Block::default().borders(Borders::ALL).title(title));
    frame.render_widget(list, area);
}

fn render_config_info(frame: &mut Frame, area: Rect, arena: &Arena, loaded_mods: &[String]) {
    let mod_list = if loaded_mods.is_empty() {
        "None".to_string()
    } else {
        loaded_mods.join(", ")
    };

    let content = vec![
        Line::from(format!("🔧 Active MODs: {}", mod_list)),
        Line::from(""),
        Line::from(format!(
            "⚙️  Combat Settings: Max HP={}, Difficulty={:.1}x",
            arena.player.max_hp, arena.difficulty_multiplier
        )),
        Line::from(format!(
            "⚙️  Inventory Settings: Max Slots={}, Stacking={}",
            if arena.inventory.max_slots == 0 {
                "∞".to_string()
            } else {
                arena.inventory.max_slots.to_string()
            },
            arena.inventory.allow_stacking
        )),
    ];

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("⚙️ Configuration"))
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn render_controls(frame: &mut Frame, area: Rect, state: &CombatState) {
    let controls = match state {
        CombatState::Idle | CombatState::Victory | CombatState::Defeat => {
            "[N] New Combat | [M] Load MOD | [U] Unload MOD | [Q] Quit"
        }
        CombatState::PlayerTurn => {
            "[SPACE] Attack | [0-9] Use Item | [M] Load MOD | [Q] Quit"
        }
        CombatState::EnemyTurn => "[Please wait...] | [Q] Quit",
    };

    let paragraph = Paragraph::new(controls)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Controls"));
    frame.render_widget(paragraph, area);
}
