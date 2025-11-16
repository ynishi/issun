//! Combat system - battle logic

use crate::models::entities::{Player, Enemy};

/// Execute player attack
pub fn player_attack(player: &Player, enemy: &mut Enemy) -> String {
    let damage = player.attack;
    enemy.take_damage(damage);
    format!("{} attacks {} for {} damage!", player.name, enemy.name, damage)
}

/// Execute enemy attack
pub fn enemy_attack(enemy: &Enemy, player: &mut Player) -> String {
    let damage = enemy.attack;
    player.take_damage(damage);
    format!("{} attacks {} for {} damage!", enemy.name, player.name, damage)
}

/// Check if battle is over
pub fn is_battle_over(player: &Player, enemies: &[Enemy]) -> bool {
    !player.is_alive() || enemies.iter().all(|e| !e.is_alive())
}
