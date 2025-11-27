//! Whispers of Plague V2 - using issun-core and contagion_v2

mod components;
mod plugins;
mod resources;
mod states;
mod systems;
mod ui;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use components::*;
use crossterm::event::{self, Event, KeyCode};
use issun_bevy::plugins::contagion_v2::{ContagionInputParams, ContagionState};
use plugins::GameSetupPlugin;
use resources::{GameContext, GameMode, UIState, VictoryResult};
use states::GameScene;
use std::time::{Duration, Instant};

fn main() -> std::io::Result<()> {
    // Initialize terminal
    let mut terminal = ui::init_terminal()?;

    // Build Bevy app (headless)
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .add_plugins(GameSetupPlugin);

    // Run once to initialize plugins
    app.update();

    // Custom game loop
    let mut last_update = Instant::now();
    let tick_rate = Duration::from_millis(100); // 10 FPS

    loop {
        let now = Instant::now();
        let elapsed = now.duration_since(last_update);

        // Poll input (non-blocking)
        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                let world = app.world_mut();
                let current_state = world.resource::<State<GameScene>>().get().clone();

                match current_state {
                    GameScene::Title => match key.code {
                        KeyCode::Char('1') => {
                            {
                                let mut game_context = world.resource_mut::<GameContext>();
                                game_context.mode = GameMode::Plague;
                            }
                            {
                                let mut ui_state = world.resource_mut::<UIState>();
                                ui_state
                                    .add_message("Plague mode selected (issun-core)".to_string());
                            }
                            {
                                let mut next_state = world.resource_mut::<NextState<GameScene>>();
                                next_state.set(GameScene::Game);
                            }
                        }
                        KeyCode::Char('2') => {
                            {
                                let mut game_context = world.resource_mut::<GameContext>();
                                game_context.mode = GameMode::Savior;
                            }
                            {
                                let mut ui_state = world.resource_mut::<UIState>();
                                ui_state
                                    .add_message("Savior mode selected (issun-core)".to_string());
                            }
                            {
                                let mut next_state = world.resource_mut::<NextState<GameScene>>();
                                next_state.set(GameScene::Game);
                            }
                        }
                        KeyCode::Char('q') | KeyCode::Char('Q') => break,
                        _ => {}
                    },
                    GameScene::Game => match key.code {
                        KeyCode::Char('n') | KeyCode::Char('N') => {
                            let message = {
                                let mut game_context = world.resource_mut::<GameContext>();
                                game_context.turn += 1;
                                format!("Turn {}/{}", game_context.turn, game_context.max_turns)
                            };
                            let mut ui_state = world.resource_mut::<UIState>();
                            ui_state.add_message(message);
                        }
                        // District selection: 1-5
                        KeyCode::Char('1') => {
                            let mut ui_state = world.resource_mut::<UIState>();
                            ui_state.selected_district = 0;
                            ui_state.add_message("Selected: Downtown".to_string());
                        }
                        KeyCode::Char('2') => {
                            let mut ui_state = world.resource_mut::<UIState>();
                            ui_state.selected_district = 1;
                            ui_state.add_message("Selected: Industrial Zone".to_string());
                        }
                        KeyCode::Char('3') => {
                            let mut ui_state = world.resource_mut::<UIState>();
                            ui_state.selected_district = 2;
                            ui_state.add_message("Selected: Residential Area".to_string());
                        }
                        KeyCode::Char('4') => {
                            let mut ui_state = world.resource_mut::<UIState>();
                            ui_state.selected_district = 3;
                            ui_state.add_message("Selected: Suburbs".to_string());
                        }
                        KeyCode::Char('5') => {
                            let mut ui_state = world.resource_mut::<UIState>();
                            ui_state.selected_district = 4;
                            ui_state.add_message("Selected: Harbor District".to_string());
                        }
                        // Spread rumor (increase panic)
                        KeyCode::Char('r') | KeyCode::Char('R') => {
                            let selected = world.resource::<UIState>().selected_district;
                            let mut query = world.query::<&mut District>();
                            let mut districts: Vec<_> = query.iter_mut(world).collect();

                            if let Some(mut district) = districts.get_mut(selected) {
                                // Increase panic level
                                district.panic_level = (district.panic_level + 0.2).min(1.0);
                                let msg = format!(
                                    "Rumor spread in {}! Panic: {:.1}%",
                                    district.name,
                                    district.panic_level * 100.0
                                );
                                let mut ui_state = world.resource_mut::<UIState>();
                                ui_state.add_message(msg);
                            }
                        }
                        // Isolation policy (decrease panic and increase resistance)
                        KeyCode::Char('i') | KeyCode::Char('I') => {
                            let selected = world.resource::<UIState>().selected_district;
                            let mut query =
                                world.query::<(&mut District, &mut ContagionInputParams)>();
                            let mut items: Vec<_> = query.iter_mut(world).collect();

                            if let Some((ref mut district, ref mut params)) =
                                items.get_mut(selected)
                            {
                                // Decrease panic level
                                district.panic_level = (district.panic_level - 0.15).max(0.0);
                                // Increase resistance
                                params.resistance = (params.resistance + 5).min(50);
                                let msg = format!(
                                    "Isolation in {}! Panic: {:.1}%, Resistance: {}",
                                    district.name,
                                    district.panic_level * 100.0,
                                    params.resistance
                                );
                                let mut ui_state = world.resource_mut::<UIState>();
                                ui_state.add_message(msg);
                            }
                        }
                        KeyCode::Char('q') | KeyCode::Char('Q') => break,
                        _ => {}
                    },
                    GameScene::Result => match key.code {
                        KeyCode::Enter => {
                            {
                                let mut game_context = world.resource_mut::<GameContext>();
                                game_context.turn = 0;
                            }
                            {
                                let mut victory_result = world.resource_mut::<VictoryResult>();
                                victory_result.0 = None;
                            }
                            {
                                let mut query = world
                                    .query::<(&mut District, &mut ContagionState<PlagueVirus>)>();
                                for (mut district, mut state) in query.iter_mut(world) {
                                    district.infected = 0;
                                    district.dead = 0;
                                    district.panic_level = 0.2;
                                    state.state.severity = 0;
                                }
                            }
                            {
                                let mut next_state = world.resource_mut::<NextState<GameScene>>();
                                next_state.set(GameScene::Title);
                            }
                        }
                        KeyCode::Char('q') | KeyCode::Char('Q') => break,
                        _ => {}
                    },
                }
            }
        }

        // Update Bevy systems at fixed rate
        if elapsed >= tick_rate {
            app.update();
            last_update = now;
        }

        // Render UI
        let world = app.world_mut();
        ui::render_frame(&mut terminal, world)?;

        // Small sleep to prevent busy-wait
        std::thread::sleep(Duration::from_millis(10));
    }

    // Cleanup
    ui::restore_terminal(&mut terminal)?;
    Ok(())
}
