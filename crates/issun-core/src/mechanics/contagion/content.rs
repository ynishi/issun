//! Contagion content types and metadata.
//!
//! Supports multiple types of spreadable content:
//! - Disease: Biological infections
//! - ProductReputation: Brand/product sentiment
//! - Political: Political claims and ideology
//! - MarketTrend: Economic trends and commodity prices
//! - Custom: User-defined content types

/// Content that can spread through contagion mechanics.
///
/// This enum supports modeling various types of spreading phenomena,
/// not just biological diseases.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::contagion::{ContagionContent, DiseaseLevel};
///
/// // Biological disease
/// let plague = ContagionContent::Disease {
///     severity: DiseaseLevel::Severe,
///     location: "London".to_string(),
/// };
///
/// // Product reputation
/// let reputation = ContagionContent::ProductReputation {
///     product: "iPhone 15".to_string(),
///     sentiment: 0.8, // Positive sentiment
/// };
///
/// // Political ideology
/// let ideology = ContagionContent::Political {
///     faction: "Reformers".to_string(),
///     claim: "Lower taxes will boost economy".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum ContagionContent {
    /// Biological disease or infection.
    Disease {
        /// Severity level of the disease
        severity: DiseaseLevel,
        /// Origin location (for tracking)
        location: String,
    },

    /// Product or brand reputation.
    ProductReputation {
        /// Product or brand name
        product: String,
        /// Sentiment score (-1.0 to 1.0, where -1 is very negative, +1 is very positive)
        sentiment: f32,
    },

    /// Political ideology or claim.
    Political {
        /// Political faction or party
        faction: String,
        /// The claim or message being spread
        claim: String,
    },

    /// Market trend or commodity price movement.
    MarketTrend {
        /// Commodity or asset name
        commodity: String,
        /// Direction of the trend
        direction: TrendDirection,
    },

    /// Custom user-defined content.
    Custom {
        /// Type identifier
        key: String,
        /// Serialized data (JSON, etc.)
        data: String,
    },
}

impl ContagionContent {
    /// Get a human-readable description of this content.
    ///
    /// # Examples
    ///
    /// ```
    /// use issun_core::mechanics::contagion::{ContagionContent, DiseaseLevel};
    ///
    /// let disease = ContagionContent::Disease {
    ///     severity: DiseaseLevel::Moderate,
    ///     location: "Paris".to_string(),
    /// };
    /// assert!(disease.description().contains("Disease"));
    /// assert!(disease.description().contains("Moderate"));
    /// ```
    pub fn description(&self) -> String {
        match self {
            ContagionContent::Disease { severity, location } => {
                format!("Disease ({:?}) from {}", severity, location)
            }
            ContagionContent::ProductReputation { product, sentiment } => {
                format!("Reputation: {} (sentiment: {:.2})", product, sentiment)
            }
            ContagionContent::Political { faction, claim } => {
                format!("{}: {}", faction, claim)
            }
            ContagionContent::MarketTrend {
                commodity,
                direction,
            } => {
                format!("{} trend: {:?}", commodity, direction)
            }
            ContagionContent::Custom { key, .. } => {
                format!("Custom: {}", key)
            }
        }
    }
}

/// Disease severity levels.
///
/// Represents the severity of a disease, with methods for progression.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::contagion::DiseaseLevel;
///
/// let mut level = DiseaseLevel::Mild;
/// level = level.increase();
/// assert_eq!(level, DiseaseLevel::Moderate);
///
/// level = level.decrease();
/// assert_eq!(level, DiseaseLevel::Mild);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiseaseLevel {
    /// Mild symptoms, low mortality
    Mild,
    /// Moderate symptoms, some risk
    Moderate,
    /// Severe symptoms, high risk
    Severe,
    /// Critical condition, very high mortality
    Critical,
}

impl DiseaseLevel {
    /// Increase severity by one level.
    ///
    /// Critical remains Critical (cannot increase further).
    ///
    /// # Examples
    ///
    /// ```
    /// use issun_core::mechanics::contagion::DiseaseLevel;
    ///
    /// assert_eq!(DiseaseLevel::Mild.increase(), DiseaseLevel::Moderate);
    /// assert_eq!(DiseaseLevel::Moderate.increase(), DiseaseLevel::Severe);
    /// assert_eq!(DiseaseLevel::Severe.increase(), DiseaseLevel::Critical);
    /// assert_eq!(DiseaseLevel::Critical.increase(), DiseaseLevel::Critical);
    /// ```
    pub fn increase(self) -> Self {
        match self {
            DiseaseLevel::Mild => DiseaseLevel::Moderate,
            DiseaseLevel::Moderate => DiseaseLevel::Severe,
            DiseaseLevel::Severe => DiseaseLevel::Critical,
            DiseaseLevel::Critical => DiseaseLevel::Critical,
        }
    }

