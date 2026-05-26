//! TUI 组件模块

pub mod chat;
pub mod code_editor;
pub mod code_viewer;
pub mod color_theme;
pub mod confirmation;
pub mod error_display;
pub mod input;
pub mod key_hints;
pub mod markdown_renderer;
pub mod permission_popup;
pub mod session_list;
pub mod status;
pub mod syntax_highlight;
pub mod token_stats;
pub mod tool_display;

pub use chat::ChatComponent;
pub use confirmation::{ConfirmationDialog, ConfirmAction, PermissionManager};
pub use input::InputComponent;
pub use key_hints::{HintContext, KeyHintsComponent};
pub use permission_popup::{PermissionAction, PermissionPopup};
pub use status::StatusComponent;
pub use tool_display::ToolDisplayComponent;
