use crate::models::scenes::{
    EconomicSceneData, IntelReportSceneData, StrategySceneData, TacticalSceneData, TitleSceneData,
    VaultSceneData,
};
use crate::models::{BudgetChannel, GameContext, SlotEffect, SlotType, VaultOutcome, VaultStatus};
use crate::plugins::{
    EconomyState, FactionOpsState, MarketPulse, PrototypeBacklog, ReputationLedger,
    TerritoryStateCache, VaultState,
};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

pub fn render_title(frame: &mut Frame, data: &TitleSceneData) {
    let area = centered(frame.area(), 60, 16);
    let block = Block::default()
        .title("Border Economy")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    frame.render_widget(block, area);
    let lines: Vec<ListItem> = data
        .options
        .iter()
        .enumerate()
        .map(|(idx, label)| {
            let style = if idx == data.selected_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            ListItem::new(Span::styled(label.clone(), style))
        })
        .collect();

    frame.render_widget(List::new(lines), inner);
}

pub fn render_strategy(
    frame: &mut Frame,
    ctx: Option<&GameContext>,
    clock: Option<&issun::plugin::GameTimer>,
    ledger: Option<&issun::plugin::BudgetLedger>,
    ops: Option<&FactionOpsState>,
    territory: Option<&TerritoryStateCache>,
    reputation: Option<&ReputationLedger>,
    points: Option<&issun::plugin::ActionPoints>,
    data: &StrategySceneData,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(frame.area());

    let list_items: Vec<ListItem> = data
        .actions
        .iter()
        .enumerate()
        .map(|(idx, action)| {
            let label = match action {
                crate::models::scenes::StrategyAction::DeployOperation => "作戦展開",
                crate::models::scenes::StrategyAction::FundResearch => "R&D投資",
                crate::models::scenes::StrategyAction::InspectIntel => "状況報告",
                crate::models::scenes::StrategyAction::ManageBudget => "資金配分",
                crate::models::scenes::StrategyAction::InvestDevelopment => "開拓投資",
                crate::models::scenes::StrategyAction::DiplomaticAction => "外交行動",
                crate::models::scenes::StrategyAction::SetPolicy => "政策切替",
                crate::models::scenes::StrategyAction::FortifyFront => "前線強化",
                crate::models::scenes::StrategyAction::ManageVaults => "Vault投資",
                crate::models::scenes::StrategyAction::EndDay => "日次終了",
            };
            let style = if idx == data.cursor {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            ListItem::new(Span::styled(label, style))
        })
        .collect();

    let command_block = Block::default()
        .title("Strategic Command")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));
    frame.render_widget(command_block.clone(), chunks[0]);
    let inner = command_block.inner(chunks[0]);
    frame.render_widget(List::new(list_items), inner);

    render_hq_feed(frame, chunks[1], ctx, clock, ledger, ops, territory, reputation, points);

    let mut status_lines = vec![Line::from(data.status_line.clone())];
    if let Some(points) = points {
        status_lines.push(Line::from(format!(
            "Actions remaining: {}/{}",
            points.available, points.max_per_period
        )));
    }
    status_lines.push(Line::from("Enter: select   Esc: Title"));
    let status = Paragraph::new(status_lines)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title("Status"));
    let status_area = Rect {
        x: chunks[0].x,
        y: chunks[0].y + chunks[0].height.saturating_sub(5),
        width: chunks[0].width,
        height: 5,
    };
    frame.render_widget(status, status_area);
}

