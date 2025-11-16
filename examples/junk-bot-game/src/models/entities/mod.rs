//! Game entities
//!
//! Pure data structures representing game objects

mod player;
mod enemy;
mod weapon;
mod bot;
mod rarity;
mod loot;
mod buff_card;
mod room_buff;
mod room;
mod dungeon;

pub use player::Player;
pub use enemy::Enemy;
pub use weapon::{Weapon, WeaponEffect};
pub use bot::{Bot, BotState};
pub use rarity::Rarity;
pub use loot::{LootItem, ItemEffect, generate_random_loot};
pub use buff_card::{BuffCard, BuffType, generate_random_cards};
pub use room_buff::RoomBuff;
pub use room::{Room, RoomType, generate_random_rooms};
pub use dungeon::{Dungeon, Floor4Choice};
