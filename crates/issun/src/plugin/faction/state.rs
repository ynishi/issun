//! Faction runtime state (Mutable)

use super::types::*;
use crate::state::State;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Faction runtime state (Mutable)
///
/// Contains operation information that changes during gameplay.
/// This is a save/load target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionState {
    /// Active operations
    operations: HashMap<OperationId, Operation>,
}

impl State for FactionState {}

impl FactionState {
    /// Create a new empty state
    pub fn new() -> Self {
        Self {
            operations: HashMap::new(),
        }
    }

    // ========================================
    // Operation Management
    // ========================================

    /// Launch a new operation
    ///
    /// # Returns
    ///
    /// `Ok(())` if operation was launched successfully,
    /// `Err(FactionError)` if operation already exists.
    pub fn launch_operation(&mut self, operation: Operation) -> Result<(), FactionError> {
        // Verify operation doesn't already exist
        if self.operations.contains_key(&operation.id) {
            return Err(FactionError::OperationAlreadyExists);
        }

        self.operations.insert(operation.id.clone(), operation);
        Ok(())
    }

    /// Get operation by id
    pub fn get_operation(&self, id: &OperationId) -> Option<&Operation> {
        self.operations.get(id)
    }

    /// Get mutable operation by id
    pub fn get_operation_mut(&mut self, id: &OperationId) -> Option<&mut Operation> {
        self.operations.get_mut(id)
    }

    /// List all operations
    pub fn operations(&self) -> impl Iterator<Item = &Operation> {
        self.operations.values()
    }

    /// List operations for a specific faction
    pub fn operations_for_faction<'a>(
        &'a self,
        faction_id: &'a FactionId,
    ) -> impl Iterator<Item = &'a Operation> + 'a {
        self.operations
            .values()
            .filter(move |op| &op.faction_id == faction_id)
    }

    /// List operations with a specific status
    pub fn operations_with_status(
        &self,
        status: OperationStatus,
    ) -> impl Iterator<Item = &Operation> {
        self.operations
            .values()
            .filter(move |op| op.status == status)
    }

    /// Update operation status
    ///
    /// # Returns
    ///
    /// `Ok(())` if status was updated successfully,
    /// `Err(FactionError)` if operation not found.
    pub fn update_operation_status(
        &mut self,
        id: &OperationId,
        status: OperationStatus,
    ) -> Result<(), FactionError> {
        let operation = self
            .operations
            .get_mut(id)
            .ok_or(FactionError::OperationNotFound)?;

        operation.status = status;
        Ok(())
    }

    /// Complete an operation
    ///
    /// Sets operation status to `Completed`.
    pub fn complete_operation(&mut self, id: &OperationId) -> Result<(), FactionError> {
        self.update_operation_status(id, OperationStatus::Completed)
    }

    /// Fail an operation
    ///
    /// Sets operation status to `Failed`.
    pub fn fail_operation(&mut self, id: &OperationId) -> Result<(), FactionError> {
        self.update_operation_status(id, OperationStatus::Failed)
    }

    /// Remove a completed or failed operation
    ///
    /// This is useful for cleanup of old operations.
    ///
    /// # Returns
    ///
    /// `Some(operation)` if operation was removed,
    /// `None` if operation not found or still in progress.
    pub fn remove_operation(&mut self, id: &OperationId) -> Option<Operation> {
        // Only remove if completed or failed
        if let Some(op) = self.operations.get(id) {
            if op.is_completed() || op.is_failed() {
                return self.operations.remove(id);
            }
        }
        None
    }

    /// Count total operations
    pub fn operation_count(&self) -> usize {
        self.operations.len()
    }

    /// Count operations for a specific faction
    pub fn operation_count_for_faction(&self, faction_id: &FactionId) -> usize {
        self.operations
            .values()
            .filter(|op| &op.faction_id == faction_id)
            .count()
    }

    /// Clear all operations
    pub fn clear(&mut self) {
        self.operations.clear();
    }
}