fn render_hq_feed(
    frame: &mut Frame,
    area: Rect,
    ctx: Option<&GameContext>,
    clock: Option<&issun::plugin::GameTimer>,
    ledger: Option<&issun::plugin::BudgetLedger>,
    ops: Option<&FactionOpsState>,
    territory: Option<&TerritoryStateCache>,
    reputation: Option<&ReputationLedger>,
    points: Option<&issun::plugin::ActionPoints>,
) {
    let block = Block::default().title("HQ Feed").borders(Borders::ALL);
    frame.render_widget(block.clone(), area);
    let inner = block.inner(area);
    let mut lines = Vec::new();

    if let Some(ctx) = ctx {
        let day_text = clock.map(|c| c.day).unwrap_or(ctx.day);
        let (remaining, total) = points
            .map(|p| (p.available, p.max_per_period))
            .unwrap_or((0, 0));
        lines.push(Line::from(vec![Span::styled(
            format!("Day {} | AP {}/{}", day_text, remaining, total),
            Style::default().fg(Color::Yellow),
        )]));

        if let Some(ledger) = ledger {
            lines.push(Line::from(format!("Cash {}", ledger.cash)));
            lines.push(Line::from(format!(
                "Ops {} | R&D {} | Reserve {}",
                ledger.ops_pool, ledger.research_pool, ledger.reserve
            )));
            lines.push(Line::from(format!(
                "Innovation {} | Security {}",
                ledger.innovation_fund, ledger.security_fund
            )));
        }
        lines.push(Line::from(format!("政策: {}", ctx.active_policy().name)));
        if let Some(next_op) = ctx.enemy_operations.iter().min_by_key(|op| op.eta) {
            let faction = ctx
                .enemy_faction_by_id(&next_op.faction)
                .map(|f| f.codename.as_str())
                .unwrap_or("未知");
            lines.push(Line::from(format!(
                "敵作戦予告: {} [{}] ETA {}日",
                next_op.territory.as_str(),
                faction,
                next_op.eta
            )));
        }
        if let Some(enemy_note) = &ctx.last_enemy_action {
            lines.push(Line::from(vec![Span::styled(
                enemy_note.as_str(),
                Style::default().fg(Color::Red),
            )]));
        }
        lines.push(Line::from("Recent:"));
        lines.extend(
            ctx.recent_log
                .iter()
                .map(|entry| Line::from(entry.as_str())),
        );
    } else {
        lines.push(Line::from("HQ telemetry unavailable"));
    }

    if let Some(ops) = ops {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "Operations",
            Style::default().fg(Color::Cyan),
        )]));
        lines.push(Line::from(format!(
            "Sorties {} | Casualties {}",
            ops.sorties_launched, ops.total_casualties
        )));
        if !ops.active_operations.is_empty() {
            let active = ops
                .active_operations
                .iter()
                .map(|t| t.as_str().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            lines.push(Line::from(format!("Active: {}", active)));
        }
        for report in ops.recent_reports.iter().take(3) {
            lines.push(Line::from(format!("• {}", report)));
        }
    }

    if let Some(territory) = territory {
        if !territory.updates.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "Territory Intel",
                Style::default().fg(Color::LightGreen),
            )]));
            for update in territory.updates.iter().take(3) {
                lines.push(Line::from(format!("• {}", update)));
            }
        }
        if let Some(front) = &territory.active_front {
            let faction = territory
                .active_front_faction
                .as_deref()
                .unwrap_or("Unknown");
            lines.push(Line::from(vec![Span::styled(
                format!("Battlefront: {} [{}]", front, faction),
                Style::default().fg(Color::LightRed),
            )]));
        }
    }

    if let Some(reputation) = reputation {
        if !reputation.events.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "Reputation",
                Style::default().fg(Color::Magenta),
            )]));
            for event in reputation.events.iter().take(3) {
                lines.push(Line::from(format!("• {}", event)));
            }
        }
    }

    if let Some(territory) = territory {
        if !territory.faction_reports.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "Enemy Factions",
                Style::default().fg(Color::Red),
            )]));
            for report in territory.faction_reports.iter().take(3) {
                lines.push(Line::from(format!("• {}", report)));
            }
        }
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, inner);
}

