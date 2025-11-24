//! Validation of event flows and system dependencies

use crate::types::AnalysisResult;
use std::collections::{HashMap, HashSet};

/// Validation warning categories
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationWarning {
    /// Event is published but never subscribed
    UnusedEvent {
        event_type: String,
        publishers: Vec<String>,
    },

    /// Event is subscribed but never published
    MissingPublisher {
        event_type: String,
        subscribers: Vec<String>,
    },

    /// Potential circular dependency in event flow
    PotentialEventLoop { cycle: Vec<String> },

    /// System subscribes to the same event multiple times
    DuplicateSubscription { system: String, event_type: String },
}

impl ValidationWarning {
    /// Get the severity level of this warning
    pub fn severity(&self) -> WarningSeverity {
        match self {
            ValidationWarning::UnusedEvent { .. } => WarningSeverity::Low,
            ValidationWarning::MissingPublisher { .. } => WarningSeverity::Medium,
            ValidationWarning::PotentialEventLoop { .. } => WarningSeverity::High,
            ValidationWarning::DuplicateSubscription { .. } => WarningSeverity::Low,
        }
    }

    /// Format warning as human-readable string
    pub fn format(&self) -> String {
        match self {
            ValidationWarning::UnusedEvent {
                event_type,
                publishers,
            } => {
                format!(
                    "⚠️  Event '{}' is published but never subscribed\n   Publishers: {}",
                    event_type,
                    publishers.join(", ")
                )
            }
            ValidationWarning::MissingPublisher {
                event_type,
                subscribers,
            } => {
                format!(
                    "⚠️  Event '{}' is subscribed but never published\n   Subscribers: {}",
                    event_type,
                    subscribers.join(", ")
                )
            }
            ValidationWarning::PotentialEventLoop { cycle } => {
                format!(
                    "⚠️  Potential event loop detected:\n   {}",
                    cycle.join(" → ")
                )
            }
            ValidationWarning::DuplicateSubscription { system, event_type } => {
                format!(
                    "⚠️  System '{}' subscribes to event '{}' multiple times",
                    system, event_type
                )
            }
        }
    }
}

/// Warning severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WarningSeverity {
    Low,
    Medium,
    High,
}

/// Validation result containing all warnings
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            warnings: Vec::new(),
        }
    }

    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }

    /// Get warnings by severity level
    pub fn warnings_by_severity(&self, severity: WarningSeverity) -> Vec<&ValidationWarning> {
        self.warnings
            .iter()
            .filter(|w| w.severity() == severity)
            .collect()
    }

    /// Check if there are any high-severity warnings
    pub fn has_high_severity_warnings(&self) -> bool {
        self.warnings
            .iter()
            .any(|w| w.severity() == WarningSeverity::High)
    }

    /// Print formatted report
    pub fn print_report(&self) {
        if self.warnings.is_empty() {
            println!("✅ No validation warnings found!");
            return;
        }

        println!("⚠️  Validation Warnings ({})\n", self.warnings.len());

        // Group by severity
        let high = self.warnings_by_severity(WarningSeverity::High);
        let medium = self.warnings_by_severity(WarningSeverity::Medium);
        let low = self.warnings_by_severity(WarningSeverity::Low);

        if !high.is_empty() {
            println!("🔴 High Severity ({}):", high.len());
            for warning in high {
                println!("{}\n", warning.format());
            }
        }

        if !medium.is_empty() {
            println!("🟡 Medium Severity ({}):", medium.len());
            for warning in medium {
                println!("{}\n", warning.format());
            }
        }

        if !low.is_empty() {
            println!("🟢 Low Severity ({}):", low.len());
            for warning in low {
                println!("{}\n", warning.format());
            }
        }
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Validator for analyzing event flows and dependencies
pub struct Validator<'a> {
    result: &'a AnalysisResult,
}

impl<'a> Validator<'a> {
    pub fn new(result: &'a AnalysisResult) -> Self {
        Self { result }
    }

    /// Run all validation checks
    pub fn validate(&self) -> ValidationResult {
        let mut validation = ValidationResult::new();

        // Check for unused events
        self.check_unused_events(&mut validation);

        // Check for missing publishers
        self.check_missing_publishers(&mut validation);

        // Check for duplicate subscriptions
        self.check_duplicate_subscriptions(&mut validation);

        // Check for potential event loops
        self.check_event_loops(&mut validation);

        validation
    }

    /// Check for events that are published but never subscribed
    fn check_unused_events(&self, validation: &mut ValidationResult) {
        let mut published_events: HashMap<String, Vec<String>> = HashMap::new();
        let mut subscribed_events: HashSet<String> = HashSet::new();

        // Collect published events
        for publication in self.result.all_publications() {
            published_events
                .entry(publication.event_type.clone())
                .or_default()
                .push(publication.publisher.clone());
        }

        // Collect subscribed events
        for subscription in self.result.all_subscriptions() {
            subscribed_events.insert(subscription.event_type.clone());
        }

        // Find unused events
        for (event_type, publishers) in published_events {
            if !subscribed_events.contains(&event_type) {
                validation.add_warning(ValidationWarning::UnusedEvent {
                    event_type,
                    publishers,
                });
            }
        }
    }

    /// Check for events that are subscribed but never published
    fn check_missing_publishers(&self, validation: &mut ValidationResult) {
        let mut subscribed_events: HashMap<String, Vec<String>> = HashMap::new();
        let mut published_events: HashSet<String> = HashSet::new();

        // Collect subscribed events
        for subscription in self.result.all_subscriptions() {
            subscribed_events
                .entry(subscription.event_type.clone())
                .or_default()
                .push(subscription.subscriber.clone());
        }

        // Collect published events
        for publication in self.result.all_publications() {
            published_events.insert(publication.event_type.clone());
        }

        // Find missing publishers
        for (event_type, subscribers) in subscribed_events {
            if !published_events.contains(&event_type) {
                validation.add_warning(ValidationWarning::MissingPublisher {
                    event_type,
                    subscribers,
                });
            }
        }
    }

