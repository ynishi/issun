//! Faction registry for managing factions and operations

use super::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Registry of all factions and their operations
///
/// This is the central Resource for faction management.
/// It stores all factions and operations in the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionRegistry {
    factions: HashMap<FactionId, Faction>,
    operations: HashMap<OperationId, Operation>,
}

impl FactionRegistry {
    /// Create a new empty faction registry
    pub fn new() -> Self {
        Self {
            factions: HashMap::new(),
            operations: HashMap::new(),
        }
    }

    // ========================================
    // Faction Management
    // ========================================

    /// Add a new faction
    ///
    /// # Example
    ///
    /// ```
    /// use issun::plugin::faction::{FactionRegistry, Faction};
    ///
    /// let mut registry = FactionRegistry::new();
    /// registry.add_faction(Faction::new("crimson", "Crimson Syndicate"));
    /// ```
    pub fn add_faction(&mut self, faction: Faction) {
        self.factions.insert(faction.id.clone(), faction);
    }

    /// Get faction by id
    pub fn get_faction(&self, id: &FactionId) -> Option<&Faction> {
        self.factions.get(id)
    }

    /// Get mutable faction by id
    pub fn get_faction_mut(&mut self, id: &FactionId) -> Option<&mut Faction> {
        self.factions.get_mut(id)
    }

    /// List all factions
    pub fn factions(&self) -> impl Iterator<Item = &Faction> {
        self.factions.values()
    }

    /// Remove a faction
    ///
    /// # Note
    ///
    /// This does NOT remove associated operations.
    /// Consider cleaning up operations separately if needed.
    pub fn remove_faction(&mut self, id: &FactionId) -> Option<Faction> {
        self.factions.remove(id)
    }

    // ========================================
    // Operation Management
    // ========================================

    /// Launch a new operation
    ///
    /// # Returns
    ///
    /// `Ok(())` if operation was launched successfully,
    /// `Err(FactionError)` if faction not found or operation already exists.
    ///
    /// # Example
    ///
    /// ```
    /// use issun::plugin::faction::{FactionRegistry, Faction, Operation, FactionId};
    ///
    /// let mut registry = FactionRegistry::new();
    /// registry.add_faction(Faction::new("crimson", "Crimson Syndicate"));
    ///
    /// let op = Operation::new("op-001", FactionId::new("crimson"), "Capture Nova Harbor");
    /// registry.launch_operation(op).unwrap();
    /// ```
    pub fn launch_operation(&mut self, operation: Operation) -> Result<(), FactionError> {
        // Verify faction exists
        if !self.factions.contains_key(&operation.faction_id) {
            return Err(FactionError::FactionNotFound);
        }

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
    ///
    /// # Returns
    ///
    /// `Ok(())` if operation was completed successfully,
    /// `Err(FactionError)` if operation not found.
    pub fn complete_operation(&mut self, id: &OperationId) -> Result<(), FactionError> {
        self.update_operation_status(id, OperationStatus::Completed)
    }

    /// Fail an operation
    ///
    /// Sets operation status to `Failed`.
    ///
    /// # Returns
    ///
    /// `Ok(())` if operation was failed successfully,
    /// `Err(FactionError)` if operation not found.
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
}

impl Default for FactionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_get_faction() {
        let mut registry = FactionRegistry::new();
        let faction = Faction::new("crimson", "Crimson Syndicate");

        registry.add_faction(faction.clone());

        let retrieved = registry.get_faction(&FactionId::new("crimson")).unwrap();
        assert_eq!(retrieved.name, "Crimson Syndicate");
    }

    #[test]
    fn test_get_faction_mut() {
        let mut registry = FactionRegistry::new();
        registry.add_faction(Faction::new("crimson", "Crimson Syndicate"));

        {
            let faction = registry.get_faction_mut(&FactionId::new("crimson")).unwrap();
            faction.name = "Updated Name".to_string();
        }

        let faction = registry.get_faction(&FactionId::new("crimson")).unwrap();
        assert_eq!(faction.name, "Updated Name");
    }

    #[test]
    fn test_launch_operation() {
        let mut registry = FactionRegistry::new();
        registry.add_faction(Faction::new("crimson", "Crimson Syndicate"));

        let op = Operation::new("op-001", FactionId::new("crimson"), "Capture Nova");
        let result = registry.launch_operation(op);

        assert!(result.is_ok());
        assert_eq!(registry.operation_count(), 1);
    }

    #[test]
    fn test_launch_operation_faction_not_found() {
        let mut registry = FactionRegistry::new();
        let op = Operation::new("op-001", FactionId::new("nonexistent"), "Test");

        let result = registry.launch_operation(op);
        assert_eq!(result, Err(FactionError::FactionNotFound));
    }

