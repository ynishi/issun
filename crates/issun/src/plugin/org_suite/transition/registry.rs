//! Transition registry for managing converters and conditions

use super::condition::TransitionCondition;
use super::converter::OrgConverter;
use crate::plugin::org_suite::types::{OrgArchetype, OrgSuiteError};
use std::collections::HashMap;

/// Registry for managing available transitions
///
/// The registry stores converters and conditions that define which organizational
/// transitions are possible and when they should occur.
pub struct TransitionRegistry {
    /// Converters keyed by (from, to) archetype pair
    converters: HashMap<(OrgArchetype, OrgArchetype), Box<dyn OrgConverter>>,

    /// Conditions for evaluating when transitions should occur
    conditions: Vec<Box<dyn TransitionCondition>>,
}

impl TransitionRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            converters: HashMap::new(),
            conditions: Vec::new(),
        }
    }

    /// Register a converter for a specific transition
    ///
    /// # Arguments
    ///
    /// * `converter` - Converter implementation for transforming organization data
    pub fn register_converter(&mut self, converter: Box<dyn OrgConverter>) {
        let key = (converter.source_archetype(), converter.target_archetype());
        self.converters.insert(key, converter);
    }

    /// Register a condition for transition evaluation
    ///
    /// # Arguments
    ///
    /// * `condition` - Condition implementation that determines when transitions occur
    pub fn register_condition(&mut self, condition: Box<dyn TransitionCondition>) {
        self.conditions.push(condition);
    }

    /// Get a converter for a specific transition
    ///
    /// # Arguments
    ///
    /// * `from` - Source archetype
    /// * `to` - Target archetype
    ///
    /// # Returns
    ///
    /// Reference to the converter, or error if not found
    pub fn get_converter(
        &self,
        from: OrgArchetype,
        to: OrgArchetype,
    ) -> Result<&dyn OrgConverter, OrgSuiteError> {
        self.converters
            .get(&(from, to))
            .map(|b| b.as_ref())
            .ok_or(OrgSuiteError::ConverterNotFound { from, to })
    }

    /// Check if a transition is valid (has a registered converter)
    ///
    /// # Arguments
    ///
    /// * `from` - Source archetype
    /// * `to` - Target archetype
    ///
    /// # Returns
    ///
    /// true if converter is registered, false otherwise
    pub fn is_transition_valid(&self, from: OrgArchetype, to: OrgArchetype) -> bool {
        self.converters.contains_key(&(from, to))
    }

    /// Get all registered conditions
    pub fn conditions(&self) -> &[Box<dyn TransitionCondition>] {
        &self.conditions
    }

    /// Get count of registered converters
    pub fn converter_count(&self) -> usize {
        self.converters.len()
    }

    /// Get count of registered conditions
    pub fn condition_count(&self) -> usize {
        self.conditions.len()
    }
}

impl Default for TransitionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::condition::ConditionContext;
    use super::*;
    use crate::plugin::org_suite::types::TransitionTrigger;

    // Mock converter
    struct MockConverter {
        from: OrgArchetype,
        to: OrgArchetype,
    }

    impl OrgConverter for MockConverter {
        fn source_archetype(&self) -> OrgArchetype {
            self.from
        }

        fn target_archetype(&self) -> OrgArchetype {
            self.to
        }

        fn convert(
            &self,
            _source_data: &serde_json::Value,
        ) -> Result<serde_json::Value, OrgSuiteError> {
            Ok(serde_json::json!({"converted": true}))
        }
    }

    // Mock condition
    struct MockCondition;

    impl TransitionCondition for MockCondition {
        fn evaluate(
            &self,
            _faction_id: &str,
            _current: OrgArchetype,
            _context: &ConditionContext,
        ) -> Option<TransitionTrigger> {
            None
        }
    }

    #[test]
    fn test_register_and_get_converter() {
        let mut registry = TransitionRegistry::new();

        let converter = Box::new(MockConverter {
            from: OrgArchetype::Holacracy,
            to: OrgArchetype::Hierarchy,
        });

        registry.register_converter(converter);

        assert!(registry.is_transition_valid(OrgArchetype::Holacracy, OrgArchetype::Hierarchy));

        let result = registry.get_converter(OrgArchetype::Holacracy, OrgArchetype::Hierarchy);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_converter_not_found() {
        let registry = TransitionRegistry::new();

        let result = registry.get_converter(OrgArchetype::Culture, OrgArchetype::Social);
        assert!(result.is_err());

        match result {
            Err(OrgSuiteError::ConverterNotFound { from, to }) => {
                assert_eq!(from, OrgArchetype::Culture);
                assert_eq!(to, OrgArchetype::Social);
            }
            _ => panic!("Expected ConverterNotFound error"),
        }
    }

    #[test]
    fn test_register_condition() {
        let mut registry = TransitionRegistry::new();

        registry.register_condition(Box::new(MockCondition));

        assert_eq!(registry.condition_count(), 1);
        assert_eq!(registry.conditions().len(), 1);
    }

    #[test]
    fn test_multiple_converters() {
        let mut registry = TransitionRegistry::new();

        registry.register_converter(Box::new(MockConverter {
            from: OrgArchetype::Holacracy,
            to: OrgArchetype::Hierarchy,
        }));

        registry.register_converter(Box::new(MockConverter {
            from: OrgArchetype::Hierarchy,
            to: OrgArchetype::Social,
        }));

        assert_eq!(registry.converter_count(), 2);
        assert!(registry.is_transition_valid(OrgArchetype::Holacracy, OrgArchetype::Hierarchy));
        assert!(registry.is_transition_valid(OrgArchetype::Hierarchy, OrgArchetype::Social));
        assert!(!registry.is_transition_valid(OrgArchetype::Social, OrgArchetype::Culture));
    }

    #[test]
    fn test_default() {
        let registry = TransitionRegistry::default();
        assert_eq!(registry.converter_count(), 0);
        assert_eq!(registry.condition_count(), 0);
    }
}