    /// Check for duplicate subscriptions in the same system
    fn check_duplicate_subscriptions(&self, validation: &mut ValidationResult) {
        let mut system_subscriptions: HashMap<String, HashSet<String>> = HashMap::new();

        for subscription in self.result.all_subscriptions() {
            let subscriber = &subscription.subscriber;
            let event_type = &subscription.event_type;

            let events = system_subscriptions.entry(subscriber.clone()).or_default();

            if events.contains(event_type) {
                validation.add_warning(ValidationWarning::DuplicateSubscription {
                    system: subscriber.clone(),
                    event_type: event_type.clone(),
                });
            } else {
                events.insert(event_type.clone());
            }
        }
    }

    /// Check for potential circular dependencies in event flow
    fn check_event_loops(&self, validation: &mut ValidationResult) {
        // Build dependency graph: System -> Events it publishes -> Systems that subscribe
        let mut system_dependencies: HashMap<String, Vec<String>> = HashMap::new();
        let mut event_to_publishers: HashMap<String, Vec<String>> = HashMap::new();
        let mut event_to_subscribers: HashMap<String, Vec<String>> = HashMap::new();

        // Collect event publishers
        for publication in self.result.all_publications() {
            event_to_publishers
                .entry(publication.event_type.clone())
                .or_default()
                .push(publication.publisher.clone());
        }

        // Collect event subscribers
        for subscription in self.result.all_subscriptions() {
            event_to_subscribers
                .entry(subscription.event_type.clone())
                .or_default()
                .push(subscription.subscriber.clone());
        }

        // Build system dependency graph
        for (event_type, publishers) in &event_to_publishers {
            if let Some(subscribers) = event_to_subscribers.get(event_type) {
                for publisher in publishers {
                    for subscriber in subscribers {
                        // Skip self-dependencies
                        if publisher != subscriber {
                            system_dependencies
                                .entry(publisher.clone())
                                .or_default()
                                .push(subscriber.clone());
                        }
                    }
                }
            }
        }

        // Detect cycles using DFS
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for system in system_dependencies.keys() {
            if !visited.contains(system) {
                if let Some(cycle) = self.detect_cycle_dfs(
                    system,
                    &system_dependencies,
                    &mut visited,
                    &mut rec_stack,
                    &mut vec![system.clone()],
                ) {
                    validation.add_warning(ValidationWarning::PotentialEventLoop { cycle });
                }
            }
        }
    }

    /// DFS-based cycle detection
    fn detect_cycle_dfs(
        &self,
        node: &str,
        graph: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    path.push(neighbor.clone());
                    if let Some(cycle) =
                        self.detect_cycle_dfs(neighbor, graph, visited, rec_stack, path)
                    {
                        return Some(cycle);
                    }
                    path.pop();
                } else if rec_stack.contains(neighbor) {
                    // Found a cycle
                    let cycle_start = path.iter().position(|n| n == neighbor).unwrap();
                    let mut cycle = path[cycle_start..].to_vec();
                    cycle.push(neighbor.clone()); // Close the cycle
                    return Some(cycle);
                }
            }
        }

        rec_stack.remove(node);
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AnalysisResult, EventPublication, EventSubscription, FileAnalysis};

    #[test]
    fn test_unused_event_detection() {
        let mut result = AnalysisResult::new();

        // Add a published event with no subscribers
        let file = FileAnalysis {
            path: "test.rs".to_string(),
            subscriptions: vec![],
            publications: vec![EventPublication {
                publisher: "TestSystem".to_string(),
                event_type: "UnusedEvent".to_string(),
                file_path: "test.rs".to_string(),
                line: 10,
            }],
        };
        result.add_file(file);

        let validator = Validator::new(&result);
        let validation = validator.validate();

        assert_eq!(validation.warnings.len(), 1);
        assert!(matches!(
            validation.warnings[0],
            ValidationWarning::UnusedEvent { .. }
        ));
    }

    #[test]
    fn test_missing_publisher_detection() {
        let mut result = AnalysisResult::new();

        // Add a subscribed event with no publishers
        let file = FileAnalysis {
            path: "test.rs".to_string(),
            subscriptions: vec![EventSubscription {
                subscriber: "TestSystem".to_string(),
                event_type: "MissingEvent".to_string(),
                file_path: "test.rs".to_string(),
                line: 10,
            }],
            publications: vec![],
        };
        result.add_file(file);

        let validator = Validator::new(&result);
        let validation = validator.validate();

        assert_eq!(validation.warnings.len(), 1);
        assert!(matches!(
            validation.warnings[0],
            ValidationWarning::MissingPublisher { .. }
        ));
    }

    #[test]
    fn test_no_warnings_for_complete_flow() {
        let mut result = AnalysisResult::new();

        // Add a complete event flow (published and subscribed)
        let file = FileAnalysis {
            path: "test.rs".to_string(),
            subscriptions: vec![EventSubscription {
                subscriber: "SubscriberSystem".to_string(),
                event_type: "TestEvent".to_string(),
                file_path: "test.rs".to_string(),
                line: 10,
            }],
            publications: vec![EventPublication {
                publisher: "PublisherSystem".to_string(),
                event_type: "TestEvent".to_string(),
                file_path: "test.rs".to_string(),
                line: 20,
            }],
        };
        result.add_file(file);

        let validator = Validator::new(&result);
        let validation = validator.validate();

        assert_eq!(validation.warnings.len(), 0);
    }
}