pub fn render_tactical(frame: &mut Frame, ctx: Option<&GameContext>, data: &TacticalSceneData) {
    let area = frame.area();
    let block = Block::default().title("Tactical Ops").borders(Borders::ALL);
    frame.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let summary = vec![
        Line::from(format!("Faction: {}", data.brief.faction)),
        Line::from(format!("Target: {}", data.brief.target)),
        Line::from(format!("Prototype: {}", data.brief.prototype)),
        Line::from(format!("Progress {:>3.0}%", data.progress * 100.0)),
        Line::from(format!("Payout {}", data.brief.expected_payout)),
    ];
    let mut paragraph = Paragraph::new(summary)
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(Color::White));
    paragraph = paragraph.block(
        Block::default()
            .borders(Borders::ALL)
            .title("Mission Brief"),
    );
    frame.render_widget(paragraph, inner);

    if let Some(ctx) = ctx {
        let intel = ctx
            .territory_snapshot(&data.brief.target)
            .map(|t| {
                format!(
                    "支配率 {:>3.0}% / 不安 {:>3.0}%",
                    t.control * 100.0,
                    t.unrest * 100.0
                )
            })
            .unwrap_or_else(|| "データなし".into());
        let note = Paragraph::new(intel)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().title("Intel").borders(Borders::ALL));
        let intel_area = Rect {
            x: inner.x,
            y: inner.y + inner.height.saturating_sub(5),
            width: inner.width,
            height: 5,
        };
        frame.render_widget(note, intel_area);
    }
}

pub fn render_economic(
    frame: &mut Frame,
    ctx: Option<&GameContext>,
    clock: Option<&issun::plugin::GameTimer>,
    ledger: Option<&issun::plugin::BudgetLedger>,
    economy: Option<&EconomyState>,
    prototypes: Option<&PrototypeBacklog>,
    market: Option<&MarketPulse>,
    data: &EconomicSceneData,
) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(frame.area());

    let channels = data
        .channels
        .iter()
        .enumerate()
        .map(|(idx, channel)| {
            let label = format!("{} ⟶ {}", channel, ctx_value(ctx, ledger, channel));
            let style = if idx == data.cursor {
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            ListItem::new(Span::styled(label, style))
        })
        .collect::<Vec<_>>();

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Length(5)])
        .split(layout[0]);
    let list = List::new(channels).block(
        Block::default()
            .title("Budget Channels")
            .borders(Borders::ALL),
    );
    frame.render_widget(list, left_chunks[0]);

    let amount = data.amount_options[data.amount_cursor];
    let mode_label = if data.diplomacy_mode {
        "外交モード (Gで解除) / F: 外交投資"
    } else {
        "予算モード / F: Cash→選択"
    };
    let info_lines = vec![
        Line::from(format!("投入額: {} (A/Dで切替)", amount)),
        Line::from(mode_label),
        Line::from("←→: Reserve⇄Channel | Enter: 日次締め"),
    ];
    let info = Paragraph::new(info_lines).wrap(Wrap { trim: true }).block(
        Block::default()
            .title("Controls")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Gray)),
    );
    frame.render_widget(info, left_chunks[1]);

    render_econ_sidebar(frame, layout[1], ctx, clock, ledger, economy, prototypes, market, data);
}