    #[test]
    fn test_launch_operation_already_exists() {
        let mut registry = FactionRegistry::new();
        registry.add_faction(Faction::new("crimson", "Crimson Syndicate"));

        let op1 = Operation::new("op-001", FactionId::new("crimson"), "Test 1");
        let op2 = Operation::new("op-001", FactionId::new("crimson"), "Test 2");

        registry.launch_operation(op1).unwrap();
        let result = registry.launch_operation(op2);

        assert_eq!(result, Err(FactionError::OperationAlreadyExists));
    }

    #[test]
    fn test_complete_operation() {
        let mut registry = FactionRegistry::new();
        registry.add_faction(Faction::new("crimson", "Crimson Syndicate"));

        let op = Operation::new("op-001", FactionId::new("crimson"), "Test");
        registry.launch_operation(op).unwrap();

        registry.complete_operation(&OperationId::new("op-001")).unwrap();

        let op = registry.get_operation(&OperationId::new("op-001")).unwrap();
        assert!(op.is_completed());
    }

    #[test]
    fn test_fail_operation() {
        let mut registry = FactionRegistry::new();
        registry.add_faction(Faction::new("crimson", "Crimson Syndicate"));

        let op = Operation::new("op-001", FactionId::new("crimson"), "Test");
        registry.launch_operation(op).unwrap();

        registry.fail_operation(&OperationId::new("op-001")).unwrap();

        let op = registry.get_operation(&OperationId::new("op-001")).unwrap();
        assert!(op.is_failed());
    }

    #[test]
    fn test_operations_for_faction() {
        let mut registry = FactionRegistry::new();
        registry.add_faction(Faction::new("crimson", "Crimson Syndicate"));
        registry.add_faction(Faction::new("azure", "Azure Collective"));

        registry
            .launch_operation(Operation::new("op-001", FactionId::new("crimson"), "Test 1"))
            .unwrap();
        registry
            .launch_operation(Operation::new("op-002", FactionId::new("crimson"), "Test 2"))
            .unwrap();
        registry
            .launch_operation(Operation::new("op-003", FactionId::new("azure"), "Test 3"))
            .unwrap();

        let crimson_id = FactionId::new("crimson");
        let crimson_ops: Vec<_> = registry.operations_for_faction(&crimson_id).collect();
        assert_eq!(crimson_ops.len(), 2);
    }

    #[test]
    fn test_operations_with_status() {
        let mut registry = FactionRegistry::new();
        registry.add_faction(Faction::new("crimson", "Crimson Syndicate"));

        registry
            .launch_operation(Operation::new("op-001", FactionId::new("crimson"), "Test 1"))
            .unwrap();
        registry
            .launch_operation(Operation::new("op-002", FactionId::new("crimson"), "Test 2"))
            .unwrap();

        registry.complete_operation(&OperationId::new("op-001")).unwrap();

        let pending: Vec<_> = registry
            .operations_with_status(OperationStatus::Pending)
            .collect();
        assert_eq!(pending.len(), 1);

        let completed: Vec<_> = registry
            .operations_with_status(OperationStatus::Completed)
            .collect();
        assert_eq!(completed.len(), 1);
    }

    #[test]
    fn test_remove_operation() {
        let mut registry = FactionRegistry::new();
        registry.add_faction(Faction::new("crimson", "Crimson Syndicate"));

        registry
            .launch_operation(Operation::new("op-001", FactionId::new("crimson"), "Test"))
            .unwrap();

        // Cannot remove pending operation
        assert!(registry.remove_operation(&OperationId::new("op-001")).is_none());

        // Can remove after completion
        registry.complete_operation(&OperationId::new("op-001")).unwrap();
        assert!(registry.remove_operation(&OperationId::new("op-001")).is_some());
        assert_eq!(registry.operation_count(), 0);
    }

    #[test]
    fn test_remove_faction() {
        let mut registry = FactionRegistry::new();
        registry.add_faction(Faction::new("crimson", "Crimson Syndicate"));

        let removed = registry.remove_faction(&FactionId::new("crimson"));
        assert!(removed.is_some());
        assert!(registry.get_faction(&FactionId::new("crimson")).is_none());
    }

    #[test]
    fn test_operation_count_for_faction() {
        let mut registry = FactionRegistry::new();
        registry.add_faction(Faction::new("crimson", "Crimson Syndicate"));
        registry.add_faction(Faction::new("azure", "Azure Collective"));

        registry
            .launch_operation(Operation::new("op-001", FactionId::new("crimson"), "Test 1"))
            .unwrap();
        registry
            .launch_operation(Operation::new("op-002", FactionId::new("crimson"), "Test 2"))
            .unwrap();
        registry
            .launch_operation(Operation::new("op-003", FactionId::new("azure"), "Test 3"))
            .unwrap();

        assert_eq!(
            registry.operation_count_for_faction(&FactionId::new("crimson")),
            2
        );
        assert_eq!(
            registry.operation_count_for_faction(&FactionId::new("azure")),
            1
        );
    }
}
