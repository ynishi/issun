//! Organization data converter trait and implementations

use crate::plugin::org_suite::types::{OrgArchetype, OrgSuiteError};

/// Trait for converting organization data between archetypes
///
/// Converters are stateless pure functions that transform data from one
/// organizational archetype to another. They use JSON as an intermediate
/// representation for plugin interoperability.
pub trait OrgConverter: Send + Sync {
    /// Get the source archetype
    fn source_archetype(&self) -> OrgArchetype;

    /// Get the target archetype
    fn target_archetype(&self) -> OrgArchetype;

    /// Convert organization data from source to target archetype
    ///
    /// # Arguments
    ///
    /// * `source_data` - JSON representation of the source organization data
    ///
    /// # Returns
    ///
    /// JSON representation of the target organization data, or error if conversion fails
    fn convert(&self, source_data: &serde_json::Value) -> Result<serde_json::Value, OrgSuiteError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Mock converter for testing
    struct MockConverter;

    impl OrgConverter for MockConverter {
        fn source_archetype(&self) -> OrgArchetype {
            OrgArchetype::Holacracy
        }

        fn target_archetype(&self) -> OrgArchetype {
            OrgArchetype::Hierarchy
        }

        fn convert(
            &self,
            source_data: &serde_json::Value,
        ) -> Result<serde_json::Value, OrgSuiteError> {
            // Simple mock conversion
            Ok(json!({
                "converted": true,
                "from": "holacracy",
                "to": "hierarchy",
                "source": source_data
            }))
        }
    }

    #[test]
    fn test_mock_converter() {
        let converter = MockConverter;
        assert_eq!(converter.source_archetype(), OrgArchetype::Holacracy);
        assert_eq!(converter.target_archetype(), OrgArchetype::Hierarchy);

        let source = json!({"members": 50});
        let result = converter.convert(&source).unwrap();

        assert_eq!(result["converted"], true);
        assert_eq!(result["from"], "holacracy");
    }
}