fn render_econ_sidebar(
    frame: &mut Frame,
    area: Rect,
    ctx: Option<&GameContext>,
    clock: Option<&issun::plugin::GameTimer>,
    ledger: Option<&issun::plugin::BudgetLedger>,
    economy: Option<&EconomyState>,
    prototypes: Option<&PrototypeBacklog>,
    market: Option<&MarketPulse>,
    data: &EconomicSceneData,
) {
    let block = Block::default()
        .title("Capital Board")
        .borders(Borders::ALL);
    frame.render_widget(block.clone(), area);
    let inner = block.inner(area);
    let mut lines = Vec::new();

    if let Some(ctx) = ctx {
        if let Some(ledger) = ledger {
            lines.push(Line::from(format!("Cash {}", ledger.cash)));
            lines.push(Line::from(format!(
                "Reserve {} | Ops {} | R&D {}",
                ledger.reserve, ledger.ops_pool, ledger.research_pool
            )));
            lines.push(Line::from(format!(
                "Innovation {} | Security {}",
                ledger.innovation_fund, ledger.security_fund
            )));
            // Border-economy specific bonus calculations
            let rd_bonus = (ledger.innovation_fund.amount() as f32 / 2000.0).clamp(0.0, 0.35) * 100.0;
            if rd_bonus > 0.0 {
                lines.push(Line::from(format!("R&D bonus +{:.0}%", rd_bonus)));
            }
            let upkeep_cut = (ledger.security_fund.amount() as f32 * 0.08) as i64;
            if upkeep_cut > 0 {
                lines.push(Line::from(format!("Upkeep reduction ₡{}", upkeep_cut)));
            }
        }
        lines.push(Line::from(""));
        lines.push(Line::from("Development:"));
        for territory in ctx.territories.iter().take(3) {
            lines.push(Line::from(format!(
                "{} L{:.0}% Tier{} (pending {:.0}%)",
                territory.id.as_str(),
                territory.development_level * 100.0,
                territory.market_tier,
                territory.pending_investment * 100.0
            )));
        }
        lines.push(Line::from(format!(
            "政策: {} ({})",
            ctx.active_policy().name,
            ctx.active_policy().description
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(format!("Last shift: {}", data.last_transfer)));
    if let Some(origin) = &data.origin_story {
        lines.push(Line::from(format!("Origin: {}", origin)));
    }

    if let Some(econ) = economy {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "Economy",
            Style::default().fg(Color::Yellow),
        )]));
        lines.push(Line::from(format!(
            "Pending ops {} | Last cashflow {}",
            econ.pending_operations, econ.last_cashflow
        )));
        if !econ.rolling_income.is_empty() {
            let avg =
                econ.rolling_income.iter().sum::<i64>() as f32 / econ.rolling_income.len() as f32;
            lines.push(Line::from(format!("Avg income {:.0}", avg)));
        }
        for job in econ.research_backlog.iter().take(3) {
            lines.push(Line::from(format!("R&D: {}", job)));
        }
        if !econ.settlement_log.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "Settlements",
                Style::default().fg(Color::Green),
            )]));
            for entry in econ.settlement_log.iter().take(3) {
                lines.push(Line::from(format!("• {}", entry)));
            }
        }
        if let Some(kpi) = &econ.last_kpi {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "Weekly KPIs",
                Style::default().fg(Color::Cyan),
            )]));
            lines.push(Line::from(format!(
                "Income {} | Upkeep {} | Net {} ({:.0}%)",
                kpi.income,
                kpi.upkeep,
                kpi.net,
                kpi.net_margin * 100.0
            )));
            lines.push(Line::from(format!(
                "Ops {} | R&D {} | Dev {}",
                kpi.ops_spent, kpi.rnd_spent, kpi.dev_spent
            )));
        }
        if !econ.warnings.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "Warnings",
                Style::default().fg(Color::Red),
            )]));
            for warning in econ.warnings.iter().take(3) {
                lines.push(Line::from(format!("! {}", warning)));
            }
        }
        if !econ.vault_reports.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "Vault Reports",
                Style::default().fg(Color::LightYellow),
            )]));
            for entry in econ.vault_reports.iter().take(3) {
                lines.push(Line::from(format!("• {}", entry)));
            }
        }
    }

    if let Some(proto) = prototypes {
        if !proto.queued.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "Prototype Queue",
                Style::default().fg(Color::LightMagenta),
            )]));
            for order in proto.queued.iter().take(3) {
                lines.push(Line::from(format!("• {}", order)));
            }
        }
        if !proto.field_reports.is_empty() {
            lines.push(Line::from("Field Reports:"));
            for report in proto.field_reports.iter().take(2) {
                lines.push(Line::from(format!("  {}", report)));
            }
        }
    }

    if let Some(market) = market {
        if let Some(last) = market.snapshots.last() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "Market Pulse",
                Style::default().fg(Color::LightCyan),
            )]));
            lines.push(Line::from(format!("Share {:.1}%", last * 100.0)));
        }
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, inner);
}

