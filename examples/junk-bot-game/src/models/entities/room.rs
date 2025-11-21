use super::{Enemy, RoomBuff};
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Room types in the dungeon
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RoomType {
    /// Combat encounter
    Combat,
    /// Treasure room (not implemented yet)
    Treasure,
    /// Boss encounter (not implemented yet)
    Boss,
}

/// A room in the dungeon with enemies and buffs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub room_type: RoomType,
    pub buff: RoomBuff,
    pub enemies: Vec<Enemy>,
    pub name: String,
}

impl Room {
    /// Create a new room with default settings
    #[allow(dead_code)]
    pub fn new(room_type: RoomType) -> Self {
        Self {
            room_type,
            buff: RoomBuff::Normal,
            enemies: Vec::new(),
            name: "Unknown Room".to_string(),
        }
    }

    /// Create a new room with specific buff and name
    pub fn new_with_buff(room_type: RoomType, buff: RoomBuff, name: impl Into<String>) -> Self {
        Self {
            room_type,
            buff,
            enemies: Vec::new(),
            name: name.into(),
        }
    }

    /// Create a combat room with enemies
    pub fn new_combat(buff: RoomBuff, enemies: Vec<Enemy>) -> Self {
        let name = format!("{} Combat Room", buff.name());
        Self {
            room_type: RoomType::Combat,
            buff,
            enemies,
            name,
        }
    }
}

/// Generate random rooms for selection
pub fn generate_random_rooms(count: usize, floor: u32) -> Vec<Room> {
    let mut rng = rand::thread_rng();
    let mut rooms = Vec::with_capacity(count);

    // Available buffs
    let buffs = [
        RoomBuff::Normal,
        RoomBuff::Narrow,
        RoomBuff::Wide,
        RoomBuff::Contaminated,
    ];

    for _ in 0..count {
        let buff = buffs[rng.gen_range(0..buffs.len())].clone();
        let room = generate_combat_room(buff, floor);
        rooms.push(room);
    }

    rooms
}

/// Generate a single combat room with enemies
fn generate_combat_room(buff: RoomBuff, floor: u32) -> Room {
    let mut rng = rand::thread_rng();

    // Base enemy count (1-3)
    let base_count = rng.gen_range(1..=3);

    // Apply room buff modifier
    let enemy_count = (base_count as i32 + buff.enemy_count_modifier()).max(1) as usize;

    // Difficulty multiplier based on floor
    let difficulty = 1.0 + (floor as f32 * 0.2);

    // Generate enemies
    let mut enemies = Vec::new();
    for _i in 0..enemy_count {
        let base_hp = 30 + rng.gen_range(0..20);
        let base_attack = 5 + rng.gen_range(0..5);

        let scaled_hp = (base_hp as f32 * difficulty) as i32;
        let scaled_attack = (base_attack as f32 * difficulty) as i32;

        let enemy_names = [
            "Rust Monster",
            "Corrupted Bot",
            "Junk Golem",
            "Scrap Stalker",
        ];
        let name = enemy_names[rng.gen_range(0..enemy_names.len())];

        enemies.push(Enemy::new(name, scaled_hp, scaled_attack));
    }

    Room::new_combat(buff, enemies)
}
