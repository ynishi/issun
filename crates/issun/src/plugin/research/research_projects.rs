//! Research project definitions (ReadOnly asset)

use super::types::*;
use crate::resources::Resource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Collection of research project definitions (ReadOnly)
///
/// This is an asset loaded at startup and does not change during gameplay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchProjects {
    projects: HashMap<ResearchId, ResearchProject>,
}

impl Resource for ResearchProjects {}

impl ResearchProjects {
    /// Create a new empty projects collection
    pub fn new() -> Self {
        Self {
            projects: HashMap::new(),
        }
    }

    /// Add a research project definition
    pub fn define(&mut self, project: ResearchProject) {
        self.projects.insert(project.id.clone(), project);
    }

    /// Get a project by id
    pub fn get(&self, id: &ResearchId) -> Option<&ResearchProject> {
        self.projects.get(id)
    }

    /// Check if a project exists
    pub fn contains(&self, id: &ResearchId) -> bool {
        self.projects.contains_key(id)
    }

    /// List all projects
    pub fn iter(&self) -> impl Iterator<Item = &ResearchProject> {
        self.projects.values()
    }

    /// Get the number of projects
    pub fn len(&self) -> usize {
        self.projects.len()
    }

    /// Check if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.projects.is_empty()
    }
}

impl Default for ResearchProjects {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_projects_new() {
        let projects = ResearchProjects::new();
        assert!(projects.is_empty());
        assert_eq!(projects.len(), 0);
    }

    #[test]
    fn test_define_and_get() {
        let mut projects = ResearchProjects::new();
        let project = ResearchProject::new("test", "Test Project", "Test description");
        projects.define(project);

        assert_eq!(projects.len(), 1);
        assert!(!projects.is_empty());
        assert!(projects.get(&ResearchId::new("test")).is_some());
        assert_eq!(
            projects.get(&ResearchId::new("test")).unwrap().name,
            "Test Project"
        );
    }

    #[test]
    fn test_contains() {
        let mut projects = ResearchProjects::new();
        projects.define(ResearchProject::new("test", "Test", "Test"));

        assert!(projects.contains(&ResearchId::new("test")));
        assert!(!projects.contains(&ResearchId::new("other")));
    }

    #[test]
    fn test_iter() {
        let mut projects = ResearchProjects::new();
        projects.define(ResearchProject::new("test1", "Test 1", "Test"));
        projects.define(ResearchProject::new("test2", "Test 2", "Test"));

        let count = projects.iter().count();
        assert_eq!(count, 2);
    }
}
