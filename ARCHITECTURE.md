# TYPF Architecture

## Overview

TYPF (Type Font) is a high-performance, cross-platform font rendering engine written in Rust. It provides a unified API for rendering text across multiple backend systems while maintaining consistent quality and performance characteristics.

## Core Architecture Principles

1.  **Backend Agnostic**: Single API, multiple rendering backends
2.  **Memory Safe**: Rust's ownership system prevents memory errors
3.  **FFI Safe**: Clean C API and Python bindings via PyO3
4.  **Performance First**: Zero-copy operations where possible
5.  **Deterministic**: Reproducible rendering across platforms

## System Architecture

The architecture is designed to be modular, separating the high-level public API from the low-level backend implementations. This is achieved through a two-tiered trait system that resolves the cyclic dependency issues that were present in earlier designs.

```
┌──────────────────────────────────────────────────┐
│                   Public API (typf-api)          │
│  • Session management (Session, SessionBuilder)  │
│  • Backend enumeration (Backend)                 │
│  • Backend factory (create_backend)              │
└───────────────────┬──────────────────────────────┘
                    │
┌───────────────────▼──────────────────────────────┐
│           Dynamic Backend Trait (typf-core)      │
│                (Box<dyn DynBackend>)             │
└───────────────────┬──────────────────────────────┘
                    │
┌───────────────────▼──────────────────────────────┐
│             Core Backend Trait (typf-core)       │
│               (CoreBackendTrait)                 │
└──┬──────────┬──────────┬──────────┬────────────┘
   │          │          │          │
┌──▼──┐   ┌──▼──┐   ┌──▼──┐   ┌──▼──┐
│Orge │   │Core │   │Harf │   │ ... │
│     │   │Text │   │Buzz │   │     │
└─────┘   └─────┘   └─────┘   └─────┘
```

## Component Breakdown

### 1. Core Library (`typf-core`)

The heart of TYPF, providing the foundational traits and types. It is backend-agnostic and has no dependencies on other crates in the workspace.

-   **`CoreBackendTrait`**: A low-level trait for backend implementations, defining methods for segmentation, shaping, and rendering.
-   **`DynBackend`**: An object-safe trait for high-level interaction, used by the `typf-api` session. This trait simplifies the public API and is implemented by all backend crates.
-   Common types like `Font`, `Glyph`, `RenderOptions`, `Bitmap`, `Point`, `FontMetrics`, etc.

### 2. High-Level API (`typf-api`)

The primary entry point for users of the library. It provides a unified, easy-to-use interface.

-   **`Session`**: The main struct for interacting with the rendering engine. It holds a `Box<dyn DynBackend>`.
-   **`SessionBuilder`**: A builder for creating `Session` instances.
-   **`Backend` enum**: An enumeration of the available backends, used for backend selection. The variants are conditionally compiled based on enabled features.
-   **Factory functions**: `create_backend` and `create_default_backend` for creating `Box<dyn DynBackend>` instances. These functions have conditional logic to instantiate the correct backend based on the `Backend` enum variant and enabled features.

```rust
// In typf-api:
pub struct Session {
    backend: Box<dyn DynBackend>,
    // ...
}

impl Session {
    pub fn render(&self, text: &str, font: &Font) -> Result<Bitmap> {
        let shaped = self.backend.shape_text(text, font);
        self.backend.render_shaped_text(&shaped, &RenderOptions::default())
            .ok_or_else(|| TypfError::render("Rendering failed".to_string()))
    }
}
```

### 3. Backend System

Each backend crate (e.g., `typf-icu-hb`, `typf-mac`) depends on `typf-core` and implements the `DynBackend` trait. Some backends may also implement the `CoreBackendTrait` if they provide a full shaping and rendering pipeline.

-   **`DynBackend` implementation**: Acts as a bridge, adapting the high-level calls from `typf-api` to the backend's internal logic.

```rust
// In typf-core:
pub trait DynBackend {
    fn shape_text(&self, text: &str, font: &Font) -> ShapingResult;
    fn render_shaped_text(&self, shaped: &ShapingResult, options: &RenderOptions) -> Option<Bitmap>;
    // ...
}
```

#### Backend Naming Convention

TYPF backends are named to clearly distinguish between **shaping engines** and **rasterizers**:

**Format:** `<RASTERIZER><SHAPER>` (e.g., `orgehb`, `skiahb`)

**Components:**
- **Shaper**: Text layout engine (HarfBuzz, CoreText, DirectWrite)
- **Rasterizer**: Glyph rendering engine (Orge, TinySkia, Zeno)

**Examples:**
- `orgehb` = Orge rasterizer + HarfBuzz shaper + ICU
- `skiahb` = TinySkia rasterizer + HarfBuzz shaper + ICU
- `coretext` = CoreText (integrated shaping + rasterization)
- `directwrite` = DirectWrite (integrated shaping + rasterization)

This naming clarifies that backends like `orgehb` use HarfBuzz for text shaping but Orge for glyph rasterization, not HarfBuzz's built-in rasterizer.

#### Supported Backends

1.  **orgehb** (Cross-Platform Default)
    -   Shaping: HarfBuzz + ICU
    -   Rasterization: Orge (custom F26Dot6 scan converter), made by FontLab https://www.fontlab.com/
    -   Platform: All (Linux default)
    -   Best cross-platform consistency
    -   Good performance

2.  **CoreText** (macOS Default)
    -   Shaping + Rasterization: CoreText (integrated)
    -   Platform: macOS only
    -   Highest quality on macOS
    -   System font integration
    -   Platform-specific features

3.  **DirectWrite** (Windows Default)
    -   Shaping + Rasterization: DirectWrite (integrated)
    -   Platform: Windows only
    -   Native Windows rendering
    -   System font integration
    -   Platform-specific features

4.  **skiahb** (Planned)
    -   Shaping: HarfBuzz + ICU
    -   Rasterization: TinySkia
    -   Platform: All
    -   Vector rendering with anti-aliasing
    -   GPU acceleration potential

5.  **Orge** (In Progress)
    -   Rasterization only (no shaping yet)
    -   Pure Rust implementation
    -   Custom CPU rasterizer, made by FontLab https://www.fontlab.com/
    -   Experimental standalone backend
