//! Pandemic Crisis - Turn-based pandemic management game
//!
//! Demonstrates the Contagion Plugin's infection state machine and
//! graph-based propagation mechanics.

mod disease;
mod display;
mod events;
mod game_rules;
mod player;
mod ui;
mod world;

use bevy::prelude::*;
use issun_bevy::{
    IssunCorePlugin,
    plugins::{
        action::*, contagion::*, time::*,
    },
};

use disease::*;
use events::*;
use game_rules::*;
use player::*;
use world::*;

use issun_macros::{log, IssunBevyPlugin};

use crossterm::event::{self, Event, KeyCode};
use std::io;
use std::time::Duration;

// ============================================================================
// Pandemic Crisis Plugin (using IssunBevyPlugin macro)
// ============================================================================

/// Pandemic Crisis game plugin with all game-specific resources
///
/// Note: auto_register_types is disabled because resources don't implement Reflect yet.
/// To enable: Add `#[derive(Reflect)]` to all resources and set `auto_register_types = true`
///
/// Note: messages attribute would use add_event(), but issun-bevy uses add_message()
/// So event registration is done manually in Plugin::build()
#[derive(Default, IssunBevyPlugin)]
#[plugin(name = "pandemic_crisis")]
pub struct PandemicCrisisPlugin {
    #[resource]
    pub game_state: GameState,

    #[resource]
    pub stats: GameStats,

    #[resource]
    pub cure_research: CureResearch,

    #[resource]
    pub budget: EmergencyBudget,

    #[resource]
    pub quarantines: ActiveQuarantines,

    #[resource]
    pub awareness: ActiveAwareness,

    #[resource]
    pub healthcare: ActiveEmergencyHealthcare,

    #[resource]
    pub travel_ban: TravelBanStatus,

    #[resource]
    pub event_log: EventLog,
}

impl PandemicCrisisPlugin {
    /// Create plugin with custom event log size
    pub fn with_event_log_size(mut self, max_entries: usize) -> Self {
        self.event_log = EventLog::new(max_entries);
        self
    }
}

fn main() -> io::Result<()> {
    // Initialize terminal
    let mut terminal = ui::init_terminal()?;

    // Default difficulty (can be made configurable later)
    let difficulty = Difficulty::Normal;

    // Create app
    let mut app = App::new();

    // Plugins
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.add_plugins(IssunCorePlugin);

    // Contagion plugin with difficulty config
    let contagion_config = difficulty.to_config();
    app.add_plugins(
        ContagionPlugin::default()
            .with_config(contagion_config)
            .with_seed(42),
    );

    // Action plugin
    app.add_plugins(ActionPlugin::default());

    // Time plugin
    app.add_plugins(TimePlugin::default());

    // Game resources (using IssunBevyPlugin macro)
    app.add_plugins(
        PandemicCrisisPlugin::default()
            .with_event_log_size(20)
    );

    // Startup systems
    app.add_systems(Startup, (
        setup_world,
        setup_player,
        spawn_initial_disease,
    ).chain());

    // Game loop systems (event handlers only, not turn-based logic)
    app.add_systems(Update, (
        handle_contagion_spawned,
        handle_contagion_spread,
        handle_state_changes,
        handle_propagation_complete,
    ));

    // Initialize
    app.update();

    // Initial stats update
    update_game_stats_manual(&mut app);

    // Render initial frame
    ui::render_frame(&mut terminal, app.world())?;

    // City selection state
    let mut selecting_city = None;

    // Main game loop
    loop {
        // Poll for input (non-blocking)
        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                let game_state = *app.world().resource::<GameState>();

                match game_state {
                    GameState::Playing => {
                        if let Some(action) = selecting_city {
                            // Handle city selection
                            if let KeyCode::Char(c) = key.code {
                                if let Some(digit) = c.to_digit(10) {
                                    if digit >= 1 && digit <= CITIES.len() as u32 {
                                        execute_city_action(&mut app, action, digit as usize - 1);
                                        selecting_city = None;
                                    }
                                }
                            }
                            if let KeyCode::Esc = key.code {
                                selecting_city = None;
                            }
                        } else {
                            // Handle action selection
                            match key.code {
                                KeyCode::Char('1') => selecting_city = Some(CityAction::Quarantine),
                                KeyCode::Char('2') => selecting_city = Some(CityAction::Awareness),
                                KeyCode::Char('3') => handle_cure_research_action(&mut app),
                                KeyCode::Char('4') => selecting_city = Some(CityAction::EmergencyHealthcare),
                                KeyCode::Char('5') => handle_travel_ban_action(&mut app),
                                KeyCode::Char('6') => selecting_city = Some(CityAction::Monitor),
                                KeyCode::Char('7') => handle_end_turn(&mut app),
                                KeyCode::Char('q') | KeyCode::Char('Q') => break,
                                _ => {}
                            }
                        }
                    }
                    GameState::Victory(_) | GameState::Defeat(_) => {
                        // Exit on any key
                        if matches!(key.code, KeyCode::Enter | KeyCode::Char('q') | KeyCode::Char('Q')) {
                            break;
                        }
                    }
                }
            }
        }

        // Update game systems
        app.update();

        // Render frame
        ui::render_frame(&mut terminal, app.world())?;

        // Check for game end
        let game_state = *app.world().resource::<GameState>();
        if matches!(game_state, GameState::Victory(_) | GameState::Defeat(_)) {
            // Keep rendering end screen
        }
    }

    // Cleanup terminal
    ui::restore_terminal(&mut terminal)?;

    Ok(())
}

