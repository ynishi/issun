//! CLI display and rendering

use bevy::prelude::*;
use issun_bevy::plugins::{action::ActionPoints, contagion::*};

use crate::{
    events::EventLog,
    game_rules::*,
    player::*,
    world::*,
};

/// Display game status
pub fn display_status(
    stats: Res<GameStats>,
    player: Res<Player>,
    action_points: Query<&ActionPoints>,
    cure_research: Res<CureResearch>,
    emergency_budget: Res<EmergencyBudget>,
    infections: Query<&ContagionInfection>,
    nodes: Query<&ContagionNode>,
    contagions: Query<&Contagion>,
    event_log: Res<EventLog>,
) {
    println!("\n{}", "=".repeat(60));
    println!("           PANDEMIC CRISIS - Turn {}", stats.current_turn);
    println!("{}\n", "=".repeat(60));

    // Global status
    let total_pop = get_total_population();
    let infection_rate = stats.infection_rate() * 100.0;
    let active_rate = stats.active_rate() * 100.0;

    println!("📊 Global Status:");
    println!("   Total Population:  {:>10}", format_number(total_pop));
    println!("   Infected:          {:>10} ({:.1}%)",
        format_number(stats.total_infected), infection_rate);
    println!("   Active Cases:      {:>10} ({:.1}%)",
        format_number(stats.total_active), active_rate);
    println!("   Recovered:         {:>10}",
        format_number(stats.total_recovered));

    // Player resources
    if let Ok(ap) = action_points.get(player.entity) {
        println!("\n💰 Player Resources:");
        println!("   Action Points:     {}/15", ap.available);
        println!("   Cure Progress:     {:.0}%", cure_research.progress * 100.0);
        if cure_research.deployed {
            if cure_research.deployment_complete(stats.current_turn) {
                println!("   Cure Status:       ✅ DEPLOYED");
            } else {
                let turns_left = cure_research.deployment_turn.unwrap() + 3 - stats.current_turn;
                println!("   Cure Status:       🚀 Deploying ({} turns left)", turns_left);
            }
        }
        println!("   Emergency Budget:  {}/{}",
            emergency_budget.uses_remaining, emergency_budget.max_uses);
    }

    // Top infected cities
    let mut city_infections: Vec<(String, usize, usize)> = Vec::new();
    for city_data in CITIES {
        let city_entity = nodes
            .iter()
            .find(|n| n.node_id == city_data.id)
            .map(|_| ());

        if city_entity.is_some() {
            let mut active_count = 0;
            let mut incubating_count = 0;

            for (node, infection) in nodes.iter().zip(infections.iter()) {
                if node.node_id == city_data.id {
                    match infection.state {
                        InfectionState::Active { .. } => active_count += 1,
                        InfectionState::Incubating { .. } => incubating_count += 1,
                        _ => {}
                    }
                }
            }

            if active_count > 0 || incubating_count > 0 {
                city_infections.push((city_data.name.to_string(), active_count, incubating_count));
            }
        }
    }

    city_infections.sort_by(|a, b| b.1.cmp(&a.1));

    println!("\n🌍 City Status (Top Infected):");
    if city_infections.is_empty() {
        println!("   No infections detected");
    } else {
        for (i, (city_name, active, incubating)) in city_infections.iter().take(5).enumerate() {
            if *active > 0 {
                println!("   {}. {} - Active: {}", i + 1, city_name, format_number(*active));
            } else if *incubating > 0 {
                println!("   {}. {} - Incubating: {}", i + 1, city_name, format_number(*incubating));
            }
        }
    }

    // Mutation count
    let mutation_count = contagions.iter().count();
    if mutation_count > 1 {
        println!("\n🧬 Mutations Active: {} variants", mutation_count);
    }

    // Recent events
    println!("\n📰 Recent Events:");
    let recent_events = event_log.recent(5);
    if recent_events.is_empty() {
        println!("   (No recent events)");
    } else {
        for event in recent_events {
            println!("   {}", event);
        }
    }

    println!();
}

/// Display available actions
pub fn display_actions(
    player: Res<Player>,
    action_points: Query<&ActionPoints>,
) {
    println!("🎮 Available Actions:");

    if let Ok(ap) = action_points.get(player.entity) {
        println!("   [1] Quarantine City        (3 AP) - Block transmissions for 3 turns");
        println!("   [2] Increase Awareness     (2 AP) - Boost city resistance for 5 turns");
        println!("   [3] Develop Cure Research  (5 AP) - Advance cure progress by 10%");
        println!("   [4] Emergency Healthcare   (4 AP) - Major resistance boost (limited uses)");
        println!("   [5] Travel Ban             (2 AP) - Reduce all transmissions for 2 turns");
        println!("   [6] Monitor City           (1 AP) - View city infection details");
        println!("   [7] End Turn               (0 AP) - Advance to next turn");
        println!("\n   Current AP: {}/15", ap.available);
        print!("\n> ");
    }
}

/// Display victory message
pub fn display_victory(victory_type: VictoryType, stats: &GameStats) {
    println!("\n{}", "=".repeat(60));
    println!("                        🎉 VICTORY! 🎉");
    println!("{}", "=".repeat(60));

    match victory_type {
        VictoryType::CureDeployed => {
            println!("\nYou successfully developed and deployed the cure!");
            println!("The pandemic has been contained through scientific triumph.");
        }
        VictoryType::NaturalContainment => {
            println!("\nThrough careful management, the disease burned out naturally!");
            println!("Humanity survived through your strategic interventions.");
        }
    }

    println!("\nFinal Statistics:");
    println!("  Turns Survived:     {}", stats.current_turn);
    println!("  Total Infected:     {}", format_number(stats.total_infected));
    println!("  Final Infection:    {:.1}%", stats.infection_rate() * 100.0);

    println!("\n{}", "=".repeat(60));
}

/// Display defeat message
pub fn display_defeat(defeat_type: DefeatType, stats: &GameStats) {
    println!("\n{}", "=".repeat(60));
    println!("                        ☠️  DEFEAT  ☠️");
    println!("{}", "=".repeat(60));

    match defeat_type {
        DefeatType::GlobalPandemic => {
            println!("\nThe disease has spread to 70% of the global population.");
            println!("Healthcare systems have collapsed. Humanity faces extinction.");
        }
        DefeatType::CriticalMutations => {
            println!("\nThe disease has mutated into multiple critical strains.");
            println!("Medical responses are overwhelmed. Hope is lost.");
        }
        DefeatType::EconomicCollapse => {
            println!("\nToo many cities under quarantine for too long.");
            println!("The global economy has collapsed. Society breaks down.");
        }
    }

    println!("\nFinal Statistics:");
    println!("  Turns Survived:     {}", stats.current_turn);
    println!("  Total Infected:     {}", format_number(stats.total_infected));
    println!("  Final Infection:    {:.1}%", stats.infection_rate() * 100.0);

    println!("\n{}", "=".repeat(60));
}

/// Format number with commas
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
