//! Mermaid graph generation for Event and Hook flows

use crate::types::{AnalysisResult, PluginInfo};
use std::collections::{HashMap, HashSet};

/// Options for graph generation
#[derive(Debug, Clone)]
pub struct GraphOptions {
    /// Include only specific plugins (empty = all)
    pub filter_plugins: Vec<String>,
    /// Include only specific event types (empty = all)
    pub filter_events: Vec<String>,
    /// Show hook calls in graph
    pub show_hooks: bool,
    /// Maximum number of nodes to display
    pub max_nodes: Option<usize>,
}

impl Default for GraphOptions {
    fn default() -> Self {
        Self {
            filter_plugins: Vec::new(),
            filter_events: Vec::new(),
            show_hooks: true,
            max_nodes: None,
        }
    }
}

/// Generate Mermaid graph for Event flow
pub struct EventFlowGraphGenerator<'a> {
    result: &'a AnalysisResult,
    options: GraphOptions,
}

impl<'a> EventFlowGraphGenerator<'a> {
    pub fn new(result: &'a AnalysisResult) -> Self {
        Self {
            result,
            options: GraphOptions::default(),
        }
    }

    pub fn with_options(result: &'a AnalysisResult, options: GraphOptions) -> Self {
        Self { result, options }
    }

    /// Generate Mermaid flowchart for event flow
    pub fn generate(&self) -> String {
        let mut graph = String::from("flowchart TD\n");
        graph.push_str("    %% Event Flow Diagram\n\n");

        // Build event flow map
        let event_flows = self.build_event_flows();

        // Filter events if specified
        let events_to_show: Vec<&String> = if self.options.filter_events.is_empty() {
            event_flows.keys().collect()
        } else {
            event_flows
                .keys()
                .filter(|e| self.options.filter_events.contains(e))
                .collect()
        };

        // Apply max_nodes limit
        let events_to_show: Vec<&&String> = if let Some(max) = self.options.max_nodes {
            events_to_show.iter().take(max).collect()
        } else {
            events_to_show.iter().collect()
        };

        // Generate nodes and edges for each event
        for event_type in events_to_show {
            let flow = &event_flows[*event_type];

            // Event node (center)
            let event_node = self.sanitize_id(event_type);
            graph.push_str(&format!("    {}[\"📨 {}\"]\n", event_node, event_type));
            graph.push_str(&format!("    style {} fill:#e1f5ff\n", event_node));

            // Publishers → Event
            for publisher in &flow.publishers {
                let pub_node = self.sanitize_id(publisher);
                graph.push_str(&format!("    {}[\"{}\"]\n", pub_node, publisher));
                graph.push_str(&format!("    {} -->|publish| {}\n", pub_node, event_node));
            }

            // Event → Subscribers
            for subscriber in &flow.subscribers {
                let sub_node = self.sanitize_id(subscriber);
                graph.push_str(&format!("    {}[\"{}\"]\n", sub_node, subscriber));
                graph.push_str(&format!("    {} -->|subscribe| {}\n", event_node, sub_node));
            }

            graph.push('\n');
        }

        // Add legend
        graph.push_str("    %% Legend\n");
        graph.push_str("    subgraph Legend\n");
        graph.push_str("        L1[\"📨 Event\"]\n");
        graph.push_str("        L2[\"System/Plugin\"]\n");
        graph.push_str("        style L1 fill:#e1f5ff\n");
        graph.push_str("    end\n");

        graph
    }

    /// Build event flow map from analysis result
    fn build_event_flows(&self) -> HashMap<String, EventFlow> {
        let mut flows: HashMap<String, EventFlow> = HashMap::new();

        // Collect subscriptions
        for subscription in self.result.all_subscriptions() {
            flows
                .entry(subscription.event_type.clone())
                .or_insert_with(|| EventFlow::new(subscription.event_type.clone()))
                .add_subscriber(&subscription.subscriber);
        }

        // Collect publications
        for publication in self.result.all_publications() {
            flows
                .entry(publication.event_type.clone())
                .or_insert_with(|| EventFlow::new(publication.event_type.clone()))
                .add_publisher(&publication.publisher);
        }

        flows
    }

    /// Sanitize identifier for Mermaid (replace special chars)
    fn sanitize_id(&self, s: &str) -> String {
        s.replace("::", "_")
            .replace(['<', '>', ' '], "_")
    }
}

/// Generate Mermaid graph for Hook flow
pub struct HookFlowGraphGenerator<'a> {
    result: &'a AnalysisResult,
    options: GraphOptions,
}

