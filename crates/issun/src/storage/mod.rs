//! Storage and save/load system for ISSUN

pub mod json_repository;
pub mod repository;
pub mod ron_repository;
pub mod save_data;

pub use json_repository::JsonSaveRepository;
pub use repository::SaveRepository;
pub use ron_repository::RonSaveRepository;
pub use save_data::{SaveData, SaveMetadata};