// City action enum
#[derive(Clone, Copy)]
enum CityAction {
    Quarantine,
    Awareness,
    EmergencyHealthcare,
    Monitor,
}

// Execute action on selected city
fn execute_city_action(app: &mut App, action: CityAction, city_index: usize) {
    if city_index >= CITIES.len() {
        return;
    }

    let city = &CITIES[city_index];

    match action {
        CityAction::Quarantine => {
            let player_entity = app.world().resource::<Player>().entity;
            if let Ok(mut ap) = app.world_mut().query::<&mut ActionPoints>().get_mut(app.world_mut(), player_entity) {
                if try_consume_ap(&mut ap, 3) {
                    let current_turn = app.world().resource::<GameStats>().current_turn;
                    app.world_mut().resource_mut::<ActiveQuarantines>()
                        .add(city.id.to_string(), current_turn, 3);

                    log!(app, "✅ {} quarantined for 3 turns", city.name);
                }
            }
        }
        CityAction::Awareness => {
            let player_entity = app.world().resource::<Player>().entity;
            if let Ok(mut ap) = app.world_mut().query::<&mut ActionPoints>().get_mut(app.world_mut(), player_entity) {
                if try_consume_ap(&mut ap, 2) {
                    let current_turn = app.world().resource::<GameStats>().current_turn;
                    app.world_mut().resource_mut::<ActiveAwareness>()
                        .add(city.id.to_string(), current_turn);

                    log!(app, "✅ Awareness campaign started in {}", city.name);
                }
            }
        }
        CityAction::EmergencyHealthcare => {
            if !app.world().resource::<EmergencyBudget>().can_use() {
                log!(app, "❌ No emergency budget remaining");
                return;
            }

            let player_entity = app.world().resource::<Player>().entity;
            if let Ok(mut ap) = app.world_mut().query::<&mut ActionPoints>().get_mut(app.world_mut(), player_entity) {
                if try_consume_ap(&mut ap, 4) {
                    app.world_mut().resource_mut::<EmergencyBudget>().use_budget();

                    let current_turn = app.world().resource::<GameStats>().current_turn;
                    app.world_mut().resource_mut::<ActiveEmergencyHealthcare>()
                        .add(city.id.to_string(), current_turn);

                    app.world_mut().resource_mut::<EventLog>()
                        .add(format!("✅ Emergency healthcare deployed in {}", city.name));
                }
            }
        }
        CityAction::Monitor => {
            let player_entity = app.world().resource::<Player>().entity;
            if let Ok(mut ap) = app.world_mut().query::<&mut ActionPoints>().get_mut(app.world_mut(), player_entity) {
                if try_consume_ap(&mut ap, 1) {
                    app.world_mut().resource_mut::<EventLog>()
                        .add(format!("📊 Monitoring: {} (Pop: {}, Resistance: {:.1}%)",
                            city.name, city.population, city.resistance * 100.0));
                }
            }
        }
    }
}

