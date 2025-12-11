//! Global cache configuration
//!
//! Provides a simple way to enable/disable all caching globally.
//! **Caching is disabled by default** and must be explicitly enabled
//! via `set_caching_enabled(true)` or the `TYPF_CACHE=1` environment variable.
//!
//! # Example
//!
//! ```
//! use typf_core::cache_config;
//!
//! // Enable caching (disabled by default)
//! cache_config::set_caching_enabled(true);
//!
//! // Check if caching is enabled
//! if cache_config::is_caching_enabled() {
//!     println!("Caching is ON");
//! }
//!
//! // Disable caching again
//! cache_config::set_caching_enabled(false);
//! ```
//!
//! # Environment Variable
//!
//! Set `TYPF_CACHE=1` to enable caching at startup:
//!
//! ```bash
//! TYPF_CACHE=1 ./my_app
//! ```

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

/// Global flag for whether caching is enabled (disabled by default)
static CACHING_ENABLED: AtomicBool = AtomicBool::new(false);

/// Whether the env var has been checked
static ENV_CHECKED: OnceLock<()> = OnceLock::new();

/// Check environment variable and initialize the global cache state
fn check_env() {
    ENV_CHECKED.get_or_init(|| {
        if let Ok(val) = std::env::var("TYPF_CACHE") {
            let enabled = matches!(val.to_lowercase().as_str(), "1" | "true" | "yes" | "on");
            if enabled {
                CACHING_ENABLED.store(true, Ordering::SeqCst);
                log::info!("Typf caching enabled via TYPF_CACHE env var");
            }
        }
    });
}

/// Check if caching is globally enabled
///
/// This checks the `TYPF_CACHE` environment variable on first call,
/// then returns the cached result. Runtime changes via `set_caching_enabled`
/// are also respected.
///
/// **Note:** Caching is disabled by default. Enable it via
/// `set_caching_enabled(true)` or `TYPF_CACHE=1` environment variable.
///
/// # Returns
///
/// `true` if caching is enabled, `false` if disabled (default).
pub fn is_caching_enabled() -> bool {
    check_env();
    CACHING_ENABLED.load(Ordering::SeqCst)
}

/// Enable or disable caching globally at runtime
///
/// This overrides the environment variable setting.
///
/// # Arguments
///
/// * `enabled` - `true` to enable caching, `false` to disable
pub fn set_caching_enabled(enabled: bool) {
    check_env(); // Ensure env is checked first
    CACHING_ENABLED.store(enabled, Ordering::SeqCst);
    log::debug!(
        "Typf caching {} via runtime call",
        if enabled { "enabled" } else { "disabled" }
    );
}

/// Clear all caches (convenience function)
///
/// This doesn't disable caching, just clears existing cached data.
/// For shapers/renderers with their own caches, call their `clear_cache()` methods.
pub fn clear_all_caches() {
    // Note: This is a placeholder. Individual caches need to be cleared via their
    // own methods since they're not centrally tracked. This function exists for
    // API consistency and documentation purposes.
    log::debug!("clear_all_caches called - individual backend caches should be cleared separately");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_disabled() {
        // Reset for test (note: this is not fully isolated due to global state)
        // In practice, run tests with --test-threads=1 if needed
        set_caching_enabled(false);
        assert!(!is_caching_enabled());
    }

    #[test]
    fn test_enable_disable() {
        set_caching_enabled(true);
        assert!(is_caching_enabled());

        set_caching_enabled(false);
        assert!(!is_caching_enabled());
    }
}