impl Default for FactionState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_new() {
        let state = FactionState::new();
        assert_eq!(state.operation_count(), 0);
    }

    #[test]
    fn test_launch_operation() {
        let mut state = FactionState::new();
        let op = Operation::new("op-001", FactionId::new("crimson"), "Capture Nova");

        let result = state.launch_operation(op);
        assert!(result.is_ok());
        assert_eq!(state.operation_count(), 1);
    }

    #[test]
    fn test_launch_operation_already_exists() {
        let mut state = FactionState::new();
        let op1 = Operation::new("op-001", FactionId::new("crimson"), "Test 1");
        let op2 = Operation::new("op-001", FactionId::new("crimson"), "Test 2");

        state.launch_operation(op1).unwrap();
        let result = state.launch_operation(op2);

        assert_eq!(result, Err(FactionError::OperationAlreadyExists));
    }

    #[test]
    fn test_complete_operation() {
        let mut state = FactionState::new();
        let op = Operation::new("op-001", FactionId::new("crimson"), "Test");
        state.launch_operation(op).unwrap();

        state
            .complete_operation(&OperationId::new("op-001"))
            .unwrap();

        let op = state.get_operation(&OperationId::new("op-001")).unwrap();
        assert!(op.is_completed());
    }

    #[test]
    fn test_fail_operation() {
        let mut state = FactionState::new();
        let op = Operation::new("op-001", FactionId::new("crimson"), "Test");
        state.launch_operation(op).unwrap();

        state.fail_operation(&OperationId::new("op-001")).unwrap();

        let op = state.get_operation(&OperationId::new("op-001")).unwrap();
        assert!(op.is_failed());
    }

    #[test]
    fn test_operations_for_faction() {
        let mut state = FactionState::new();

        state
            .launch_operation(Operation::new("op-001", FactionId::new("crimson"), "Test 1"))
            .unwrap();
        state
            .launch_operation(Operation::new("op-002", FactionId::new("crimson"), "Test 2"))
            .unwrap();
        state
            .launch_operation(Operation::new("op-003", FactionId::new("azure"), "Test 3"))
            .unwrap();

        let crimson_id = FactionId::new("crimson");
        let crimson_ops: Vec<_> = state.operations_for_faction(&crimson_id).collect();
        assert_eq!(crimson_ops.len(), 2);
    }

    #[test]
    fn test_operations_with_status() {
        let mut state = FactionState::new();

        state
            .launch_operation(Operation::new("op-001", FactionId::new("crimson"), "Test 1"))
            .unwrap();
        state
            .launch_operation(Operation::new("op-002", FactionId::new("crimson"), "Test 2"))
            .unwrap();

        state
            .complete_operation(&OperationId::new("op-001"))
            .unwrap();

        let pending: Vec<_> = state
            .operations_with_status(OperationStatus::Pending)
            .collect();
        assert_eq!(pending.len(), 1);

        let completed: Vec<_> = state
            .operations_with_status(OperationStatus::Completed)
            .collect();
        assert_eq!(completed.len(), 1);
    }

    #[test]
    fn test_remove_operation() {
        let mut state = FactionState::new();

        state
            .launch_operation(Operation::new("op-001", FactionId::new("crimson"), "Test"))
            .unwrap();

        // Cannot remove pending operation
        assert!(state
            .remove_operation(&OperationId::new("op-001"))
            .is_none());

        // Can remove after completion
        state
            .complete_operation(&OperationId::new("op-001"))
            .unwrap();
        assert!(state
            .remove_operation(&OperationId::new("op-001"))
            .is_some());
        assert_eq!(state.operation_count(), 0);
    }

    #[test]
    fn test_operation_count_for_faction() {
        let mut state = FactionState::new();

        state
            .launch_operation(Operation::new("op-001", FactionId::new("crimson"), "Test 1"))
            .unwrap();
        state
            .launch_operation(Operation::new("op-002", FactionId::new("crimson"), "Test 2"))
            .unwrap();
        state
            .launch_operation(Operation::new("op-003", FactionId::new("azure"), "Test 3"))
            .unwrap();

        assert_eq!(
            state.operation_count_for_faction(&FactionId::new("crimson")),
            2
        );
        assert_eq!(
            state.operation_count_for_faction(&FactionId::new("azure")),
            1
        );
    }

    #[test]
    fn test_clear() {
        let mut state = FactionState::new();
        state
            .launch_operation(Operation::new("op-001", FactionId::new("crimson"), "Test"))
            .unwrap();

        state.clear();
        assert_eq!(state.operation_count(), 0);
    }
}
