//! Districts list component for displaying location-based game data
//!
//! Renders a scrollable list of districts/locations with detailed stats.

use crate::context::ResourceContext;
use crate::ui::core::MultiResourceComponent;
use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
};

/// Trait for district/location data that can be displayed in the districts list
pub trait DistrictData: Send + Sync {
    /// Unique identifier for this district
    fn id(&self) -> &str;

    /// Display name of the district
    fn name(&self) -> &str;

    /// Format the district as a list item string
    ///
    /// This method should return the full text to display,
    /// including emojis, stats, progress bars, etc.
    fn format_line(&self) -> String;
}

/// Trait for collections that provide district data
pub trait DistrictsProvider: Send + Sync + 'static {
    /// The district data type
    type District: DistrictData;

    /// Get all districts as a slice
    fn districts(&self) -> &[Self::District];
}

/// Districts list component
///
/// Renders a scrollable list of districts with selection highlighting.
///
/// # Type Parameters
///
/// * `T` - The districts provider (e.g., CityMap)
///
/// # Example
///
/// ```ignore
/// use issun::ui::ratatui::DistrictsComponent;
/// use issun::ui::core::MultiResourceComponent;
///
/// let districts = DistrictsComponent::<CityMap>::new();
/// if let Some(widget) = districts.render_with_selection(&resources, selected_idx) {
///     frame.render_widget(widget, area);
/// }
/// ```
pub struct DistrictsComponent<T: DistrictsProvider> {
    title: String,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: DistrictsProvider> DistrictsComponent<T> {
    /// Create a new districts component with default title
    pub fn new() -> Self {
        Self::with_title("Districts")
    }

    /// Create a new districts component with custom title
    pub fn with_title(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Render the districts list with selection highlighting
    ///
    /// # Arguments
    ///
    /// * `resources` - Resource context
    /// * `selected_index` - Index of the selected district (for highlighting)
    ///
    /// # Returns
    ///
    /// * `Some(List)` - Successfully rendered districts list
    /// * `None` - Districts data not found
    pub fn render_with_selection(
        &self,
        resources: &ResourceContext,
        selected_index: usize,
    ) -> Option<List<'static>> {
        let provider = resources.try_get::<T>()?;

        let districts = provider.districts();
        let items: Vec<ListItem> = districts
            .iter()
            .enumerate()
            .map(|(i, district)| {
                let style = if i == selected_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                ListItem::new(district.format_line()).style(style)
            })
            .collect();

        Some(
            List::new(items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.title.clone()),
            ),
        )
    }
}

impl<T: DistrictsProvider> Default for DistrictsComponent<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: DistrictsProvider> MultiResourceComponent for DistrictsComponent<T> {
    type Output = List<'static>;

    fn render_multi(&self, resources: &ResourceContext) -> Option<Self::Output> {
        self.render_with_selection(resources, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::ResourceContext;

    #[derive(Debug, Clone)]
    struct TestDistrict {
        id: String,
        name: String,
        value: u32,
    }

    impl DistrictData for TestDistrict {
        fn id(&self) -> &str {
            &self.id
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn format_line(&self) -> String {
            format!("{}: {} (value: {})", self.id, self.name, self.value)
        }
    }

    #[derive(Debug, Clone)]
    struct TestCity {
        districts: Vec<TestDistrict>,
    }

    impl DistrictsProvider for TestCity {
        type District = TestDistrict;

        fn districts(&self) -> &[Self::District] {
            &self.districts
        }
    }

    #[test]
    fn test_districts_component() {
        let mut resources = ResourceContext::new();
        resources.insert(TestCity {
            districts: vec![
                TestDistrict {
                    id: "d1".to_string(),
                    name: "District 1".to_string(),
                    value: 10,
                },
                TestDistrict {
                    id: "d2".to_string(),
                    name: "District 2".to_string(),
                    value: 20,
                },
            ],
        });

        let component = DistrictsComponent::<TestCity>::new();
        let widget = component.render_with_selection(&resources, 0);

        assert!(widget.is_some());
    }

    #[test]
    fn test_districts_component_missing_resource() {
        let resources = ResourceContext::new();
        let component = DistrictsComponent::<TestCity>::new();
        let widget = component.render_with_selection(&resources, 0);

        assert!(widget.is_none());
    }
}
