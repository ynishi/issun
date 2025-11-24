//! Core analyzer for parsing Rust source files

use crate::error::{AnalyzerError, Result};
use crate::types::FileAnalysis;
use std::path::Path;
use syn::File;

/// Main analyzer struct for processing Rust files
pub struct Analyzer {
    /// Root directory for analysis
    root_path: std::path::PathBuf,
}

impl Analyzer {
    /// Create a new analyzer for the given root directory
    pub fn new<P: AsRef<Path>>(root_path: P) -> Self {
        Self {
            root_path: root_path.as_ref().to_path_buf(),
        }
    }

    /// Analyze a single Rust source file
    pub fn analyze_file<P: AsRef<Path>>(&self, file_path: P) -> Result<FileAnalysis> {
        let path = file_path.as_ref();
        let path_str = path.to_string_lossy().to_string();

        // Read file content
        let content = std::fs::read_to_string(path).map_err(|e| AnalyzerError::FileReadError {
            path: path_str.clone(),
            source: e,
        })?;

        // Parse as Rust syntax tree
        let syntax_tree = syn::parse_file(&content).map_err(|e| AnalyzerError::ParseError {
            path: path_str.clone(),
            source: e,
        })?;

        // Extract information from the syntax tree
        self.analyze_syntax_tree(&path_str, &syntax_tree)
    }

    /// Analyze a parsed syntax tree
    fn analyze_syntax_tree(&self, file_path: &str, syntax_tree: &File) -> Result<FileAnalysis> {
        // Extract from struct fields: EventReader<E>
        let mut subscriptions =
            crate::event_extractor::extract_event_readers(file_path, syntax_tree);

        // Extract from method calls: bus.reader::<E>()
        let reader_calls = crate::event_extractor::extract_reader_calls(file_path, syntax_tree);
        subscriptions.extend(reader_calls);

        // Extract event publications
        let publications =
            crate::event_extractor::extract_event_publications(file_path, syntax_tree);

        Ok(FileAnalysis {
            path: file_path.to_string(),
            subscriptions,
            publications,
        })
    }

    /// Analyze a file and extract System implementations with event information
    pub fn analyze_systems<P: AsRef<Path>>(
        &self,
        file_path: P,
    ) -> Result<Vec<crate::types::SystemInfo>> {
        let path = file_path.as_ref();
        let path_str = path.to_string_lossy().to_string();

        // Read file content
        let content = std::fs::read_to_string(path).map_err(|e| {
            crate::error::AnalyzerError::FileReadError {
                path: path_str.clone(),
                source: e,
            }
        })?;

        // Parse as Rust syntax tree
        let syntax_tree =
            syn::parse_file(&content).map_err(|e| crate::error::AnalyzerError::ParseError {
                path: path_str.clone(),
                source: e,
            })?;

        // Extract systems and enrich with field info
        let mut systems = crate::system_extractor::extract_systems(&path_str, &syntax_tree);

        // Extract event subscriptions and publications
        let file_analysis = self.analyze_syntax_tree(&path_str, &syntax_tree)?;

        // Enrich systems with all information
        for system in &mut systems {
            // Field information (hooks, states)
            let field_info =
                crate::system_extractor::extract_system_fields(&syntax_tree, &system.name);
            system.hooks = field_info.hooks;
            system.states = field_info.states;

            // Event subscriptions - collect all unique event types from this file
            let subscribes: Vec<String> = file_analysis
                .subscriptions
                .iter()
                .map(|sub| sub.event_type.clone())
                .collect();
            system.subscribes = subscribes;
            system.subscribes.sort();
            system.subscribes.dedup();

            // Event publications - collect all unique event types from this file
            let publishes: Vec<String> = file_analysis
                .publications
                .iter()
                .map(|pub_event| pub_event.event_type.clone())
                .collect();
            system.publishes = publishes;
            system.publishes.sort();
            system.publishes.dedup();
        }

        Ok(systems)
    }

    /// Get the root path
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = Analyzer::new(".");
        assert!(analyzer.root_path().exists());
    }
}
