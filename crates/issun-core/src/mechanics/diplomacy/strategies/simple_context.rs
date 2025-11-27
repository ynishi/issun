use crate::mechanics::diplomacy::policies::ContextPolicy;
use crate::mechanics::diplomacy::types::ArgumentType;

/// No contextual modifiers (default).
pub struct NoContext;

impl ContextPolicy for NoContext {
    fn apply_context(
        influence: f32,
        _arg_type: ArgumentType,
        _relationship: f32,
    ) -> f32 {
        influence
    }
}
