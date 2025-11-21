//! Room/Lobby system for organizing multiplayer games

use anyhow::Result;
use issun::network::NodeId;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Unique identifier for a room
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct RoomId(pub u64);

impl RoomId {
    #[allow(dead_code)]
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn random() -> Self {
        Self(rand::random())
    }

    #[allow(dead_code)]
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for RoomId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Room({})", self.0)
    }
}

/// Room state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoomState {
    /// Waiting for players to join
    Waiting,
    /// Game in progress
    InProgress,
    /// Game finished
    Finished,
}

/// A game room
#[derive(Debug, Clone)]
pub struct Room {
    /// Unique room ID
    #[allow(dead_code)]
    pub id: RoomId,

    /// Room name (optional)
    #[allow(dead_code)]
    pub name: Option<String>,

    /// Current state
    pub state: RoomState,

    /// Connected clients
    pub clients: HashSet<NodeId>,

    /// Maximum number of clients allowed
    pub max_clients: usize,

    /// Room creator (host)
    pub host: NodeId,

    /// Room metadata (game type, map, etc.)
    #[allow(dead_code)]
    pub metadata: HashMap<String, String>,

    /// Creation timestamp
    #[allow(dead_code)]
    pub created_at: std::time::SystemTime,
}

impl Room {
    /// Create a new room
    #[allow(dead_code)]
    pub fn new(host: NodeId, max_clients: usize) -> Self {
        let mut clients = HashSet::new();
        clients.insert(host);

        Self {
            id: RoomId::random(),
            name: None,
            state: RoomState::Waiting,
            clients,
            max_clients,
            host,
            metadata: HashMap::new(),
            created_at: std::time::SystemTime::now(),
        }
    }

    /// Check if room is full
    pub fn is_full(&self) -> bool {
        self.clients.len() >= self.max_clients
    }

    /// Check if room is empty
    pub fn is_empty(&self) -> bool {
        self.clients.is_empty()
    }

    /// Check if a client is in the room
    #[allow(dead_code)]
    pub fn contains(&self, client: NodeId) -> bool {
        self.clients.contains(&client)
    }

    /// Add a client to the room
    #[allow(dead_code)]
    pub fn add_client(&mut self, client: NodeId) -> Result<()> {
        if self.is_full() {
            anyhow::bail!("Room is full");
        }

        if self.state != RoomState::Waiting {
            anyhow::bail!("Room is not accepting new players");
        }

        self.clients.insert(client);
        Ok(())
    }

    /// Remove a client from the room
    pub fn remove_client(&mut self, client: NodeId) -> bool {
        self.clients.remove(&client)
    }

    /// Start the game
    pub fn start(&mut self) -> Result<()> {
        if self.state != RoomState::Waiting {
            anyhow::bail!("Room is not in waiting state");
        }

        self.state = RoomState::InProgress;
        Ok(())
    }

    /// Finish the game
    #[allow(dead_code)]
    pub fn finish(&mut self) {
        self.state = RoomState::Finished;
    }
}

/// Room manager for organizing multiplayer games
pub struct RoomManager {
    /// Active rooms
    rooms: Arc<RwLock<HashMap<RoomId, Room>>>,

    /// Client to room mapping
    client_rooms: Arc<RwLock<HashMap<NodeId, RoomId>>>,
}

impl RoomManager {
    /// Create a new room manager
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
            client_rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new room
    #[allow(dead_code)]
    pub async fn create_room(
        &self,
        host: NodeId,
        max_clients: usize,
        name: Option<String>,
    ) -> Result<RoomId> {
        // Check if host is already in a room
        {
            let client_rooms = self.client_rooms.read().await;
            if client_rooms.contains_key(&host) {
                anyhow::bail!("Client is already in a room");
            }
        }

        let mut room = Room::new(host, max_clients);
        room.name = name;
        let room_id = room.id;

        // Add room and update mappings
        {
            let mut rooms = self.rooms.write().await;
            rooms.insert(room_id, room);
        }

        {
            let mut client_rooms = self.client_rooms.write().await;
            client_rooms.insert(host, room_id);
        }

        info!("Room created: {} by host {:?}", room_id, host);
        Ok(room_id)
    }

    /// Join an existing room
    #[allow(dead_code)]
    pub async fn join_room(&self, room_id: RoomId, client: NodeId) -> Result<()> {
        // Check if client is already in a room
        {
            let client_rooms = self.client_rooms.read().await;
            if client_rooms.contains_key(&client) {
                anyhow::bail!("Client is already in a room");
            }
        }

        // Add client to room
        {
            let mut rooms = self.rooms.write().await;
            let room = rooms
                .get_mut(&room_id)
                .ok_or_else(|| anyhow::anyhow!("Room not found"))?;

            room.add_client(client)?;
        }

        {
            let mut client_rooms = self.client_rooms.write().await;
            client_rooms.insert(client, room_id);
        }

        info!("Client {:?} joined room {}", client, room_id);
        Ok(())
    }

