//! The pointers off the Mac: gpui's own. The big arrow and the box are
//! put behind `NSCursor`'s class methods on a Mac (`cursor.rs`); here
//! the system's arrow stands for both.

/// The style over a web or preview body.
pub const NATIVE_CURSOR: gpui::CursorStyle = gpui::CursorStyle::Arrow;

/// Nothing to set by hand: gpui sets the pointer from the styles.
pub fn apply(_style: gpui::CursorStyle) {}

/// Nothing to install.
pub fn install() {}