pub fn render_vault(
    frame: &mut Frame,
    ctx: Option<&GameContext>,
    clock: Option<&issun::plugin::GameTimer>,
    ledger: Option<&issun::plugin::BudgetLedger>,
    vault_state: Option<&VaultState>,
    data: &VaultSceneData,
) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(frame.area());

    let block = Block::default().title("Vault Sites").borders(Borders::ALL);
    frame.render_widget(block.clone(), layout[0]);
    let inner = block.inner(layout[0]);

    let list_items = if let Some(ctx) = ctx {
        if ctx.vaults().is_empty() {
            vec![ListItem::new("Vault未発見")]
        } else {
            ctx.vaults()
                .iter()
                .enumerate()
                .map(|(idx, vault)| {
                    let status = match vault.status {
                        VaultStatus::Active => "ACTIVE",
                        VaultStatus::Peril { .. } => "PERIL",
                        VaultStatus::Captured { .. } => "CAPTURED",
                    };
                    let active_slots = vault.slots.iter().filter(|s| s.active).count();
                    let line = format!(
                        "{} [{}] slots {}/{}  SecReq {}",
                        vault.codename,
                        status,
                        active_slots,
                        vault.slots.len(),
                        vault.security_requirement
                    );
                    let style = if idx == data.vault_cursor {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    };
                    ListItem::new(Span::styled(line, style))
                })
                .collect::<Vec<_>>()
        }
    } else {
        vec![ListItem::new("Vault telemetry unavailable")]
    };
    frame.render_widget(List::new(list_items), inner);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(layout[1]);

    let detail_block = Block::default().title("Slot Detail").borders(Borders::ALL);
    frame.render_widget(detail_block.clone(), right[0]);
    let detail_area = detail_block.inner(right[0]);

    let mut detail_lines = Vec::new();
    if let Some(ctx) = ctx {
        let vaults = ctx.vaults();
        if vaults.is_empty() {
            detail_lines.push(Line::from("Vault未発見"));
        } else {
            let index = data.vault_cursor.min(vaults.len() - 1);
            let vault = &vaults[index];
            detail_lines.push(Line::from(vec![Span::styled(
                format!(
                    "{} | Volatility {:.1} | Defense {}w",
                    vault.codename, vault.volatility, vault.defense_window_weeks
                ),
                Style::default().fg(Color::Cyan),
            )]));
            detail_lines.push(Line::from(format!(
                "Security req {} | 状態: {:?}",
                vault.security_requirement, vault.status
            )));
            if let Some(report) = vault_state
                .and_then(|state| state.latest_reports.iter().find(|r| r.vault_id == vault.id))
            {
                if let Some(outcome) = &report.outcome {
                    detail_lines.push(Line::from(format!(
                        "Last Outcome: {}",
                        describe_outcome(outcome)
                    )));
                }
                if let Some(assault) = &report.assault_log {
                    detail_lines.push(Line::from(format!("Last Assault: {}", assault)));
                }
            }
            detail_lines.push(Line::from(""));
            for (slot_idx, slot) in vault.slots.iter().enumerate() {
                let selector = if slot_idx == data.slot_cursor {
                    ">"
                } else {
                    " "
                };
                detail_lines.push(Line::from(format!(
                    "{} [{}] {}: {}/{} (Decay {:.0}%/wk)",
                    selector,
                    format_slot_type(&slot.slot_type),
                    slot.name,
                    slot.current_investment,
                    slot.base_threshold,
                    slot.decay_rate * 100.0
                )));
                let effect_line = match &slot.effect {
                    SlotEffect::RnDBuff {
                        progress_bonus,
                        telemetry_bonus,
                    } => format!(
                        "    R&D +{:.0}% / Telemetry +{:.0}%",
                        progress_bonus * 100.0,
                        telemetry_bonus * 100.0
                    ),
                    SlotEffect::OpsRelief {
                        hostility_drop,
                        ops_cost_multiplier,
                    } => format!(
                        "    Hostility -{:.0}% / Ops x{:.2}",
                        hostility_drop * 100.0,
                        ops_cost_multiplier
                    ),
                    SlotEffect::SpecialCard { card_id } => {
                        format!("    Special card {}", card_id)
                    }
                };
                detail_lines.push(Line::from(effect_line));
            }
        }
    } else {
        detail_lines.push(Line::from("Context not available"));
    }
    detail_lines.push(Line::from(""));
    detail_lines.push(Line::from(format!(
        "投入額: {} (A/Dで切替) | Enter: 投資 | Esc: 戻る",
        data.amount_options[data.amount_cursor]
    )));
    detail_lines.push(Line::from(data.status_line.clone()));
    frame.render_widget(
        Paragraph::new(detail_lines).wrap(Wrap { trim: true }),
        detail_area,
    );

    let alert_block = Block::default().title("Vault Alerts").borders(Borders::ALL);
    frame.render_widget(alert_block.clone(), right[1]);
    let alert_area = alert_block.inner(right[1]);
    let mut alerts = Vec::new();
    if let Some(state) = vault_state {
        for alert in state.alerts.iter().take(6) {
            alerts.push(Line::from(format!("• {}", alert)));
        }
        if !state.latest_reports.is_empty() {
            alerts.push(Line::from(""));
            alerts.push(Line::from("Latest Reports:"));
            for report in state.latest_reports.iter().take(2) {
                let outcome_text = report
                    .outcome
                    .as_ref()
                    .map(describe_outcome)
                    .unwrap_or_else(|| "未報告".into());
                alerts.push(Line::from(format!(
                    "{}: {} / 投資 {} / 減耗 {}",
                    report.codename, outcome_text, report.total_investment, report.decay_applied
                )));
                if let Some(assault) = &report.assault_log {
                    alerts.push(Line::from(format!("   Assault: {}", assault)));
                }
            }
        }
    } else {
        alerts.push(Line::from("Vault state offline"));
    }
    frame.render_widget(Paragraph::new(alerts).wrap(Wrap { trim: true }), alert_area);
}

