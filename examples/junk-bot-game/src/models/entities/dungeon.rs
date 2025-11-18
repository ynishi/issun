use super::{generate_random_rooms, Enemy, Room, RoomBuff, RoomType};
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Dungeon with 5 fixed floors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dungeon {
    /// All rooms in the dungeon (5 rooms)
    pub rooms: Vec<Room>,
    /// Current room index (0-4)
    pub current_room: usize,
}

impl Dungeon {
    /// Create a new 5-floor dungeon
    pub fn new() -> Self {
        let mut rooms = Vec::new();

        // FLOOR 1: Tutorial - Easy start
        rooms.push(generate_floor1_room());

        // FLOOR 2: Standard Combat
        rooms.push(generate_floor2_room());

        // FLOOR 3: Mini-Boss
        rooms.push(generate_floor3_miniboss());

        // FLOOR 4: Room Selection (player choice) - placeholder
        rooms.push(Room::new_with_buff(
            RoomType::Combat,
            RoomBuff::Normal,
            "FLOOR 4: Choice",
        ));

        // FLOOR 5: Final Boss
        rooms.push(generate_floor5_boss());

        Self {
            rooms,
            current_room: 0,
        }
    }

    /// Get current room
    pub fn get_current_room(&self) -> Option<&Room> {
        self.rooms.get(self.current_room)
    }

    /// Get mutable current room
    pub fn get_current_room_mut(&mut self) -> Option<&mut Room> {
        self.rooms.get_mut(self.current_room)
    }

    /// Advance to next room, returns true if successful
    pub fn advance(&mut self) -> bool {
        if self.current_room < self.rooms.len() - 1 {
            self.current_room += 1;
            true
        } else {
            false
        }
    }

    /// Check if dungeon is complete
    pub fn is_complete(&self) -> bool {
        self.current_room >= self.rooms.len() - 1
    }

    /// Check if we're at floor 4 and need room selection
    pub fn needs_floor4_selection(&self) -> bool {
        if self.current_room == 3 {
            if let Some(room) = self.rooms.get(3) {
                room.name == "FLOOR 4: Choice"
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Set floor 4 choice
    pub fn set_floor4_choice(&mut self, choice: Floor4Choice) {
        if self.current_room == 3 {
            self.rooms[3] = match choice {
                Floor4Choice::Easy => generate_floor4_easy(),
                Floor4Choice::Normal => generate_floor4_normal(),
                Floor4Choice::Hard => generate_floor4_hard(),
            };
        }
    }

    /// Get current floor number (1-5)
    pub fn current_floor_number(&self) -> usize {
        self.current_room + 1
    }

    /// Get total floor count
    pub fn total_floors(&self) -> usize {
        self.rooms.len()
    }
}

impl Default for Dungeon {
    fn default() -> Self {
        Self::new()
    }
}

/// Floor 4 choice options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Floor4Choice {
    /// Easy path - fewer enemies, normal rewards
    Easy,
    /// Normal path - standard difficulty
    Normal,
    /// Hard path - more enemies, better rewards
    Hard,
}

impl Floor4Choice {
    pub fn description(&self) -> &str {
        match self {
            Floor4Choice::Easy => "Safe route - Fewer enemies, standard loot",
            Floor4Choice::Normal => "Balanced route - Normal difficulty",
            Floor4Choice::Hard => "Risky route - More enemies, better loot!",
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Floor4Choice::Easy => "Easy Path",
            Floor4Choice::Normal => "Normal Path",
            Floor4Choice::Hard => "Hard Path",
        }
    }
}

// Floor generation functions

fn generate_floor1_room() -> Room {
    let mut rng = rand::thread_rng();
    let enemy_names = ["Training Bot", "Scrap Drone"];
    let enemies = vec![Enemy::new(
        enemy_names[rng.gen_range(0..enemy_names.len())],
        20,
        3,
    )];
    Room::new_combat(RoomBuff::Normal, enemies)
}

fn generate_floor2_room() -> Room {
    let rooms = generate_random_rooms(1, 2);
    let mut room = rooms.into_iter().next().unwrap();
    room.name = "FLOOR 2: Combat".to_string();
    room
}

fn generate_floor3_miniboss() -> Room {
    let mut rng = rand::thread_rng();
    let boss_names = ["Corrupted Guardian", "Scrap Titan", "Junk Overlord"];
    let miniboss = Enemy::new(boss_names[rng.gen_range(0..boss_names.len())], 80, 12);
    let mut room = Room::new_combat(RoomBuff::Normal, vec![miniboss]);
    room.name = "FLOOR 3: Mini-Boss".to_string();
    room
}

fn generate_floor4_easy() -> Room {
    let mut rng = rand::thread_rng();
    let enemy_names = ["Weak Bot", "Damaged Drone"];
    let enemies = vec![Enemy::new(
        enemy_names[rng.gen_range(0..enemy_names.len())],
        30,
        6,
    )];
    let mut room = Room::new_combat(RoomBuff::Normal, enemies);
    room.name = "FLOOR 4: Easy Path".to_string();
    room
}

fn generate_floor4_normal() -> Room {
    let rooms = generate_random_rooms(1, 4);
    let mut room = rooms.into_iter().next().unwrap();
    room.name = "FLOOR 4: Normal Path".to_string();
    room
}

fn generate_floor4_hard() -> Room {
    let rooms = generate_random_rooms(1, 4);
    let mut room = rooms.into_iter().next().unwrap();
    // Double the enemies
    let extra_enemies = room.enemies.clone();
    room.enemies.extend(extra_enemies);
    room.name = "FLOOR 4: Hard Path".to_string();
    room.buff = RoomBuff::Contaminated; // Extra challenge!
    room
}

fn generate_floor5_boss() -> Room {
    let mut rng = rand::thread_rng();
    let boss_names = [
        "The Corrupted Core",
        "Master of Scrap",
        "Final Guardian",
        "Omega Destroyer",
    ];
    let boss = Enemy::new(boss_names[rng.gen_range(0..boss_names.len())], 150, 20);
    let mut room = Room::new_combat(RoomBuff::Normal, vec![boss]);
    room.name = "FLOOR 5: FINAL BOSS".to_string();
    room
}
