//! UI rendering macros for simplified component composition
//!
//! This module provides macros to streamline UI rendering with automatic
//! error handling and layout management.

/// Drive macro for rendering components with automatic error handling
///
/// This macro simplifies the common pattern of:
/// 1. Creating a layout
/// 2. Rendering components to each layout chunk
/// 3. Handling missing resources gracefully
///
/// # Basic Usage
///
/// ```ignore
/// use issun::drive;
/// use issun::ui::ratatui::*;
/// use issun::ui::layer::UILayoutPresets;
///
/// fn render(frame: &mut Frame, resources: &ResourceContext, selected: usize) {
///     let header = HeaderComponent::<GameContext>::new();
///     let districts = DistrictsComponent::<CityMap>::new();
///     let log = LogComponent::<GameLog>::new();
///
///     drive! {
///         frame: frame,
///         layout: RatatuiLayer::three_panel().apply(frame.area()),
///         [
///             header.render(resources),
///             districts.render_with_selection(resources, selected),
///             log.render_multi(resources),
///         ]
///     }
/// }
/// ```
///
/// # Automatic Error Handling
///
/// The macro automatically handles `Option<Widget>` return values.
/// If a component returns `None` (resource not found), that chunk is skipped.
///
/// # Manual Layout
///
/// You can also provide your own layout chunks:
///
/// ```ignore
/// drive! {
///     frame: frame,
///     layout: custom_chunks,
///     [
///         component1.render(resources),
///         component2.render_multi(resources),
///     ]
/// }
/// ```
#[macro_export]
macro_rules! drive {
    // Main pattern: frame, layout, component array
    (
        frame: $frame:expr,
        layout: $layout:expr,
        [ $( $component:expr ),* $(,)? ]
    ) => {{
        let chunks = $layout;
        let mut chunk_idx = 0;

        $(
            if let Some(widget) = $component {
                if chunk_idx < chunks.len() {
                    $frame.render_widget(widget, chunks[chunk_idx]);
                }
            }
            chunk_idx += 1;
        )*
        let _ = chunk_idx;
    }};
}

/// Render components to specific areas with automatic error handling
///
/// This macro is useful when you have custom layout logic or want
/// to render components to specific, non-sequential areas.
///
/// # Usage
///
/// ```ignore
/// use issun::drive_to;
///
/// drive_to! {
///     frame: frame,
///     [
///         (header_area, header.render(resources)),
///         (main_area, content.render_multi(resources)),
///         (footer_area, log.render_multi(resources)),
///     ]
/// }
/// ```
///
/// # Fallback Widgets
///
/// You can provide fallback widgets for when components return `None`:
///
/// ```ignore
/// drive_to! {
///     frame: frame,
///     [
///         (area1, component.render(resources), Paragraph::new("Loading...")),
///         (area2, other.render_multi(resources)),
///     ]
/// }
/// ```
#[macro_export]
macro_rules! drive_to {
    // Pattern with fallback widgets
    (
        frame: $frame:expr,
        [
            $(
                ( $area:expr, $component:expr, $fallback:expr )
            ),* $(,)?
        ]
    ) => {{
        $(
            if let Some(widget) = $component {
                $frame.render_widget(widget, $area);
            } else {
                $frame.render_widget($fallback, $area);
            }
        )*
    }};

    // Pattern without fallback (skip if None)
    (
        frame: $frame:expr,
        [
            $(
                ( $area:expr, $component:expr )
            ),* $(,)?
        ]
    ) => {{
        $(
            if let Some(widget) = $component {
                $frame.render_widget(widget, $area);
            }
        )*
    }};
}

#[cfg(test)]
mod tests {
    /// Build-time test to ensure macros expand to valid code
    #[test]
    fn test_drive_macro_expands() {
        // This test verifies that the drive! macro expands correctly
        // by actually compiling the macro invocation
        use ratatui::layout::Rect;
        use ratatui::text::Text;
        use ratatui::widgets::{Block, Borders, Paragraph};

        // Mock frame - in real usage, this would be a ratatui Frame
        struct MockFrame;
        impl MockFrame {
            fn render_widget<W>(&mut self, _widget: W, _area: Rect) {}
        }
        let mut frame = MockFrame;

        let text = Text::from("test");
        let block = Block::default().borders(Borders::ALL);
        let widget1 = Some(Paragraph::new(text.clone()).block(block.clone()));
        let widget2 = Some(Paragraph::new(text).block(block));
        let chunks = vec![Rect::new(0, 0, 10, 10), Rect::new(0, 10, 10, 10)];

        // If this compiles, the macro works correctly
        drive! {
            frame: frame,
            layout: chunks,
            [widget1, widget2]
        };
    }

    /// Build-time test to ensure drive_to! macro expands correctly
    #[test]
    fn test_drive_to_macro_expands() {
        // This test verifies that the drive_to! macro expands correctly
        use ratatui::layout::Rect;
        use ratatui::text::Text;
        use ratatui::widgets::{Block, Borders, Paragraph};

        // Mock frame - in real usage, this would be a ratatui Frame
        struct MockFrame;
        impl MockFrame {
            fn render_widget<W>(&mut self, _widget: W, _area: Rect) {}
        }
        let mut frame = MockFrame;

        let text = Text::from("test");
        let block = Block::default().borders(Borders::ALL);
        let widget = Some(Paragraph::new(text.clone()).block(block.clone()));
        let fallback = Paragraph::new("fallback");
        let area1 = Rect::new(0, 0, 10, 10);
        let area2 = Rect::new(0, 10, 10, 10);

        // If this compiles, the macro works correctly
        drive_to! {
            frame: frame,
            [
                (area1, widget, fallback),
                (area2, None::<Paragraph>, Paragraph::new("empty")),
            ]
        };
    }
}