// Action handlers (for non-city actions)

fn handle_cure_research_action(app: &mut App) {
    let player_entity = app.world().resource::<Player>().entity;
    if let Ok(mut ap) = app.world_mut().query::<&mut ActionPoints>().get_mut(app.world_mut(), player_entity) {
        if try_consume_ap(&mut ap, 5) {
            app.world_mut().resource_mut::<CureResearch>().advance(0.1);
            let progress = app.world().resource::<CureResearch>().progress;

            app.world_mut().resource_mut::<EventLog>()
                .add(format!("✅ Cure research advanced to {:.0}%", progress * 100.0));

            if progress >= 1.0 {
                let current_turn = app.world().resource::<GameStats>().current_turn;
                app.world_mut().resource_mut::<CureResearch>().deploy(current_turn);
                app.world_mut().resource_mut::<EventLog>()
                    .add("🎉 Cure complete! Deploying to all cities (3 turns)...".to_string());
            }
        } else {
            app.world_mut().resource_mut::<EventLog>()
                .add("❌ Not enough AP (need 5)".to_string());
        }
    }
}

fn handle_travel_ban_action(app: &mut App) {
    let player_entity = app.world().resource::<Player>().entity;
    if let Ok(mut ap) = app.world_mut().query::<&mut ActionPoints>().get_mut(app.world_mut(), player_entity) {
        if try_consume_ap(&mut ap, 2) {
            let current_turn = app.world().resource::<GameStats>().current_turn;
            app.world_mut().resource_mut::<TravelBanStatus>().activate(current_turn);

            app.world_mut().resource_mut::<EventLog>()
                .add("✅ Global travel ban activated for 2 turns".to_string());
        } else {
            app.world_mut().resource_mut::<EventLog>()
                .add("❌ Not enough AP (need 2)".to_string());
        }
    }
}

fn handle_end_turn(app: &mut App) {
    // Trigger propagation
    app.world_mut().write_message(PropagationStepRequested);

    // Advance turn
    app.world_mut().resource_mut::<GameStats>().current_turn += 1;

    // Trigger turn advancement for contagion plugin
    app.world_mut().write_message(TurnAdvancedMessage);

    // Regenerate AP
    {
        let player_entity = app.world().resource::<Player>().entity;
        if let Ok(mut ap) = app.world_mut().query::<&mut ActionPoints>().get_mut(app.world_mut(), player_entity) {
            let new_ap = (ap.available + 8).min(15);
            ap.available = new_ap;

            app.world_mut().resource_mut::<EventLog>()
                .add(format!("⏭️ Turn ended. AP regenerated: {}/15", new_ap));
        }
    }

    // Cleanup expired effects
    let current_turn = app.world().resource::<GameStats>().current_turn;
    app.world_mut().resource_mut::<ActiveQuarantines>().cleanup(current_turn);
    app.world_mut().resource_mut::<ActiveAwareness>().cleanup(current_turn);
    app.world_mut().resource_mut::<ActiveEmergencyHealthcare>().cleanup(current_turn);
    app.world_mut().resource_mut::<TravelBanStatus>().update(current_turn);

    // Update app to process propagation and state changes
    app.update();

    // Manual turn-based logic: Update stats and check win/loss conditions
    update_game_stats_manual(app);
    check_victory_conditions_manual(app);
    check_defeat_conditions_manual(app);
}

