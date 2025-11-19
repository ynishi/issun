//! Multiplayer Pong - Network-transparent EventBus demonstration
//!
//! A simple 2-player pong game demonstrating ISSUN's network capabilities.
//! Players connect to a relay server and control paddles to hit a ball back and forth.
//!
//! Usage:
//!   # Start relay server (in one terminal)
//!   make server
//!
//!   # Start Player 1 (in another terminal)
//!   cargo run -p multiplayer-pong -- --server 127.0.0.1:5000
//!
//!   # Start Player 2 (in another terminal)
//!   cargo run -p multiplayer-pong -- --server 127.0.0.1:5000

use clap::Parser;
use issun::event::{Event, EventBus};
use issun::network::{NetworkBackend, NetworkScope, QuicClientBackend};
use std::time::Duration;

/// Command-line arguments
#[derive(Parser, Debug)]
#[command(name = "Multiplayer Pong")]
#[command(about = "A networked pong game using ISSUN EventBus")]
struct Args {
    /// Relay server address (e.g., 127.0.0.1:5000)
    #[arg(short, long)]
    server: String,
}

/// Networked event: Player moves their paddle
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct PaddleMove {
    player_id: u64,
    y_position: i32,
}

impl Event for PaddleMove {
    fn is_networked() -> bool {
        true
    }

    fn network_scope() -> NetworkScope {
        NetworkScope::Broadcast
    }
}

/// Networked event: Ball position update (sent by host only)
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct BallUpdate {
    x: i32,
    y: i32,
    velocity_x: i32,
    velocity_y: i32,
}

impl Event for BallUpdate {
    fn is_networked() -> bool {
        true
    }

    fn network_scope() -> NetworkScope {
        NetworkScope::Broadcast
    }
}

/// Local event: User input
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
enum Input {
    Up,
    Down,
    Quit,
}

impl Event for Input {}

/// Game state
struct GameState {
    /// This player's paddle Y position
    my_paddle_y: i32,

    /// Other player's paddle Y position
    other_paddle_y: i32,

    /// Ball position
    ball_x: i32,
    ball_y: i32,
    ball_vx: i32,
    ball_vy: i32,

    /// Player IDs
    my_id: u64,
    other_id: Option<u64>,

    /// Am I the host? (Host simulates ball physics)
    is_host: bool,

    /// Frame counter
    frame: u64,
}

impl GameState {
    fn new(my_id: u64, is_host: bool) -> Self {
        Self {
            my_paddle_y: 10,
            other_paddle_y: 10,
            ball_x: 40,
            ball_y: 10,
            ball_vx: if is_host { 1 } else { 0 },
            ball_vy: if is_host { 1 } else { 0 },
            my_id,
            other_id: None,
            is_host,
            frame: 0,
        }
    }

    fn update(&mut self, bus: &mut EventBus) {
        self.frame += 1;

        // Process paddle moves from other players
        for event in bus.reader::<PaddleMove>().iter() {
            if event.player_id != self.my_id {
                self.other_paddle_y = event.y_position;
                if self.other_id.is_none() {
                    self.other_id = Some(event.player_id);
                    println!("🎮 Player {} joined!", event.player_id);
                }
            }
        }

        // If host, simulate ball physics
        if self.is_host && self.other_id.is_some() {
            self.ball_x += self.ball_vx;
            self.ball_y += self.ball_vy;

            // Bounce off top/bottom
            if self.ball_y <= 0 || self.ball_y >= 20 {
                self.ball_vy = -self.ball_vy;
            }

            // Paddle collision (simplified)
            if self.ball_x <= 2 && (self.ball_y >= self.my_paddle_y && self.ball_y <= self.my_paddle_y + 4) {
                self.ball_vx = -self.ball_vx;
            }
            if self.ball_x >= 78 && (self.ball_y >= self.other_paddle_y && self.ball_y <= self.other_paddle_y + 4) {
                self.ball_vx = -self.ball_vx;
            }

            // Score/reset
            if self.ball_x < 0 {
                println!("🎉 Player {} scores!", self.other_id.unwrap());
                self.reset_ball();
            } else if self.ball_x > 80 {
                println!("🎉 You score!");
                self.reset_ball();
            }

            // Broadcast ball state every frame
            bus.publish(BallUpdate {
                x: self.ball_x,
                y: self.ball_y,
                velocity_x: self.ball_vx,
                velocity_y: self.ball_vy,
            });
        } else {
            // Receive ball updates from host
            for event in bus.reader::<BallUpdate>().iter() {
                self.ball_x = event.x;
                self.ball_y = event.y;
                self.ball_vx = event.velocity_x;
                self.ball_vy = event.velocity_y;
            }
        }
    }

