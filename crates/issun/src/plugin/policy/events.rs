//! Policy events for command and state changes

use super::types::*;
use crate::event::Event;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request to activate a policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyActivateRequested {
    pub policy_id: PolicyId,
}

impl Event for PolicyActivateRequested {}

/// Request to deactivate the current policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDeactivateRequested {
    /// Optional: specific policy to deactivate (for multi-active mode)
    pub policy_id: Option<PolicyId>,
}

impl Event for PolicyDeactivateRequested {}

/// Request to cycle to the next policy (for games with policy cycling)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCycleRequested;

impl Event for PolicyCycleRequested {}

/// Published when a policy is activated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyActivatedEvent {
    pub policy_id: PolicyId,
    pub policy_name: String,
    pub effects: HashMap<String, f32>,
    pub previous_policy_id: Option<PolicyId>,
}

impl Event for PolicyActivatedEvent {}

/// Published when a policy is deactivated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDeactivatedEvent {
    pub policy_id: PolicyId,
    pub policy_name: String,
}

impl Event for PolicyDeactivatedEvent {}