impl<'a> HookFlowGraphGenerator<'a> {
    pub fn new(result: &'a AnalysisResult) -> Self {
        Self {
            result,
            options: GraphOptions::default(),
        }
    }

    pub fn with_options(result: &'a AnalysisResult, options: GraphOptions) -> Self {
        Self { result, options }
    }

    /// Generate Mermaid flowchart for hook dependencies
    pub fn generate(&self) -> String {
        let mut graph = String::from("flowchart TD\n");
        graph.push_str("    %% Hook Flow Diagram\n\n");

        // Filter plugins if specified
        let plugins_to_show: Vec<&PluginInfo> = if self.options.filter_plugins.is_empty() {
            self.result.plugins.iter().collect()
        } else {
            self.result
                .plugins
                .iter()
                .filter(|p| self.options.filter_plugins.contains(&p.name))
                .collect()
        };

        // Generate subgraph for each plugin
        for plugin in plugins_to_show {
            if plugin.hook_details.is_empty() {
                continue;
            }

            let plugin_id = self.sanitize_id(&plugin.name);

            graph.push_str(&format!(
                "    subgraph {}[\"{} Plugin\"]\n",
                plugin_id, plugin.name
            ));

            // System node (if exists)
            if let Some(system) = &plugin.system {
                let system_id = format!("{}_{}", plugin_id, self.sanitize_id(&system.name));
                graph.push_str(&format!("        {}[\"⚙️ {}\"]\n", system_id, system.name));

                // Hook nodes
                for hook_info in &plugin.hook_details {
                    let hook_id =
                        format!("{}_{}", plugin_id, self.sanitize_id(&hook_info.trait_name));
                    graph.push_str(&format!(
                        "        {}[\"🪝 {}\"]\n",
                        hook_id, hook_info.trait_name
                    ));
                    graph.push_str(&format!("        style {} fill:#fff4e6\n", hook_id));

                    // System uses Hook
                    graph.push_str(&format!("        {} -.->|uses| {}\n", system_id, hook_id));

                    // Hook methods (collapsed)
                    let method_count = hook_info.methods.len();
                    let notification_count = hook_info
                        .methods
                        .iter()
                        .filter(|m| matches!(m.category, crate::types::HookCategory::Notification))
                        .count();
                    let validation_count = hook_info
                        .methods
                        .iter()
                        .filter(|m| matches!(m.category, crate::types::HookCategory::Validation))
                        .count();

                    if method_count > 0 {
                        let methods_id = format!("{}_methods", hook_id);
                        let summary = format!(
                            "{} methods ({} notif, {} valid)",
                            method_count, notification_count, validation_count
                        );
                        graph.push_str(&format!("        {}[\"📋 {}\"]\n", methods_id, summary));
                        graph.push_str(&format!("        {} -.-> {}\n", hook_id, methods_id));
                    }
                }
            }

            graph.push_str("    end\n\n");
        }

        // Add legend
        graph.push_str("    %% Legend\n");
        graph.push_str("    subgraph Legend\n");
        graph.push_str("        L1[\"⚙️ System\"]\n");
        graph.push_str("        L2[\"🪝 Hook Trait\"]\n");
        graph.push_str("        L3[\"📋 Methods\"]\n");
        graph.push_str("        style L2 fill:#fff4e6\n");
        graph.push_str("    end\n");

        graph
    }

    /// Sanitize identifier for Mermaid
    fn sanitize_id(&self, s: &str) -> String {
        s.replace("::", "_")
            .replace(['<', '>', ' ', '-'], "_")
    }
}

/// Generate combined Event + Hook flow graph
pub struct CombinedFlowGraphGenerator<'a> {
    result: &'a AnalysisResult,
    options: GraphOptions,
}

impl<'a> CombinedFlowGraphGenerator<'a> {
    pub fn new(result: &'a AnalysisResult) -> Self {
        Self {
            result,
            options: GraphOptions::default(),
        }
    }

    pub fn with_options(result: &'a AnalysisResult, options: GraphOptions) -> Self {
        Self { result, options }
    }