fn format_slot_type(slot_type: &SlotType) -> &'static str {
    match slot_type {
        SlotType::Research => "R&D",
        SlotType::Security => "SEC",
        SlotType::Special => "SPC",
    }
}

fn describe_outcome(outcome: &VaultOutcome) -> String {
    match outcome {
        VaultOutcome::Jackpot { credits, .. } => format!("Jackpot +{}", credits),
        VaultOutcome::Success { credits } => format!("Success +{}", credits),
        VaultOutcome::Mediocre { credits, .. } => format!("Mediocre +{}", credits),
        VaultOutcome::Disaster { debt, .. } => format!("Disaster {}", debt),
        VaultOutcome::Catastrophe { .. } => "Catastrophe!".into(),
    }
}

fn ctx_value(
    ctx: Option<&GameContext>,
    ledger: Option<&issun::plugin::BudgetLedger>,
    channel: &BudgetChannel,
) -> String {
    if let Some(ledger) = ledger {
        match channel {
            BudgetChannel::Research => ledger.research_pool.to_string(),
            BudgetChannel::Operations => ledger.ops_pool.to_string(),
            BudgetChannel::Reserve => ledger.reserve.to_string(),
            BudgetChannel::Innovation => ledger.innovation_fund.to_string(),
            BudgetChannel::Security => ledger.security_fund.to_string(),
        }
    } else {
        "-".into()
    }
}

