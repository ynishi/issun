//! Hello ISSUN - Minimal example

use issun::prelude::*;
use issun::ui::{TitleScreenAsset, AsciiFont};
use issun::engine::GameRng;

fn main() -> Result<()> {
    println!("=== Hello ISSUN (一寸) ===\n");

    // Create a title screen
    let title = TitleScreenAsset::new("ISSUN")
        .with_subtitle("A Mini Game Engine")
        .with_font(AsciiFont::Preset("minimal"));

    println!("{}", issun::ui::TitleScreenService::render(&title, 0));

    // Test RNG
    let mut rng = GameRng::new(42);
    println!("\n=== RNG Test ===");
    println!("Roll d6: {}", rng.roll(6));
    println!("Roll d20: {}", rng.roll(20));
    println!("Chance 50%: {}", rng.chance(0.5));

    // Test Input Mapper
    println!("\n=== Input Mapper Test ===");
    let mapper = issun::engine::InputMapper::new();
    use crossterm::event::KeyCode;
    println!("Up key maps to: {:?}", mapper.map_key(KeyCode::Up));
    println!("k key maps to: {:?}", mapper.map_key(KeyCode::Char('k')));

    println!("\n✅ ISSUN engine loaded successfully!");
    println!("Ready to build games in ISSUN (一寸) of time.\n");

    Ok(())
}
