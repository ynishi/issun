//! Event log resource for capturing game events

use std::collections::VecDeque;

/// Maximum number of log entries to keep
const MAX_LOG_ENTRIES: usize = 100;

/// Event log for displaying game events in TUI
#[derive(Debug, Clone)]
pub struct EventLog {
    entries: VecDeque<String>,
}

impl EventLog {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::with_capacity(MAX_LOG_ENTRIES),
        }
    }

    /// Add a new log entry
    pub fn log(&mut self, message: String) {
        if self.entries.len() >= MAX_LOG_ENTRIES {
            self.entries.pop_front();
        }
        self.entries.push_back(message);
    }

    /// Get last N entries (most recent last)
    pub fn last_n(&self, n: usize) -> Vec<&String> {
        self.entries.iter().rev().take(n).rev().collect()
    }
}

impl Default for EventLog {
    fn default() -> Self {
        Self::new()
    }
}
