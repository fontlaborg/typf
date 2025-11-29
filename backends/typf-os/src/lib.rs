//! Platform-native linra text rendering
//!
//! This crate provides a platform-dispatching layer for linra text rendering.
//! It automatically selects the best native backend for the current platform:
//!
//! - **macOS**: CoreText via `typf-os-mac`
//! - **Windows**: DirectWrite via `typf-os-win` (planned)
//!
//! ## Usage
//!
//! ```rust,ignore
//! use typf_os::{OsRenderer, create_renderer, LinraRenderer, LinraRenderParams};
//! use std::sync::Arc;
//!
//! fn load_font() -> Arc<dyn typf_core::traits::FontRef> { unimplemented!() }
//!
//! let renderer = create_renderer();
//! let font = load_font();
//! let params = LinraRenderParams::with_size(24.0);
//!
//! let output = renderer.render_text("Hello!", font, &params);
//! ```
//!
//! ## Platform Support
//!
//! | Platform | Backend | Status |
//! |----------|---------|--------|
//! | macOS    | CoreText (CTLineDraw) | Implemented |
//! | Windows  | DirectWrite (DrawTextLayout) | Implemented |
//! | Linux    | - | Not supported |

// Re-export the linra renderer trait and params from typf-core
pub use typf_core::linra::{LinraRenderParams, LinraRenderer};

// Platform-specific implementations
#[cfg(target_os = "macos")]
pub use typf_os_mac::CoreTextLinraRenderer;

// Type alias for the current platform's renderer
#[cfg(target_os = "macos")]
pub type OsRenderer = typf_os_mac::CoreTextLinraRenderer;

// Windows support
#[cfg(target_os = "windows")]
pub use typf_os_win::DirectWriteLinraRenderer;

#[cfg(target_os = "windows")]
pub type OsRenderer = typf_os_win::DirectWriteLinraRenderer;

/// Creates a new linra renderer for the current platform
///
/// This is the simplest way to get started with platform-native text rendering.
///
/// # Panics
///
/// Panics at compile time if the current platform is not supported.
///
/// # Example
///
/// ```rust,ignore
/// use typf_os::{create_renderer, LinraRenderer};
///
/// let renderer = create_renderer();
/// println!("Using renderer: {}", renderer.name());
/// ```
#[cfg(target_os = "macos")]
pub fn create_renderer() -> OsRenderer {
    typf_os_mac::CoreTextLinraRenderer::new()
}

#[cfg(target_os = "windows")]
pub fn create_renderer() -> typf_core::error::Result<OsRenderer> {
    typf_os_win::DirectWriteLinraRenderer::new()
}

// Compile-time error for unsupported platforms
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
compile_error!(
    "typf-os requires macOS or Windows. For other platforms, use the separate \
     shaper and renderer backends (e.g., typf-shape-hb + typf-render-skia)."
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "macos")]
    fn test_create_renderer() {
        let renderer = create_renderer();
        assert_eq!(renderer.name(), "coretext-linra");
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_os_renderer_type() {
        // OsRenderer should be CoreTextLinraRenderer on macOS
        let _renderer: OsRenderer = create_renderer();
    }
}
