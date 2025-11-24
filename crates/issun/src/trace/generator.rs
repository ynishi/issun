//! Graph generators for event chain visualization

use super::tracer::EventChainTracer;
use super::types::{TraceEntry, TraceEntryType};

impl EventChainTracer {
    /// Mermaid形式のグラフを生成
    pub fn generate_mermaid_graph(&self) -> String {
        self.generate_mermaid_for_all()
    }

    /// 特定フレーム範囲のMermaidグラフを生成
    pub fn generate_mermaid_for_frames(&self, start: u64, end: u64) -> String {
        let traces = self.traces_for_range(start, end);
        Self::generate_mermaid_from_traces(&traces)
    }

    /// 全トレースからMermaidグラフを生成
    fn generate_mermaid_for_all(&self) -> String {
        let traces: Vec<&TraceEntry> = self.traces().iter().collect();
        Self::generate_mermaid_from_traces(&traces)
    }

    /// トレースリストからMermaidグラフを生成
    fn generate_mermaid_from_traces(traces: &[&TraceEntry]) -> String {
        let mut graph = String::from("graph TD\n");
        let mut node_counter = 0;
        let mut prev_node: Option<String> = None;

        for entry in traces {
            let current_node = match &entry.entry_type {
                TraceEntryType::EventPublished { event_type, .. } => {
                    let node_id = format!("E{}", node_counter);
                    graph.push_str(&format!(
                        "    {}[\"📤 {}: {}\"]\n",
                        node_id, entry.source, event_type
                    ));
                    graph.push_str(&format!("    style {} fill:#e1f5ff\n", node_id));
                    node_counter += 1;
                    Some(node_id)
                }
                TraceEntryType::EventDispatched {
                    event_type,
                    subscriber_count,
                } => {
                    let node_id = format!("D{}", node_counter);
                    graph.push_str(&format!(
                        "    {}[\"📨 Dispatch {} ({} subscribers)\"]\n",
                        node_id, event_type, subscriber_count
                    ));
                    graph.push_str(&format!("    style {} fill:#e8f5e9\n", node_id));
                    node_counter += 1;
                    Some(node_id)
                }
                TraceEntryType::HookCalled {
                    hook_name, plugin, ..
                } => {
                    let node_id = format!("H{}", node_counter);
                    graph.push_str(&format!(
                        "    {}[\"🪝 {}::{}\"]\n",
                        node_id, plugin, hook_name
                    ));
                    graph.push_str(&format!("    style {} fill:#fff4e1\n", node_id));
                    node_counter += 1;
                    Some(node_id)
                }
                TraceEntryType::HookCompleted {
                    hook_name,
                    plugin,
                    duration_ms,
                    result,
                } => {
                    let node_id = format!("HC{}", node_counter);
                    let result_str = match result {
                        super::types::HookResult::Success => "✅",
                        super::types::HookResult::Error(_) => "❌",
                    };
                    graph.push_str(&format!(
                        "    {}[\"✓ {}::{} ({:.2}ms) {}\"]\n",
                        node_id, plugin, hook_name, duration_ms, result_str
                    ));
                    let fill_color = match result {
                        super::types::HookResult::Success => "#f1f8e9",
                        super::types::HookResult::Error(_) => "#ffebee",
                    };
                    graph.push_str(&format!("    style {} fill:{}\n", node_id, fill_color));
                    node_counter += 1;
                    Some(node_id)
                }
            };

            // 前のノードとエッジを作成
            if let (Some(prev), Some(current)) = (&prev_node, &current_node) {
                graph.push_str(&format!("    {} --> {}\n", prev, current));
            }

            prev_node = current_node;
        }

        graph
    }

    /// Graphviz DOT形式のグラフを生成
    pub fn generate_graphviz(&self) -> String {
        self.generate_graphviz_for_all()
    }

    /// 特定フレーム範囲のGraphvizグラフを生成
    pub fn generate_graphviz_for_frames(&self, start: u64, end: u64) -> String {
        let traces = self.traces_for_range(start, end);
        Self::generate_graphviz_from_traces(&traces)
    }

    /// 全トレースからGraphvizグラフを生成
    fn generate_graphviz_for_all(&self) -> String {
        let traces: Vec<&TraceEntry> = self.traces().iter().collect();
        Self::generate_graphviz_from_traces(&traces)
    }

