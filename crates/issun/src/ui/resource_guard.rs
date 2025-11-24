//! Resource Guard for safe resource access in UI rendering
//!
//! This module provides a type-safe wrapper for accessing resources from ResourceContext
//! with improved error handling and debugging capabilities.
//!
//! # Example
//!
//! ```ignore
//! use issun::ui::ResourceGuard;
//! use issun::context::ResourceContext;
//!
//! fn render(resources: &ResourceContext) {
//!     let ctx = ResourceGuard::new::<GameContext>(resources);
//!
//!     match ctx.get() {
//!         Ok(data) => {
//!             // Use data safely
//!         }
//!         Err(err) => {
//!             // Display error message
//!             eprintln!("Resource error: {}", err);
//!         }
//!     }
//! }
//! ```

use crate::context::{ResourceContext, ResourceReadGuard};
use std::fmt;

/// Error type for resource access failures
#[derive(Debug, Clone)]
pub struct ResourceError {
    pub resource_type: &'static str,
    pub message: String,
}

impl ResourceError {
    fn new<T: 'static>(message: impl Into<String>) -> Self {
        Self {
            resource_type: std::any::type_name::<T>(),
            message: message.into(),
        }
    }
}

impl fmt::Display for ResourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Resource '{}' error: {}",
            self.resource_type, self.message
        )
    }
}

impl std::error::Error for ResourceError {}

/// Resource Guard for safe resource access
///
/// This guard wraps a `ResourceReadGuard` and provides convenient methods
/// for accessing resources with error handling.
pub struct ResourceGuard<T: 'static> {
    inner: Option<ResourceReadGuard<T>>,
}

impl<T: 'static + Send + Sync> ResourceGuard<T> {
    /// Create a new ResourceGuard from ResourceContext
    ///
    /// # Example
    ///
    /// ```ignore
    /// let guard = ResourceGuard::<GameContext>::new(resources);
    /// ```
    pub fn new(resources: &ResourceContext) -> Self {
        Self {
            inner: resources.try_get::<T>(),
        }
    }

    /// Get a reference to the resource, or return an error
    ///
    /// # Errors
    ///
    /// Returns `ResourceError` if the resource is not found.
    ///
    /// # Example
    ///
    /// ```ignore
    /// match guard.get() {
    ///     Ok(data) => println!("Got data: {:?}", data),
    ///     Err(err) => eprintln!("Error: {}", err),
    /// }
    /// ```
    pub fn get(&self) -> Result<&T, ResourceError> {
        self.inner
            .as_deref()
            .ok_or_else(|| ResourceError::new::<T>("Resource not found"))
    }

    /// Get a reference to the resource, or return None
    ///
    /// This is a convenience method for cases where you want to handle
    /// the absence of a resource with `Option` instead of `Result`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Some(data) = guard.get_option() {
    ///     println!("Got data: {:?}", data);
    /// }
    /// ```
    pub fn get_option(&self) -> Option<&T> {
        self.inner.as_deref()
    }

    /// Get a reference to the resource, or return a default value
    ///
    /// # Example
    ///
    /// ```ignore
    /// let default_ctx = GameContext::default();
    /// let data = guard.get_or(&default_ctx);
    /// ```
    pub fn get_or<'a>(&'a self, default: &'a T) -> &'a T {
        self.inner.as_deref().unwrap_or(default)
    }

    /// Check if the resource exists
    ///
    /// # Example
    ///
    /// ```ignore
    /// if guard.is_available() {
    ///     println!("Resource is available");
    /// }
    /// ```
    pub fn is_available(&self) -> bool {
        self.inner.is_some()
    }

    /// Get the resource type name for debugging
    pub fn type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }
}

/// Helper macro for creating multiple ResourceGuards at once
///
/// # Example
///
/// ```ignore
/// use issun::resource_guards;
///
/// fn render(resources: &ResourceContext) {
///     let (ctx, city, state) = resource_guards!(
///         resources => GameContext, CityMap, ContagionState
///     );
///
///     if let (Ok(ctx), Ok(city), Ok(state)) = (ctx.get(), city.get(), state.get()) {
///         // All resources available
///     }
/// }
/// ```
#[macro_export]
macro_rules! resource_guards {
    ($resources:expr => $($ty:ty),+ $(,)?) => {
        (
            $(
                $crate::ui::ResourceGuard::<$ty>::new($resources),
            )+
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::ResourceContext;

    #[derive(Debug, Clone, PartialEq)]
    struct TestResource {
        value: u32,
    }

    #[test]
    fn test_resource_guard_get() {
        let mut resources = ResourceContext::new();
        resources.insert(TestResource { value: 42 });

        let guard = ResourceGuard::<TestResource>::new(&resources);
        assert!(guard.is_available());
        assert_eq!(guard.get().unwrap().value, 42);
    }

    #[test]
    fn test_resource_guard_not_found() {
        let resources = ResourceContext::new();
        let guard = ResourceGuard::<TestResource>::new(&resources);

        assert!(!guard.is_available());
        assert!(guard.get().is_err());
    }

    #[test]
    fn test_resource_guard_get_option() {
        let mut resources = ResourceContext::new();
        resources.insert(TestResource { value: 99 });

        let guard = ResourceGuard::<TestResource>::new(&resources);
        assert_eq!(guard.get_option().map(|r| r.value), Some(99));

        let empty_resources = ResourceContext::new();
        let empty_guard = ResourceGuard::<TestResource>::new(&empty_resources);
        assert!(empty_guard.get_option().is_none());
    }

    #[test]
    fn test_resource_guard_get_or() {
        let resources = ResourceContext::new();
        let guard = ResourceGuard::<TestResource>::new(&resources);

        let default = TestResource { value: 100 };
        assert_eq!(guard.get_or(&default).value, 100);
    }

    #[test]
    fn test_resource_error_display() {
        let err = ResourceError::new::<TestResource>("test error");
        let message = format!("{}", err);
        assert!(message.contains("TestResource"));
        assert!(message.contains("test error"));
    }
}
