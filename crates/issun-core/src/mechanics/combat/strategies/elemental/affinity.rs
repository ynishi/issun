//! Elemental affinity/weakness strategy.
//!
//! This strategy implements a Pokémon-style elemental type matchup system.

use crate::mechanics::combat::policies::ElementalPolicy;
use crate::mechanics::combat::types::Element;

/// Elemental affinity system with super effective / not very effective modifiers.
///
/// This strategy implements a type matchup system similar to Pokémon:
/// - Super effective: 2.0x damage (Fire vs Ice, Water vs Fire, etc.)
/// - Not very effective: 0.5x damage (Fire vs Water, Water vs Lightning, etc.)
/// - Normal: 1.0x damage (all other matchups)
///
/// # Type Chart
///
/// ```text
/// Attacker → Defender | Multiplier
/// -------------------|------------
/// Fire → Ice         | 2.0x (Super effective)
/// Fire → Water       | 0.5x (Not very effective)
/// Water → Fire       | 2.0x (Super effective)
/// Water → Lightning  | 0.5x (Not very effective)
/// Lightning → Water  | 2.0x (Super effective)
/// Lightning → Earth  | 0.5x (Not very effective)
/// Ice → Wind         | 2.0x (Super effective)
/// Ice → Fire         | 0.5x (Not very effective)
/// Wind → Earth       | 2.0x (Super effective)
/// Wind → Ice         | 0.5x (Not very effective)
/// Earth → Lightning  | 2.0x (Super effective)
/// Earth → Wind       | 0.5x (Not very effective)
/// Physical → *       | 1.0x (Neutral)
/// ```
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::combat::policies::ElementalPolicy;
/// use issun_core::mechanics::combat::strategies::elemental::ElementalAffinity;
/// use issun_core::mechanics::combat::Element;
///
/// // Super effective: Fire vs Ice
/// let damage = ElementalAffinity::apply_elemental_modifier(
///     50,
///     Some(Element::Fire),
///     Some(Element::Ice),
/// );
/// assert_eq!(damage, 100); // 2.0x multiplier
///
/// // Not very effective: Fire vs Water
/// let damage = ElementalAffinity::apply_elemental_modifier(
///     50,
///     Some(Element::Fire),
///     Some(Element::Water),
/// );
/// assert_eq!(damage, 25); // 0.5x multiplier
///
/// // No element = neutral
/// let damage = ElementalAffinity::apply_elemental_modifier(
///     50,
///     None,
///     Some(Element::Fire),
/// );
/// assert_eq!(damage, 50); // 1.0x multiplier
/// ```
pub struct ElementalAffinity;

impl ElementalPolicy for ElementalAffinity {
    fn apply_elemental_modifier(
        damage: i32,
        attacker_element: Option<Element>,
        defender_element: Option<Element>,
    ) -> i32 {
        // If either element is None, no modifier
        let (Some(attacker), Some(defender)) = (attacker_element, defender_element) else {
            return damage;
        };

        let multiplier = match (attacker, defender) {
            // Fire matchups
            (Element::Fire, Element::Ice) => 2.0,
            (Element::Fire, Element::Water) => 0.5,

            // Water matchups
            (Element::Water, Element::Fire) => 2.0,
            (Element::Water, Element::Lightning) => 0.5,

            // Lightning matchups
            (Element::Lightning, Element::Water) => 2.0,
            (Element::Lightning, Element::Earth) => 0.5,

            // Ice matchups
            (Element::Ice, Element::Wind) => 2.0,
            (Element::Ice, Element::Fire) => 0.5,

            // Wind matchups
            (Element::Wind, Element::Earth) => 2.0,
            (Element::Wind, Element::Ice) => 0.5,

            // Earth matchups
            (Element::Earth, Element::Lightning) => 2.0,
            (Element::Earth, Element::Wind) => 0.5,

            // Physical is always neutral
            (Element::Physical, _) | (_, Element::Physical) => 1.0,

            // All other combinations are neutral
            _ => 1.0,
        };

        ((damage as f32) * multiplier) as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_super_effective() {
        // Fire vs Ice
        assert_eq!(
            ElementalAffinity::apply_elemental_modifier(
                50,
                Some(Element::Fire),
                Some(Element::Ice)
            ),
            100
        );

        // Water vs Fire
        assert_eq!(
            ElementalAffinity::apply_elemental_modifier(
                50,
                Some(Element::Water),
                Some(Element::Fire)
            ),
            100
        );

        // Lightning vs Water
        assert_eq!(
            ElementalAffinity::apply_elemental_modifier(
                50,
                Some(Element::Lightning),
                Some(Element::Water)
            ),
            100
        );
    }

    #[test]
    fn test_not_very_effective() {
        // Fire vs Water
        assert_eq!(
            ElementalAffinity::apply_elemental_modifier(
                50,
                Some(Element::Fire),
                Some(Element::Water)
            ),
            25
        );

        // Water vs Lightning
        assert_eq!(
            ElementalAffinity::apply_elemental_modifier(
                50,
                Some(Element::Water),
                Some(Element::Lightning)
            ),
            25
        );

        // Lightning vs Earth
        assert_eq!(
            ElementalAffinity::apply_elemental_modifier(
                50,
                Some(Element::Lightning),
                Some(Element::Earth)
            ),
            25
        );
    }

    #[test]
    fn test_neutral_matchups() {
        // Fire vs Fire (same element)
        assert_eq!(
            ElementalAffinity::apply_elemental_modifier(
                50,
                Some(Element::Fire),
                Some(Element::Fire)
            ),
            50
        );

        // Fire vs Lightning (no special interaction)
        assert_eq!(
            ElementalAffinity::apply_elemental_modifier(
                50,
                Some(Element::Fire),
                Some(Element::Lightning)
            ),
            50
        );
    }

    #[test]
    fn test_no_element() {
        // No attacker element
        assert_eq!(
            ElementalAffinity::apply_elemental_modifier(50, None, Some(Element::Fire)),
            50
        );

        // No defender element
        assert_eq!(
            ElementalAffinity::apply_elemental_modifier(50, Some(Element::Fire), None),
            50
        );

        // Both None
        assert_eq!(
            ElementalAffinity::apply_elemental_modifier(50, None, None),
            50
        );
    }

    #[test]
    fn test_physical_element() {
        // Physical always neutral
        assert_eq!(
            ElementalAffinity::apply_elemental_modifier(
                50,
                Some(Element::Physical),
                Some(Element::Fire)
            ),
            50
        );

        assert_eq!(
            ElementalAffinity::apply_elemental_modifier(
                50,
                Some(Element::Fire),
                Some(Element::Physical)
            ),
            50
        );
    }
}
