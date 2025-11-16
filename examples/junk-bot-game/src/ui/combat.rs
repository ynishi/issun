//! Combat screen rendering

use crate::models::{GameContext, scenes::CombatSceneData};
use issun::ui::ratatui::GaugeWidget;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, List, ListItem},
};

pub fn render_combat(frame: &mut Frame, ctx: &GameContext, data: &CombatSceneData) {
    let area = frame.area();

    if data.show_inventory {
        // Show inventory overlay
        render_combat_with_inventory(frame, area, ctx, data);
    } else {
        // Normal combat view
        render_normal_combat(frame, area, ctx, data);
    }
}

fn render_normal_combat(frame: &mut Frame, area: Rect, ctx: &GameContext, data: &CombatSceneData) {
    // Main layout: [Floor info | Party status | Enemy status | Combat log | Controls]
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Floor info
            Constraint::Min(8),     // Party status (Player + Bots)
            Constraint::Min(8),     // Enemy status
            Constraint::Length(8),  // Combat log
            Constraint::Length(2),  // Controls
        ])
        .split(area);

    render_floor_info(frame, chunks[0], ctx);
    render_party_status(frame, chunks[1], ctx);
    render_enemies(frame, chunks[2], data);
    render_combat_log(frame, chunks[3], data);
    render_controls(frame, chunks[4]);
}

fn render_combat_with_inventory(frame: &mut Frame, area: Rect, ctx: &GameContext, data: &CombatSceneData) {
    // Split: [Combat info (left) | Inventory (right)]
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);

    // Left side: Party + Enemies
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(h_chunks[0]);

    render_party_status(frame, left_chunks[0], ctx);
    render_enemies(frame, left_chunks[1], data);

    // Right side: Inventory
    render_inventory(frame, h_chunks[1], ctx, data);
}