    /// Generate combined Mermaid flowchart
    pub fn generate(&self) -> String {
        let mut graph = String::from("flowchart TD\n");
        graph.push_str("    %% Combined Event + Hook Flow Diagram\n\n");

        // Filter plugins
        let plugins_to_show: Vec<&PluginInfo> = if self.options.filter_plugins.is_empty() {
            self.result.plugins.iter().collect()
        } else {
            self.result
                .plugins
                .iter()
                .filter(|p| self.options.filter_plugins.contains(&p.name))
                .collect()
        };

        for plugin in plugins_to_show {
            if plugin.system.is_none() {
                continue;
            }

            let system = plugin.system.as_ref().unwrap();
            let plugin_id = self.sanitize_id(&plugin.name);

            graph.push_str(&format!(
                "    subgraph {}[\"{} Plugin\"]\n",
                plugin_id, plugin.name
            ));

            // System node
            let system_id = format!("{}_{}", plugin_id, self.sanitize_id(&system.name));
            graph.push_str(&format!("        {}[\"⚙️ {}\"]\n", system_id, system.name));

            // Subscribed events
            for event_type in &system.subscribes {
                let event_id = format!("{}_{}_sub", plugin_id, self.sanitize_id(event_type));
                graph.push_str(&format!("        {}[\"📨 {}\"]\n", event_id, event_type));
                graph.push_str(&format!("        style {} fill:#e1f5ff\n", event_id));
                graph.push_str(&format!(
                    "        {} -->|subscribe| {}\n",
                    event_id, system_id
                ));
            }

            // Published events
            for event_type in &system.publishes {
                let event_id = format!("{}_{}_pub", plugin_id, self.sanitize_id(event_type));
                graph.push_str(&format!("        {}[\"📤 {}\"]\n", event_id, event_type));
                graph.push_str(&format!("        style {} fill:#e8f5e9\n", event_id));
                graph.push_str(&format!(
                    "        {} -->|publish| {}\n",
                    system_id, event_id
                ));
            }

            // Hooks (if enabled)
            if self.options.show_hooks {
                for hook in &system.hooks {
                    let hook_id = format!("{}_{}", plugin_id, self.sanitize_id(hook));
                    graph.push_str(&format!("        {}[\"🪝 {}\"]\n", hook_id, hook));
                    graph.push_str(&format!("        style {} fill:#fff4e6\n", hook_id));
                    graph.push_str(&format!("        {} -.->|uses| {}\n", system_id, hook_id));
                }
            }

            graph.push_str("    end\n\n");
        }

        // Add legend
        graph.push_str("    %% Legend\n");
        graph.push_str("    subgraph Legend\n");
        graph.push_str("        L1[\"⚙️ System\"]\n");
        graph.push_str("        L2[\"📨 Subscribed Event\"]\n");
        graph.push_str("        L3[\"📤 Published Event\"]\n");
        graph.push_str("        L4[\"🪝 Hook\"]\n");
        graph.push_str("        style L2 fill:#e1f5ff\n");
        graph.push_str("        style L3 fill:#e8f5e9\n");
        graph.push_str("        style L4 fill:#fff4e6\n");
        graph.push_str("    end\n");

        graph
    }

    fn sanitize_id(&self, s: &str) -> String {
        s.replace("::", "_")
            .replace(['<', '>', ' ', '-'], "_")
    }
}

/// Event flow data structure
#[derive(Debug, Clone)]
struct EventFlow {
    #[allow(dead_code)]
    event_type: String,
    publishers: HashSet<String>,
    subscribers: HashSet<String>,
}

impl EventFlow {
    fn new(event_type: String) -> Self {
        Self {
            event_type,
            publishers: HashSet::new(),
            subscribers: HashSet::new(),
        }
    }

    fn add_publisher(&mut self, publisher: &str) {
        self.publishers.insert(publisher.to_string());
    }

    fn add_subscriber(&mut self, subscriber: &str) {
        self.subscribers.insert(subscriber.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AnalysisResult, EventSubscription, FileAnalysis};

    #[test]
    fn test_event_flow_generator() {
        let mut result = AnalysisResult::new();

        // Add test data
        let file = FileAnalysis {
            path: "test.rs".to_string(),
            subscriptions: vec![EventSubscription {
                subscriber: "TestSystem".to_string(),
                event_type: "TestEvent".to_string(),
                file_path: "test.rs".to_string(),
                line: 10,
            }],
            publications: vec![],
        };

        result.add_file(file);

        let generator = EventFlowGraphGenerator::new(&result);
        let graph = generator.generate();

        assert!(graph.contains("flowchart TD"));
        assert!(graph.contains("TestEvent"));
        assert!(graph.contains("TestSystem"));
    }

    #[test]
    fn test_sanitize_id() {
        let result = AnalysisResult::new();
        let generator = EventFlowGraphGenerator::new(&result);

        assert_eq!(generator.sanitize_id("Foo::Bar"), "Foo_Bar");
        assert_eq!(generator.sanitize_id("Vec<String>"), "Vec_String_");
    }
}
