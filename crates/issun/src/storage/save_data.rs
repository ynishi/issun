//! Save data structures for ISSUN

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Save data container
///
/// Generic over the game context type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    /// Save data version (for migration)
    pub version: u32,

    /// Save slot name
    pub slot: String,

    /// Timestamp when saved
    #[serde(with = "system_time_serde")]
    pub timestamp: SystemTime,

    /// Game-specific data (serialized as JSON/RON)
    pub data: serde_json::Value,
}

impl SaveData {
    /// Create a new save data
    pub fn new(slot: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            version: 1,
            slot: slot.into(),
            timestamp: SystemTime::now(),
            data,
        }
    }

    /// Create from typed context
    pub fn from_context<T: Serialize>(
        slot: impl Into<String>,
        context: &T,
    ) -> Result<Self, serde_json::Error> {
        let data = serde_json::to_value(context)?;
        Ok(Self::new(slot, data))
    }

    /// Deserialize into typed context
    pub fn into_context<T: for<'de> Deserialize<'de>>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.data.clone())
    }
}

/// Save metadata (lightweight info without full data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMetadata {
    pub version: u32,
    pub slot: String,
    #[serde(with = "system_time_serde")]
    pub timestamp: SystemTime,
    pub size_bytes: u64,
}

impl SaveMetadata {
    pub fn from_save_data(data: &SaveData, size_bytes: u64) -> Self {
        Self {
            version: data.version,
            slot: data.slot.clone(),
            timestamp: data.timestamp,
            size_bytes,
        }
    }
}

// SystemTime serialization helpers
mod system_time_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time
            .duration_since(UNIX_EPOCH)
            .map_err(serde::ser::Error::custom)?;
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + std::time::Duration::from_secs(secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestContext {
        score: u32,
        level: u32,
    }

    #[test]
    fn test_save_data_creation() {
        let ctx = TestContext {
            score: 100,
            level: 5,
        };
        let save = SaveData::from_context("slot1", &ctx).unwrap();

        assert_eq!(save.version, 1);
        assert_eq!(save.slot, "slot1");
    }

    #[test]
    fn test_save_data_roundtrip() {
        let ctx = TestContext {
            score: 100,
            level: 5,
        };
        let save = SaveData::from_context("slot1", &ctx).unwrap();
        let loaded: TestContext = save.into_context().unwrap();

        assert_eq!(ctx, loaded);
    }
}