pub fn render_report(
    frame: &mut Frame,
    ctx: Option<&GameContext>,
    clock: Option<&issun::plugin::GameTimer>,
    ledger: Option<&issun::plugin::BudgetLedger>,
    territory: Option<&TerritoryStateCache>,
    prototypes: Option<&PrototypeBacklog>,
    reputation: Option<&ReputationLedger>,
    data: &IntelReportSceneData,
) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(frame.area());

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(layout[0]);

    let render_column =
        |frame: &mut Frame, area: Rect, title: &str, lines: &[String], active: bool| {
            let style = if active {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            let list = List::new(
                lines
                    .iter()
                    .map(|line| ListItem::new(Span::styled(line.clone(), style)))
                    .collect::<Vec<_>>(),
            )
            .block(Block::default().title(title).borders(Borders::ALL));
            frame.render_widget(list, area);
        };

    render_column(
        frame,
        columns[0],
        "Factions",
        &data.faction_lines,
        data.focus == 0,
    );
    render_column(
        frame,
        columns[1],
        "Territories",
        &data.territory_lines,
        data.focus == 1,
    );
    render_column(
        frame,
        columns[2],
        "Prototypes",
        &data.prototype_lines,
        data.focus == 2,
    );

    render_report_summary(frame, layout[1], ctx, clock, ledger, territory, prototypes, reputation);
}

fn render_report_summary(
    frame: &mut Frame,
    area: Rect,
    ctx: Option<&GameContext>,
    clock: Option<&issun::plugin::GameTimer>,
    ledger: Option<&issun::plugin::BudgetLedger>,
    territory: Option<&TerritoryStateCache>,
    prototypes: Option<&PrototypeBacklog>,
    reputation: Option<&ReputationLedger>,
) {
    let block = Block::default().title("Signals").borders(Borders::ALL);
    frame.render_widget(block.clone(), area);
    let inner = block.inner(area);
    let mut lines = Vec::new();

    if let Some(ctx) = ctx {
        let cash_text = ledger.map(|l| l.cash.to_string()).unwrap_or_else(|| "-".into());
        lines.push(Line::from(format!("Cash {}", cash_text)));
        lines.push(Line::from(format!(
            "Prototypes in dev: {}",
            ctx.prototypes.len()
        )));
        if let Some(front) = ctx.territories.iter().find(|t| t.battlefront) {
            let faction = ctx
                .enemy_faction_by_id(&front.enemy_faction)
                .map(|f| f.codename.as_str())
                .unwrap_or("Unknown");
            lines.push(Line::from(format!(
                "Battlefront: {} [{}] (Enemy {:.0}% | Conflict {:.0}%)",
                front.id.as_str(),
                faction,
                front.enemy_share * 100.0,
                front.conflict_intensity * 100.0
            )));
        }
    }

    if let Some(territory) = territory {
        if !territory.updates.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from("Recent Ops:"));
            for update in territory.updates.iter().take(3) {
                lines.push(Line::from(format!("• {}", update)));
            }
        }
    }

    if let Some(prototypes) = prototypes {
        if !prototypes.field_reports.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from("Telemetry:"));
            for report in prototypes.field_reports.iter().take(3) {
                lines.push(Line::from(format!("• {}", report)));
            }
        }
    }

    if let Some(reputation) = reputation {
        if !reputation.events.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from("Reputation events:"));
            for rep in reputation.events.iter().take(3) {
                lines.push(Line::from(format!("• {}", rep)));
            }
        }
    }

    if lines.is_empty() {
        lines.push(Line::from("No telemetry received this turn."));
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, inner);
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(area.height.saturating_sub(height) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(area.width.saturating_sub(width) / 2),
            Constraint::Length(width),
            Constraint::Min(0),
        ])
        .split(vertical[1]);

    horizontal[1]
}
