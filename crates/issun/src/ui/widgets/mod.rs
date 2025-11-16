//! Widget system for ISSUN TUI

pub mod widget_trait;
pub mod menu;
pub mod dialog;

pub use widget_trait::Widget;
pub use menu::MenuWidget;
pub use dialog::DialogWidget;
