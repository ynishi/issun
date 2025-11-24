//! Garden Simulator - Demonstration of GenerationPlugin + EntropyPlugin
//!
//! A simple garden management game where plants grow (GenerationPlugin)
//! and decay (EntropyPlugin) based on environmental conditions.

mod hooks;
mod models;

use hecs::Entity;
use hooks::{GardenEntropyHook, GardenGenerationHook};
use issun::plugin::entropy::{
    Durability, EntropyConfig, EntropyStateECS, EntropySystemECS, EnvironmentalExposure,
    MaterialType,
};
use issun::plugin::generation::{
    Generation, GenerationConfig, GenerationEnvironment, GenerationStateECS, GenerationSystemECS,
    GenerationType,
};
use models::{GrowthStage, PlantHealth, PlantSpecies};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

struct Garden {
    generation_system: GenerationSystemECS,
    generation_state: GenerationStateECS,
    generation_config: GenerationConfig,

    entropy_system: EntropySystemECS,
    entropy_state: EntropyStateECS,
    entropy_config: EntropyConfig,

    tick_count: u64,
    plants: Vec<(Entity, PlantSpecies)>,
}

impl Garden {
    fn new() -> Self {
        let generation_hook = Arc::new(GardenGenerationHook);
        let entropy_hook = Arc::new(GardenEntropyHook);

        Self {
            generation_system: GenerationSystemECS::new(generation_hook),
            generation_state: GenerationStateECS::new(),
            generation_config: GenerationConfig::default(),

            entropy_system: EntropySystemECS::new(entropy_hook),
            entropy_state: EntropyStateECS::new(),
            entropy_config: EntropyConfig::default(),

            tick_count: 0,
            plants: Vec::new(),
        }
    }

    fn plant_seed(&mut self, species: PlantSpecies) {
        // Create generation component (growth)
        let generation = Generation::new(
            species.max_growth(),
            species.growth_rate(),
            GenerationType::Organic,
        );

        // Create environment (optimal conditions for now)
        let environment = GenerationEnvironment::with_values(
            22.0, // temperature
            0.8,  // fertility
            1.0,  // resources (water)
            0.9,  // light
        );

        // Spawn in generation system
        let gen_entity = self.generation_state.spawn_entity(generation, environment);

        // Create durability component (health/decay)
        let durability = Durability::new(
            species.max_durability(),
            species.decay_rate(),
            MaterialType::Organic,
        );

        // Create environmental exposure
        let exposure = EnvironmentalExposure {
            humidity: 0.5,
            pollution: 0.0,
            temperature: 22.0,
            sunlight_exposure: 0.9,
        };

        // Spawn in entropy system
        let _ent_entity = self.entropy_state.spawn_entity(durability, exposure);

        println!(
            "🌱 Planted {} {} (growth: {:.1}/tick, decay: {:.1}/tick)",
            species.icon(),
            species.name(),
            species.growth_rate(),
            species.decay_rate()
        );

        // Store plant info
        self.plants.push((gen_entity, species));
    }

    async fn update_tick(&mut self, delta_time: f32) {
        self.tick_count += 1;

        // Update generation (plants grow)
        self.generation_system
            .update_generation(&mut self.generation_state, &self.generation_config, delta_time)
            .await;

        // Update entropy (plants decay)
        self.entropy_system
            .update_decay(&mut self.entropy_state, &self.entropy_config, delta_time)
            .await;

        // Cleanup
        self.generation_system
            .cleanup_completed(&mut self.generation_state);
        self.entropy_system
            .cleanup_destroyed(&mut self.entropy_state);
    }

    fn display_status(&self) {
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("🌻 GARDEN STATUS - Tick #{}", self.tick_count);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        for (idx, (entity, species)) in self.plants.iter().enumerate() {
            // Get generation status
            if let Ok(generation) = self
                .generation_state
                .world
                .get::<&Generation>(*entity)
            {
                let progress = generation.progress_ratio();
                let stage = GrowthStage::from_progress(progress);

                // Get durability status
                let durability_ratio = if let Ok(durability) =
                    self.entropy_state.world.get::<&Durability>(*entity)
                {
                    durability.current / durability.max
                } else {
                    1.0
                };

                let health = PlantHealth::from_durability_ratio(durability_ratio);

                println!(
                    "{}. {} {} - Growth: {} {:.1}% | Health: {:?} {:.1}%",
                    idx + 1,
                    species.icon(),
                    species.name(),
                    stage.icon(),
                    progress * 100.0,
                    health,
                    durability_ratio * 100.0
                );
            }
        }

        // Display metrics
        let gen_metrics = self.generation_state.metrics();
        let ent_metrics = self.entropy_state.metrics();

        println!("\n📊 Metrics:");
        println!(
            "  Generation: {} entities, {:.2} total progress",
            gen_metrics.entities_processed, gen_metrics.total_progress_applied
        );
        println!(
            "  Entropy: {} entities, {:.2} total decay",
            ent_metrics.entities_processed, ent_metrics.total_decay_applied
        );
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    }
}

#[tokio::main]
async fn main() {
    println!("🌻 Welcome to Garden Simulator!");
    println!("Demonstrating GenerationPlugin + EntropyPlugin\n");

    let mut garden = Garden::new();

    // Plant some seeds
    garden.plant_seed(PlantSpecies::Tomato);
    garden.plant_seed(PlantSpecies::Lettuce);
    garden.plant_seed(PlantSpecies::Carrot);
    garden.plant_seed(PlantSpecies::Wheat);
    garden.plant_seed(PlantSpecies::Sunflower);

    println!("\n🎮 Starting simulation...\n");

    // Run simulation for 50 ticks
    for _ in 0..50 {
        garden.update_tick(1.0).await;

        // Display status every 5 ticks
        if garden.tick_count.is_multiple_of(5) {
            garden.display_status();
        }

        // Simulate tick rate (200ms per tick)
        sleep(Duration::from_millis(200)).await;
    }

    println!("🏁 Simulation complete!");
    garden.display_status();
}