// Manual stat update (called only at turn end)
fn update_game_stats_manual(app: &mut App) {
    // Collect infection states using iter_entities
    let infection_data: Vec<InfectionState> = app.world()
        .iter_entities()
        .filter_map(|entity_ref| {
            entity_ref.get::<ContagionInfection>().map(|inf| inf.state.clone())
        })
        .collect();

    // Count stats
    let mut total_infected = 0;
    let mut total_active = 0;
    let mut total_recovered = 0;

    for state in infection_data {
        total_infected += 1;
        match state {
            InfectionState::Active { .. } => total_active += 1,
            InfectionState::Recovered { .. } => total_recovered += 1,
            _ => {}
        }
    }

    // Update resource
    let mut stats = app.world_mut().resource_mut::<GameStats>();
    stats.total_infected = total_infected;
    stats.total_active = total_active;
    stats.total_recovered = total_recovered;

    // Update low infection streak (per turn, not per frame!)
    if stats.infection_rate() < 0.1 {
        stats.low_infection_streak += 1;
    } else {
        stats.low_infection_streak = 0;
    }
}

// Manual victory check (called only at turn end)
fn check_victory_conditions_manual(app: &mut App) {
    let game_state = *app.world().resource::<GameState>();
    if game_state != GameState::Playing {
        return;
    }

    let stats = app.world().resource::<GameStats>();
    let cure_research = app.world().resource::<CureResearch>();

    // Victory: Cure deployed and complete
    if cure_research.deployed && cure_research.deployment_complete(stats.current_turn) {
        app.world_mut().resource_mut::<EventLog>()
            .add("🎉 VICTORY: Cure deployed successfully!".to_string());
        *app.world_mut().resource_mut::<GameState>() = GameState::Victory(VictoryType::CureDeployed);
        return;
    }

    // Victory: Natural containment
    if stats.low_infection_streak >= 15 {
        app.world_mut().resource_mut::<EventLog>()
            .add("🎉 VICTORY: Natural containment achieved!".to_string());
        *app.world_mut().resource_mut::<GameState>() = GameState::Victory(VictoryType::NaturalContainment);
        return;
    }
}

// Manual defeat check (called only at turn end)
fn check_defeat_conditions_manual(app: &mut App) {
    let game_state = *app.world().resource::<GameState>();
    if game_state != GameState::Playing {
        return;
    }

    // Collect data
    let stats_infection_rate = app.world().resource::<GameStats>().infection_rate();
    let stats_current_turn = app.world().resource::<GameStats>().current_turn;
    let mutation_count = app.world()
        .iter_entities()
        .filter(|entity_ref| entity_ref.contains::<Contagion>())
        .count();
    let quarantine_data = app.world().resource::<ActiveQuarantines>().quarantines.clone();

    // Defeat: Global pandemic (70%+ infected)
    if stats_infection_rate >= 0.7 {
        app.world_mut().resource_mut::<EventLog>()
            .add("☠️ DEFEAT: Global pandemic - 70%+ infected!".to_string());
        *app.world_mut().resource_mut::<GameState>() = GameState::Defeat(DefeatType::GlobalPandemic);
        return;
    }

    // Defeat: Critical mutations (3+ contagion strains)
    if mutation_count >= 3 {
        app.world_mut().resource_mut::<EventLog>()
            .add("☠️ DEFEAT: Critical mutations overwhelm response!".to_string());
        *app.world_mut().resource_mut::<GameState>() = GameState::Defeat(DefeatType::CriticalMutations);
        return;
    }

    // Defeat: Economic collapse (5+ cities quarantined for 10+ turns)
    let long_quarantine_count = quarantine_data.iter()
        .filter(|q| stats_current_turn.saturating_sub(q.start_turn) >= 10)
        .count();
    if long_quarantine_count >= 5 {
        app.world_mut().resource_mut::<EventLog>()
            .add("☠️ DEFEAT: Economic collapse from extended quarantines!".to_string());
        *app.world_mut().resource_mut::<GameState>() = GameState::Defeat(DefeatType::EconomicCollapse);
        return;
    }
}