    fn reset_ball(&mut self) {
        self.ball_x = 40;
        self.ball_y = 10;
        self.ball_vx = if rand::random::<bool>() { 1 } else { -1 };
        self.ball_vy = if rand::random::<bool>() { 1 } else { -1 };
    }

    fn render(&self) {
        // Clear screen
        print!("\x1B[2J\x1B[1;1H");

        println!("╔════════════════════════════════════════════════════════════════════════════════╗");
        println!("║  Multiplayer Pong - Player {}  {}                                          ║",
            self.my_id % 1000,
            if self.is_host { "[HOST]" } else { "[CLIENT]" });
        println!("╠════════════════════════════════════════════════════════════════════════════════╣");

        for y in 0..20 {
            print!("║");

            for x in 0..80 {
                if x == self.ball_x && y == self.ball_y {
                    print!("●");
                } else if x == 1 && y >= self.my_paddle_y && y < self.my_paddle_y + 4 {
                    print!("█");
                } else if x == 78 && self.other_id.is_some() && y >= self.other_paddle_y && y < self.other_paddle_y + 4 {
                    print!("█");
                } else {
                    print!(" ");
                }
            }

            println!("║");
        }

        println!("╚════════════════════════════════════════════════════════════════════════════════╝");
        println!("  Controls: W/S = Move Up/Down, Q = Quit");
        if self.other_id.is_none() {
            println!("  ⏳ Waiting for another player to join...");
        }
        println!("  Frame: {} | Ball: ({}, {})", self.frame, self.ball_x, self.ball_y);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("🎮 Connecting to relay server at {}...", args.server);

    // Connect to relay server
    let backend = QuicClientBackend::connect_to_server(&args.server).await?;
    let my_id = backend.node_id().as_u64();

    println!("✅ Connected! Your Player ID: {}", my_id);
    println!("🎲 You are the {} player", if my_id % 2 == 0 { "HOST" } else { "CLIENT" });

    // Create EventBus with network
    let mut bus = EventBus::new().with_network(backend);
    bus.register_networked_event::<PaddleMove>();
    bus.register_networked_event::<BallUpdate>();

    // Create game state
    let is_host = my_id % 2 == 0; // Simple host selection
    let mut game = GameState::new(my_id, is_host);

    // Spawn input thread
    let (input_tx, mut input_rx) = tokio::sync::mpsc::channel::<Input>(10);
    tokio::task::spawn_blocking(move || {
        use std::io::Read;
        let mut stdin = std::io::stdin();
        let mut buf = [0u8; 1];

        loop {
            if stdin.read(&mut buf).is_ok() {
                match buf[0] {
                    b'w' | b'W' => { let _ = input_tx.blocking_send(Input::Up); }
                    b's' | b'S' => { let _ = input_tx.blocking_send(Input::Down); }
                    b'q' | b'Q' => { let _ = input_tx.blocking_send(Input::Quit); break; }
                    _ => {}
                }
            }
        }
    });

    // Game loop
    println!("🎮 Game starting in 2 seconds...");
    tokio::time::sleep(Duration::from_secs(2)).await;

    let mut running = true;
    while running {
        // Poll network
        bus.poll_network();

        // Process input
        while let Ok(input) = input_rx.try_recv() {
            match input {
                Input::Up => {
                    game.my_paddle_y = (game.my_paddle_y - 1).max(0);
                }
                Input::Down => {
                    game.my_paddle_y = (game.my_paddle_y + 1).min(16);
                }
                Input::Quit => {
                    running = false;
                }
            }
        }

        // Broadcast my paddle position
        bus.publish(PaddleMove {
            player_id: my_id,
            y_position: game.my_paddle_y,
        });

        // Dispatch events
        bus.dispatch();

        // Update game state
        game.update(&mut bus);

        // Render
        game.render();

        // 60 FPS
        tokio::time::sleep(Duration::from_millis(16)).await;
    }

    println!("\n👋 Thanks for playing!");
    Ok(())
}

// Simple random number generator (no external dependency)
mod rand {
    use std::cell::Cell;
    use std::time::{SystemTime, UNIX_EPOCH};

    thread_local! {
        static SEED: Cell<u64> = Cell::new(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64
        );
    }

    pub fn random<T: From<bool>>() -> T {
        SEED.with(|s| {
            let mut x = s.get();
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            s.set(x);
            T::from((x & 1) == 1)
        })
    }
}
