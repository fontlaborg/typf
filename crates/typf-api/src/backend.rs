// this_file: crates/typf-api/src/backend.rs


// Re-export these types publicly for use in the API
pub use typf_core::{DynBackend, BackendFeatures, Point, FontMetrics};

#[cfg(feature = "backend-harfbuzz")]
use typf_icu_hb::HarfBuzzBackend;
#[cfg(feature = "backend-coretext")]
use typf_mac::CoreTextBackend;
#[cfg(feature = "backend-orge")]
use typf_orge::OrgeBackend;
#[cfg(feature = "backend-directwrite")]
use typf_win::DirectWriteBackend;

/// An enum representing the available rendering backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    #[cfg(feature = "backend-harfbuzz")]
    HarfBuzz,
    #[cfg(feature = "backend-coretext")]
    CoreText,
    #[cfg(feature = "backend-orge")]
    Orge,
    #[cfg(feature = "backend-directwrite")]
    DirectWrite,
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "backend-harfbuzz")]
            Backend::HarfBuzz => write!(f, "HarfBuzz"),
            #[cfg(feature = "backend-coretext")]
            Backend::CoreText => write!(f, "CoreText"),
            #[cfg(feature = "backend-orge")]
            Backend::Orge => write!(f, "Orge"),
            #[cfg(feature = "backend-directwrite")]
            Backend::DirectWrite => write!(f, "DirectWrite"),
            #[allow(unreachable_patterns)]
            _ => write!(f, "Unknown"),
        }
    }
}

pub fn create_default_backend() -> Box<dyn DynBackend> {
    // Priority: CoreText (macOS) -> DirectWrite (Windows) -> HarfBuzz (cross-platform) -> Orge (pure Rust fallback)
    #[cfg(all(target_os = "macos", feature = "backend-coretext"))]
    if cfg!(all(target_os = "macos", feature = "backend-coretext")) {
        return Box::new(CoreTextBackend::new());
    }
    #[cfg(all(windows, feature = "backend-directwrite"))]
    if cfg!(all(windows, feature = "backend-directwrite")) {
        return Box::new(DirectWriteBackend::new());
    }
    #[cfg(feature = "backend-harfbuzz")]
    if cfg!(feature = "backend-harfbuzz") {
        return Box::new(HarfBuzzBackend::new());
    }
    #[cfg(feature = "backend-orge")]
    if cfg!(feature = "backend-orge") {
        return Box::new(OrgeBackend::new());
    }

    panic!("No rendering backend enabled. Please enable at least one backend feature (e.g., 'backend-harfbuzz', 'backend-orge').");
}

pub fn create_backend(backend: Backend) -> Box<dyn DynBackend> {
    match backend {
        #[cfg(feature = "backend-harfbuzz")]
        Backend::HarfBuzz => Box::new(HarfBuzzBackend::new()),
        #[cfg(feature = "backend-coretext")]
        Backend::CoreText => Box::new(CoreTextBackend::new()),
        #[cfg(feature = "backend-orge")]
        Backend::Orge => Box::new(OrgeBackend::new()),
        #[cfg(feature = "backend-directwrite")]
        Backend::DirectWrite => Box::new(DirectWriteBackend::new()),
        #[allow(unreachable_patterns)]
        _ => panic!("Requested backend is not enabled by feature flags."),
    }
}
