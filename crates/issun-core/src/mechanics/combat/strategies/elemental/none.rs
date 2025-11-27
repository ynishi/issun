//! No elemental system strategy.
//!
//! This strategy ignores elemental types and returns damage unchanged.

use crate::mechanics::combat::policies::ElementalPolicy;
use crate::mechanics::combat::types::Element;

/// No elemental modifier: damage is returned unchanged.
///
/// This strategy is used when you don't want any elemental system in your game.
/// It simply passes through the damage value without modification.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::combat::policies::ElementalPolicy;
/// use issun_core::mechanics::combat::strategies::elemental::NoElemental;
/// use issun_core::mechanics::combat::Element;
///
/// // Elemental types are ignored
/// let damage = NoElemental::apply_elemental_modifier(
///     50,
///     Some(Element::Fire),
///     Some(Element::Ice),
/// );
/// assert_eq!(damage, 50); // No change
/// ```
pub struct NoElemental;

impl ElementalPolicy for NoElemental {
    fn apply_elemental_modifier(
        damage: i32,
        _attacker_element: Option<Element>,
        _defender_element: Option<Element>,
    ) -> i32 {
        damage // No modification
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_elemental_passthrough() {
        // Damage should be unchanged regardless of elements
        assert_eq!(
            NoElemental::apply_elemental_modifier(50, Some(Element::Fire), Some(Element::Ice)),
            50
        );

        assert_eq!(
            NoElemental::apply_elemental_modifier(
                100,
                Some(Element::Water),
                Some(Element::Lightning)
            ),
            100
        );

        assert_eq!(NoElemental::apply_elemental_modifier(25, None, None), 25);
    }

    #[test]
    fn test_no_elemental_with_none() {
        assert_eq!(
            NoElemental::apply_elemental_modifier(75, None, Some(Element::Fire)),
            75
        );

        assert_eq!(
            NoElemental::apply_elemental_modifier(75, Some(Element::Ice), None),
            75
        );
    }
}