fn render_inventory(frame: &mut Frame, area: Rect, ctx: &GameContext, data: &CombatSceneData) {
    let block = Block::default()
        .title(format!("Inventory - Equip to: {:?}", data.equip_target))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if ctx.inventory.is_empty() {
        let msg = Paragraph::new("No weapons in inventory")
            .alignment(Alignment::Center);
        frame.render_widget(msg, inner);
        return;
    }

    let items: Vec<ListItem> = ctx
        .inventory
        .iter()
        .enumerate()
        .map(|(i, weapon)| {
            let prefix = if i == data.inventory_cursor { "> " } else { "  " };
            let line = Line::from(vec![
                Span::raw(prefix),
                Span::styled(&weapon.name, Style::default().fg(Color::Yellow)),
                Span::raw(format!(" [ATK: {}] ", weapon.attack)),
                Span::styled(weapon.display(), Style::default().fg(Color::Cyan)),
            ]);

            if i == data.inventory_cursor {
                ListItem::new(line).style(Style::default().add_modifier(Modifier::BOLD))
            } else {
                ListItem::new(line)
            }
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}

fn render_floor_info(frame: &mut Frame, area: Rect, ctx: &GameContext) {
    if let Some(dungeon) = ctx.get_dungeon() {
        let floor_text = format!(
            "FLOOR {}/{}  |  Score: {}",
            dungeon.current_floor_number(),
            dungeon.total_floors(),
            ctx.score
        );

        let floor_info = Paragraph::new(floor_text)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);

        frame.render_widget(floor_info, area);
    }
}

fn render_party_status(frame: &mut Frame, area: Rect, ctx: &GameContext) {
    let block = Block::default()
        .title("Your Party")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Calculate how many members to show (player + alive bots)
    let alive_bots_count = ctx.bots.iter().filter(|b| b.is_alive()).count();
    let total_members = 1 + alive_bots_count; // Player + bots
    let member_height = 3; // Lines per member

    // Split area for each party member
    let constraints: Vec<Constraint> = (0..total_members)
        .map(|_| Constraint::Length(member_height))
        .collect();

    let member_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints(constraints)
        .split(inner);

    // Render player
    render_character_status(frame, member_chunks[0], &ctx.player.name, ctx.player.hp, ctx.player.max_hp,
                          ctx.player.attack, Some(ctx.player.defense), &ctx.player.equipped_weapon.display());

    // Render bots
    let mut chunk_idx = 1;
    for bot in ctx.bots.iter().filter(|b| b.is_alive()) {
        if chunk_idx < member_chunks.len() {
            render_character_status(frame, member_chunks[chunk_idx], &bot.name, bot.hp, bot.max_hp,
                                  bot.attack, None, &bot.equipped_weapon.display());
            chunk_idx += 1;
        }
    }
}

fn render_character_status(
    frame: &mut Frame,
    area: Rect,
    name: &str,
    hp: i32,
    max_hp: i32,
    attack: i32,
    defense: Option<i32>,
    weapon: &str,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

    // Name and weapon
    let name_line = Line::from(vec![
        Span::styled(name, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(weapon, Style::default().fg(Color::Cyan)),
    ]);
    frame.render_widget(Paragraph::new(name_line), chunks[0]);

    // HP bar
    let hp_ratio = hp as f64 / max_hp as f64;
    let hp_gauge = GaugeWidget::new()
        .with_ratio(hp_ratio)
        .with_label(format!("HP: {}/{}", hp, max_hp))
        .with_auto_color(true);
    hp_gauge.render(frame, chunks[1]);

    // Stats
    let stats_line = if let Some(def) = defense {
        Line::from(vec![
            Span::raw("ATK: "),
            Span::styled(attack.to_string(), Style::default().fg(Color::Red)),
            Span::raw(" | DEF: "),
            Span::styled(def.to_string(), Style::default().fg(Color::Blue)),
        ])
    } else {
        Line::from(vec![
            Span::raw("ATK: "),
            Span::styled(attack.to_string(), Style::default().fg(Color::Red)),
        ])
    };
    frame.render_widget(Paragraph::new(stats_line), chunks[2]);
}

fn render_enemies(frame: &mut Frame, area: Rect, data: &CombatSceneData) {
    // Create title with room buff info
    let title = format!("Enemies {} {} {}",
        data.room_buff.icon(),
        data.room_buff.name(),
        data.room_buff.icon()
    );

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(data.room_buff.color()));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Render each enemy
    if data.enemies.is_empty() {
        let msg = Paragraph::new("All enemies defeated!")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Green));
        frame.render_widget(msg, inner);
        return;
    }

    let enemy_height = 3;

    for (i, enemy) in data.enemies.iter().enumerate() {
        if !enemy.is_alive() {
            continue;
        }

        let y = inner.y + (i as u16 * enemy_height);
        if y + enemy_height > inner.y + inner.height {
            break; // Out of bounds
        }

        let enemy_area = Rect {
            x: inner.x,
            y,
            width: inner.width,
            height: enemy_height,
        };

        render_enemy(frame, enemy_area, enemy);
    }
}

fn render_enemy(frame: &mut Frame, area: Rect, enemy: &crate::models::entities::Enemy) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

    // Name and attack
    let name_line = Line::from(vec![
        Span::styled(&enemy.name, Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::raw(" [ATK: "),
        Span::styled(enemy.attack.to_string(), Style::default().fg(Color::Red)),
        Span::raw("]"),
    ]);
    frame.render_widget(Paragraph::new(name_line), chunks[0]);

    // HP bar
    let hp_ratio = enemy.hp as f64 / enemy.max_hp as f64;
    let hp_gauge = GaugeWidget::new()
        .with_ratio(hp_ratio)
        .with_label(format!("HP: {}/{}", enemy.hp, enemy.max_hp))
        .with_auto_color(true);
    hp_gauge.render(frame, chunks[1]);
}

fn render_combat_log(frame: &mut Frame, area: Rect, data: &CombatSceneData) {
    let block = Block::default()
        .title("Combat Log")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    // Take last N messages that fit in the area
    let max_lines = (area.height.saturating_sub(2)) as usize;
    let start_index = data.combat_log.len().saturating_sub(max_lines);
    let messages: Vec<ListItem> = data.combat_log[start_index..]
        .iter()
        .map(|msg| ListItem::new(msg.as_str()))
        .collect();

    let list = List::new(messages).block(block);
    frame.render_widget(list, area);
}

fn render_controls(frame: &mut Frame, area: Rect) {
    let controls = Paragraph::new("Space: Attack | I: Inventory | Tab: Change Target | Q: Quit")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(controls, area);
}
