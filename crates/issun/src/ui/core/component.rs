//! Component trait for composable UI elements
//!
//! Components are higher-level UI abstractions that encapsulate both data access
//! and rendering logic. Unlike widgets which are purely presentational,
//! components can access resources and manage their own state.
//!
//! # Example
//!
//! ```ignore
//! use issun::ui::core::Component;
//! use issun::context::ResourceContext;
//!
//! struct HeaderComponent;
//!
//! impl Component for HeaderComponent {
//!     type Context = GameContext;
//!     type Output = Paragraph<'static>;
//!
//!     fn render(&self, resources: &ResourceContext) -> Option<Self::Output> {
//!         let ctx = resources.try_get::<Self::Context>()?;
//!         Some(Paragraph::new(format!("Turn {}", ctx.turn)))
//!     }
//! }
//! ```

use crate::context::ResourceContext;

/// Component trait for composable UI elements
///
/// Components encapsulate both data access and rendering logic,
/// providing a higher-level abstraction for building complex UIs.
///
/// # Design Philosophy
///
/// - **Composable**: Components can be nested and combined
/// - **Resource-aware**: Components have access to ResourceContext
/// - **Error-resilient**: Components return Option to handle missing resources gracefully
/// - **Backend-independent**: The trait doesn't depend on any specific UI backend
///
/// # Type Parameters
///
/// - `Context`: The primary resource type this component depends on
/// - `Output`: The widget type this component renders (backend-specific)
pub trait Component {
    /// The primary context/resource type this component needs
    type Context: 'static + Send + Sync;

    /// The widget type this component renders (backend-specific)
    type Output;

    /// Render the component using resources
    ///
    /// Returns `None` if required resources are not available.
    /// This allows components to gracefully handle missing data.
    ///
    /// # Arguments
    ///
    /// * `resources` - The resource context to access game state
    ///
    /// # Returns
    ///
    /// * `Some(widget)` - Successfully rendered widget
    /// * `None` - Required resources not found, render fallback UI instead
    fn render(&self, resources: &ResourceContext) -> Option<Self::Output>;

    /// Check if this component can render with current resources
    ///
    /// Default implementation checks if the primary Context type exists.
    fn can_render(&self, resources: &ResourceContext) -> bool {
        resources.try_get::<Self::Context>().is_some()
    }
}

/// Multi-resource component that depends on multiple resource types
///
/// This trait is for components that need access to multiple resources.
/// It provides a more flexible rendering interface.
///
/// # Example
///
/// ```ignore
/// impl MultiResourceComponent for StatisticsComponent {
///     type Output = Paragraph<'static>;
///
///     fn render_multi(&self, resources: &ResourceContext) -> Option<Self::Output> {
///         let ctx = resources.try_get::<GameContext>()?;
///         let city = resources.try_get::<CityMap>()?;
///         Some(Paragraph::new(format!("Pop: {}", city.population)))
///     }
/// }
/// ```
pub trait MultiResourceComponent {
    /// The widget type this component renders
    type Output;

    /// Render the component using multiple resources
    ///
    /// This method provides access to the full ResourceContext,
    /// allowing the component to fetch multiple resource types.
    ///
    /// # Arguments
    ///
    /// * `resources` - The resource context to access game state
    ///
    /// # Returns
    ///
    /// * `Some(widget)` - Successfully rendered widget
    /// * `None` - Required resources not found
    fn render_multi(&self, resources: &ResourceContext) -> Option<Self::Output>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::ResourceContext;

    #[derive(Debug, Clone)]
    struct TestContext {
        value: u32,
    }

    struct TestComponent;

    impl Component for TestComponent {
        type Context = TestContext;
        type Output = String;

        fn render(&self, resources: &ResourceContext) -> Option<Self::Output> {
            let ctx = resources.try_get::<Self::Context>()?;
            Some(format!("Value: {}", ctx.value))
        }
    }

    #[test]
    fn test_component_render() {
        let mut resources = ResourceContext::new();
        resources.insert(TestContext { value: 42 });

        let component = TestComponent;
        let output = component.render(&resources);

        assert_eq!(output, Some("Value: 42".to_string()));
    }

    #[test]
    fn test_component_render_missing_resource() {
        let resources = ResourceContext::new();
        let component = TestComponent;
        let output = component.render(&resources);

        assert_eq!(output, None);
    }

    #[test]
    fn test_component_can_render() {
        let mut resources = ResourceContext::new();
        let component = TestComponent;

        assert!(!component.can_render(&resources));

        resources.insert(TestContext { value: 42 });
        assert!(component.can_render(&resources));
    }
}