    /// Leave current room
    pub async fn leave_room(&self, client: NodeId) -> Result<()> {
        let room_id = {
            let mut client_rooms = self.client_rooms.write().await;
            client_rooms
                .remove(&client)
                .ok_or_else(|| anyhow::anyhow!("Client is not in any room"))?
        };

        let should_delete_room = {
            let mut rooms = self.rooms.write().await;
            if let Some(room) = rooms.get_mut(&room_id) {
                room.remove_client(client);

                // If room is empty or host left, mark for deletion
                room.is_empty() || room.host == client
            } else {
                false
            }
        };

        if should_delete_room {
            self.delete_room(room_id).await?;
            debug!("Room {} deleted (empty or host left)", room_id);
        }

        info!("Client {:?} left room {}", client, room_id);
        Ok(())
    }

    /// Get room information
    #[allow(dead_code)]
    pub async fn get_room(&self, room_id: RoomId) -> Option<Room> {
        let rooms = self.rooms.read().await;
        rooms.get(&room_id).cloned()
    }

    /// Get client's current room ID
    pub async fn get_client_room(&self, client: NodeId) -> Option<RoomId> {
        let client_rooms = self.client_rooms.read().await;
        client_rooms.get(&client).copied()
    }

    /// Get clients in the same room as the given client
    pub async fn get_room_clients(&self, client: NodeId) -> Vec<NodeId> {
        let room_id = {
            let client_rooms = self.client_rooms.read().await;
            match client_rooms.get(&client) {
                Some(id) => *id,
                None => return Vec::new(),
            }
        };

        let rooms = self.rooms.read().await;
        rooms
            .get(&room_id)
            .map(|room| room.clients.iter().copied().collect())
            .unwrap_or_default()
    }

    /// List all available rooms
    #[allow(dead_code)]
    pub async fn list_rooms(&self) -> Vec<Room> {
        let rooms = self.rooms.read().await;
        rooms
            .values()
            .filter(|room| room.state == RoomState::Waiting)
            .cloned()
            .collect()
    }

    /// Start a game in a room
    #[allow(dead_code)]
    pub async fn start_game(&self, room_id: RoomId, requester: NodeId) -> Result<()> {
        let mut rooms = self.rooms.write().await;
        let room = rooms
            .get_mut(&room_id)
            .ok_or_else(|| anyhow::anyhow!("Room not found"))?;

        // Only host can start the game
        if room.host != requester {
            anyhow::bail!("Only the host can start the game");
        }

        room.start()?;
        info!("Game started in room {}", room_id);
        Ok(())
    }

    /// Delete a room
    async fn delete_room(&self, room_id: RoomId) -> Result<()> {
        let mut rooms = self.rooms.write().await;
        rooms
            .remove(&room_id)
            .ok_or_else(|| anyhow::anyhow!("Room not found"))?;

        // Remove all client mappings for this room
        let mut client_rooms = self.client_rooms.write().await;
        client_rooms.retain(|_, rid| *rid != room_id);

        Ok(())
    }

    /// Clean up rooms for a disconnected client
    pub async fn handle_disconnect(&self, client: NodeId) {
        if let Err(e) = self.leave_room(client).await {
            warn!("Failed to remove disconnected client from room: {}", e);
        }
    }
}

impl Default for RoomManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_join_room() {
        let manager = RoomManager::new();
        let host = NodeId::from_u64(1);
        let client = NodeId::from_u64(2);

        // Create room
        let room_id = manager
            .create_room(host, 4, Some("Test Room".to_string()))
            .await
            .unwrap();

        // Join room
        manager.join_room(room_id, client).await.unwrap();

        // Verify
        let room = manager.get_room(room_id).await.unwrap();
        assert_eq!(room.clients.len(), 2);
        assert!(room.contains(host));
        assert!(room.contains(client));
    }

    #[tokio::test]
    async fn test_leave_room() {
        let manager = RoomManager::new();
        let host = NodeId::from_u64(1);

        let room_id = manager.create_room(host, 4, None).await.unwrap();
        manager.leave_room(host).await.unwrap();

        // Room should be deleted
        assert!(manager.get_room(room_id).await.is_none());
    }

    #[tokio::test]
    async fn test_room_full() {
        let manager = RoomManager::new();
        let host = NodeId::from_u64(1);

        let room_id = manager.create_room(host, 2, None).await.unwrap();

        let client2 = NodeId::from_u64(2);
        manager.join_room(room_id, client2).await.unwrap();

        // Room is now full
        let client3 = NodeId::from_u64(3);
        assert!(manager.join_room(room_id, client3).await.is_err());
    }

    #[tokio::test]
    async fn test_list_rooms() {
        let manager = RoomManager::new();

        manager
            .create_room(NodeId::from_u64(1), 4, Some("Room 1".to_string()))
            .await
            .unwrap();
        manager
            .create_room(NodeId::from_u64(2), 2, Some("Room 2".to_string()))
            .await
            .unwrap();

        let rooms = manager.list_rooms().await;
        assert_eq!(rooms.len(), 2);
    }
}
