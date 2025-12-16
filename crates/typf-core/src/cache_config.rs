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

// this_file: crates/typf-core/src/cache_config.rs

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::sync::OnceLock;

/// Global flag for whether caching is enabled (disabled by default)
static CACHING_ENABLED: AtomicBool = AtomicBool::new(false);

/// Serialize mutations of the global caching flag.
///
/// This is primarily for correctness in tests (which run in parallel by default), but also makes
/// `set_caching_enabled` deterministic for consumers that want scoped control.
static CACHING_CONFIG_LOCK: Mutex<()> = Mutex::new(());

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

fn set_caching_enabled_unlocked(enabled: bool) {
    check_env(); // Ensure env is checked first
    CACHING_ENABLED.store(enabled, Ordering::SeqCst);
    log::debug!(
        "Typf caching {} via runtime call",
        if enabled { "enabled" } else { "disabled" }
    );
}

/// Enable or disable caching globally at runtime
///
/// This overrides the environment variable setting.
///
/// # Arguments
///
/// * `enabled` - `true` to enable caching, `false` to disable
pub fn set_caching_enabled(enabled: bool) {
    let _lock = CACHING_CONFIG_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    set_caching_enabled_unlocked(enabled);
}

/// Scoped override for the global caching flag.
///
/// Holds an internal lock for the lifetime of the guard, preventing other threads from flipping
/// the flag via [`set_caching_enabled`] while the guard is alive. On drop, restores the previous
/// value.
pub fn scoped_caching_enabled(enabled: bool) -> ScopedCachingEnabled {
    let lock = CACHING_CONFIG_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = is_caching_enabled();
    set_caching_enabled_unlocked(enabled);
    ScopedCachingEnabled {
        _lock: lock,
        previous,
    }
}

pub struct ScopedCachingEnabled {
    _lock: std::sync::MutexGuard<'static, ()>,
    previous: bool,
}

impl Drop for ScopedCachingEnabled {
    fn drop(&mut self) {
        set_caching_enabled_unlocked(self.previous);
    }
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

    // Single test to avoid race conditions with other tests that use the global flag.
    // Tests that set_caching_enabled correctly updates the value returned by is_caching_enabled.

    #[test]
    fn test_set_caching_roundtrip() {
        let _lock = CACHING_CONFIG_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let previous = is_caching_enabled();

        set_caching_enabled_unlocked(true);
        let after_enable = is_caching_enabled();

        set_caching_enabled_unlocked(false);
        let after_disable = is_caching_enabled();

        set_caching_enabled_unlocked(previous);

        // Assert after all operations to reduce race window
        assert!(
            after_enable,
            "caching should be enabled after set_caching_enabled(true)"
        );
        assert!(
            !after_disable,
            "caching should be disabled after set_caching_enabled(false)"
        );
    }
}
