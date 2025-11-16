//! Storage and save/load system for ISSUN

pub mod repository;
pub mod save_data;
pub mod json_repository;
pub mod ron_repository;

pub use repository::SaveRepository;
pub use save_data::{SaveData, SaveMetadata};
pub use json_repository::JsonSaveRepository;
pub use ron_repository::RonSaveRepository;