    /// トレースリストからGraphvizグラフを生成
    fn generate_graphviz_from_traces(traces: &[&TraceEntry]) -> String {
        let mut graph = String::from("digraph EventChain {\n");
        graph.push_str("    rankdir=TD;\n");
        graph.push_str("    node [shape=box, style=filled];\n\n");

        let mut node_counter = 0;
        let mut prev_node: Option<String> = None;

        for entry in traces {
            let current_node = match &entry.entry_type {
                TraceEntryType::EventPublished { event_type, .. } => {
                    let node_id = format!("E{}", node_counter);
                    graph.push_str(&format!(
                        "    {} [label=\"📤 {}:\\n{}\", fillcolor=\"#e1f5ff\"];\n",
                        node_id, entry.source, event_type
                    ));
                    node_counter += 1;
                    Some(node_id)
                }
                TraceEntryType::EventDispatched {
                    event_type,
                    subscriber_count,
                } => {
                    let node_id = format!("D{}", node_counter);
                    graph.push_str(&format!(
                        "    {} [label=\"📨 Dispatch {}\\n({} subscribers)\", fillcolor=\"#e8f5e9\"];\n",
                        node_id, event_type, subscriber_count
                    ));
                    node_counter += 1;
                    Some(node_id)
                }
                TraceEntryType::HookCalled {
                    hook_name, plugin, ..
                } => {
                    let node_id = format!("H{}", node_counter);
                    graph.push_str(&format!(
                        "    {} [label=\"🪝 {}::\\n{}\", fillcolor=\"#fff4e1\"];\n",
                        node_id, plugin, hook_name
                    ));
                    node_counter += 1;
                    Some(node_id)
                }
                TraceEntryType::HookCompleted {
                    hook_name,
                    plugin,
                    duration_ms,
                    result,
                } => {
                    let node_id = format!("HC{}", node_counter);
                    let result_str = match result {
                        super::types::HookResult::Success => "✅",
                        super::types::HookResult::Error(_) => "❌",
                    };
                    let fill_color = match result {
                        super::types::HookResult::Success => "#f1f8e9",
                        super::types::HookResult::Error(_) => "#ffebee",
                    };
                    graph.push_str(&format!(
                        "    {} [label=\"✓ {}::{}\\n({:.2}ms) {}\", fillcolor=\"{}\"];\n",
                        node_id, plugin, hook_name, duration_ms, result_str, fill_color
                    ));
                    node_counter += 1;
                    Some(node_id)
                }
            };

            // 前のノードとエッジを作成
            if let (Some(prev), Some(current)) = (&prev_node, &current_node) {
                graph.push_str(&format!("    {} -> {};\n", prev, current));
            }

            prev_node = current_node;
        }

        graph.push_str("}\n");
        graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trace::types::TraceEntryType;

    #[test]
    fn test_generate_mermaid_empty() {
        let tracer = EventChainTracer::new();
        let mermaid = tracer.generate_mermaid_graph();

        assert_eq!(mermaid, "graph TD\n");
    }

    #[test]
    fn test_generate_mermaid_with_events() {
        let mut tracer = EventChainTracer::new();
        tracer.enable();

        tracer.record_simple(
            TraceEntryType::EventPublished {
                event_type: "TestEvent1".to_string(),
                event_id: "1".to_string(),
            },
            "EventBus",
        );

        tracer.record_simple(
            TraceEntryType::HookCalled {
                hook_name: "on_test".to_string(),
                plugin: "TestPlugin".to_string(),
                args: "()".to_string(),
            },
            "TestSystem",
        );

        let mermaid = tracer.generate_mermaid_graph();

        assert!(mermaid.contains("graph TD"));
        assert!(mermaid.contains("📤 EventBus: TestEvent1"));
        assert!(mermaid.contains("🪝 TestPlugin::on_test"));
        assert!(mermaid.contains("E0 --> H1"));
    }

    #[test]
    fn test_generate_mermaid_for_frames() {
        let mut tracer = EventChainTracer::new();
        tracer.enable();

        tracer.set_frame(0);
        tracer.record_simple(
            TraceEntryType::EventPublished {
                event_type: "Event1".to_string(),
                event_id: "1".to_string(),
            },
            "EventBus",
        );

        tracer.set_frame(1);
        tracer.record_simple(
            TraceEntryType::EventPublished {
                event_type: "Event2".to_string(),
                event_id: "2".to_string(),
            },
            "EventBus",
        );

        let mermaid = tracer.generate_mermaid_for_frames(0, 0);
        assert!(mermaid.contains("Event1"));
        assert!(!mermaid.contains("Event2"));

        let mermaid_all = tracer.generate_mermaid_for_frames(0, 1);
        assert!(mermaid_all.contains("Event1"));
        assert!(mermaid_all.contains("Event2"));
    }

    #[test]
    fn test_generate_graphviz_empty() {
        let tracer = EventChainTracer::new();
        let dot = tracer.generate_graphviz();

        assert!(dot.contains("digraph EventChain"));
        assert!(dot.contains("rankdir=TD"));
    }

    #[test]
    fn test_generate_graphviz_with_events() {
        let mut tracer = EventChainTracer::new();
        tracer.enable();

        tracer.record_simple(
            TraceEntryType::EventPublished {
                event_type: "TestEvent1".to_string(),
                event_id: "1".to_string(),
            },
            "EventBus",
        );

        tracer.record_simple(
            TraceEntryType::HookCalled {
                hook_name: "on_test".to_string(),
                plugin: "TestPlugin".to_string(),
                args: "()".to_string(),
            },
            "TestSystem",
        );

        let dot = tracer.generate_graphviz();

        assert!(dot.contains("digraph EventChain"));
        assert!(dot.contains("📤 EventBus"));
        assert!(dot.contains("🪝 TestPlugin"));
        assert!(dot.contains("E0 -> H1"));
    }
}