    /// Decrease severity by one level.
    ///
    /// Mild remains Mild (cannot decrease further).
    ///
    /// # Examples
    ///
    /// ```
    /// use issun_core::mechanics::contagion::DiseaseLevel;
    ///
    /// assert_eq!(DiseaseLevel::Critical.decrease(), DiseaseLevel::Severe);
    /// assert_eq!(DiseaseLevel::Severe.decrease(), DiseaseLevel::Moderate);
    /// assert_eq!(DiseaseLevel::Moderate.decrease(), DiseaseLevel::Mild);
    /// assert_eq!(DiseaseLevel::Mild.decrease(), DiseaseLevel::Mild);
    /// ```
    pub fn decrease(self) -> Self {
        match self {
            DiseaseLevel::Critical => DiseaseLevel::Severe,
            DiseaseLevel::Severe => DiseaseLevel::Moderate,
            DiseaseLevel::Moderate => DiseaseLevel::Mild,
            DiseaseLevel::Mild => DiseaseLevel::Mild,
        }
    }

    /// Get severity as a numeric value (0-3).
    ///
    /// # Examples
    ///
    /// ```
    /// use issun_core::mechanics::contagion::DiseaseLevel;
    ///
    /// assert_eq!(DiseaseLevel::Mild.as_u32(), 0);
    /// assert_eq!(DiseaseLevel::Moderate.as_u32(), 1);
    /// assert_eq!(DiseaseLevel::Severe.as_u32(), 2);
    /// assert_eq!(DiseaseLevel::Critical.as_u32(), 3);
    /// ```
    pub fn as_u32(self) -> u32 {
        match self {
            DiseaseLevel::Mild => 0,
            DiseaseLevel::Moderate => 1,
            DiseaseLevel::Severe => 2,
            DiseaseLevel::Critical => 3,
        }
    }
}

/// Market trend direction.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::contagion::TrendDirection;
///
/// let bullish = TrendDirection::Bullish;
/// assert_eq!(bullish.multiplier(), 1.0);
///
/// let bearish = TrendDirection::Bearish;
/// assert_eq!(bearish.multiplier(), -1.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrendDirection {
    /// Upward trend (prices rising)
    Bullish,
    /// Downward trend (prices falling)
    Bearish,
    /// No clear trend
    Neutral,
}

impl TrendDirection {
    /// Get trend as a multiplier (-1.0, 0.0, or 1.0).
    ///
    /// # Examples
    ///
    /// ```
    /// use issun_core::mechanics::contagion::TrendDirection;
    ///
    /// assert_eq!(TrendDirection::Bullish.multiplier(), 1.0);
    /// assert_eq!(TrendDirection::Bearish.multiplier(), -1.0);
    /// assert_eq!(TrendDirection::Neutral.multiplier(), 0.0);
    /// ```
    pub fn multiplier(self) -> f32 {
        match self {
            TrendDirection::Bullish => 1.0,
            TrendDirection::Bearish => -1.0,
            TrendDirection::Neutral => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disease_level_increase() {
        assert_eq!(DiseaseLevel::Mild.increase(), DiseaseLevel::Moderate);
        assert_eq!(DiseaseLevel::Moderate.increase(), DiseaseLevel::Severe);
        assert_eq!(DiseaseLevel::Severe.increase(), DiseaseLevel::Critical);
        assert_eq!(DiseaseLevel::Critical.increase(), DiseaseLevel::Critical);
    }

    #[test]
    fn test_disease_level_decrease() {
        assert_eq!(DiseaseLevel::Critical.decrease(), DiseaseLevel::Severe);
        assert_eq!(DiseaseLevel::Severe.decrease(), DiseaseLevel::Moderate);
        assert_eq!(DiseaseLevel::Moderate.decrease(), DiseaseLevel::Mild);
        assert_eq!(DiseaseLevel::Mild.decrease(), DiseaseLevel::Mild);
    }

    #[test]
    fn test_disease_level_as_u32() {
        assert_eq!(DiseaseLevel::Mild.as_u32(), 0);
        assert_eq!(DiseaseLevel::Moderate.as_u32(), 1);
        assert_eq!(DiseaseLevel::Severe.as_u32(), 2);
        assert_eq!(DiseaseLevel::Critical.as_u32(), 3);
    }

    #[test]
    fn test_trend_direction_multiplier() {
        assert_eq!(TrendDirection::Bullish.multiplier(), 1.0);
        assert_eq!(TrendDirection::Bearish.multiplier(), -1.0);
        assert_eq!(TrendDirection::Neutral.multiplier(), 0.0);
    }

    #[test]
    fn test_content_description() {
        let disease = ContagionContent::Disease {
            severity: DiseaseLevel::Moderate,
            location: "Paris".to_string(),
        };
        let desc = disease.description();
        assert!(desc.contains("Disease"));
        assert!(desc.contains("Paris"));

        let reputation = ContagionContent::ProductReputation {
            product: "Widget".to_string(),
            sentiment: 0.75,
        };
        let desc = reputation.description();
        assert!(desc.contains("Widget"));
        assert!(desc.contains("0.75"));
    }
}
