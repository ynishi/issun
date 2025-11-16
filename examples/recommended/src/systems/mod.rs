//! Business logic layer
//!
//! Pure functions that operate on entities and game state

pub mod combat_system;

// Example: Combat system
// Apply damage to a target
pub fn apply_damage(target_hp: i32, damage: i32) -> i32 {
    (target_hp - damage).max(0)
}

// Calculate damage (simple: attacker's attack value)
pub fn calculate_damage(attacker_attack: i32) -> i32 {
    attacker_attack
}
