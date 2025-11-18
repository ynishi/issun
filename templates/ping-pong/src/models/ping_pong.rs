//! Shared types for the Ping/Pong tutorial scenes.

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

/// Identifies which half of the mini demo is active.
#[derive(Debug, Clone, Copy)]
pub enum PingPongStage {
    Ping,
    Pong,
}

impl PingPongStage {
    pub fn label(&self) -> &'static str {
        match self {
            PingPongStage::Ping => "Ping",
            PingPongStage::Pong => "Pong",
        }
    }
}

/// Runtime resource populated from assets to supply flavor text.
pub struct PingPongMessageDeck {
    normal: Vec<&'static str>,
    congrats: Vec<&'static str>,
    rng: StdRng,
}

impl PingPongMessageDeck {
    pub fn from_assets(normal: &'static [&'static str], congrats: &'static [&'static str]) -> Self {
        Self {
            normal: normal.to_vec(),
            congrats: congrats.to_vec(),
            rng: StdRng::from_entropy(),
        }
    }

    /// Get a line appropriate for the bounce type (special = congrats).
    pub fn draw(&mut self, special: bool) -> &'static str {
        const DEFAULT: &str = "Keep bouncing!";

        let pool = if special && !self.congrats.is_empty() {
            &self.congrats
        } else if !self.normal.is_empty() {
            &self.normal
        } else {
            return DEFAULT;
        };

        pool.choose(&mut self.rng).copied().unwrap_or(DEFAULT)
    }
}
