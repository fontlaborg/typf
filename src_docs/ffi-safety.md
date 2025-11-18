# FFI Safety Patterns in TYPF

## Overview

TYPF provides Foreign Function Interface (FFI) bindings for C, Python (via PyO3), and potentially other languages. This document describes the safety patterns and best practices used to ensure memory safety, prevent undefined behavior, and maintain API stability across language boundaries.

## Core Safety Principles

1. **No Panics Across FFI Boundaries** - All panics must be caught and converted to error codes
2. **Null Pointer Validation** - All pointer parameters must be validated before dereferencing
3. **Lifetime Management** - Clear ownership rules for allocated memory
4. **Thread Safety** - Explicit documentation of thread safety guarantees
5. **Error Propagation** - Consistent error handling across language boundaries

## C FFI Safety Patterns

### 1. Null Pointer Checks

Every C-exposed function validates pointers before use:

```rust
#[no_mangle]
pub unsafe extern "C" fn typf_render(
    text_ptr: *const c_char,
    font_path_ptr: *const c_char,
    size: f32,
    out_width: *mut u32,
    out_height: *mut u32,
) -> *mut u8 {
    // SAFETY: Validate all pointers before use
    if text_ptr.is_null() || font_path_ptr.is_null() {
        return std::ptr::null_mut();
    }

    if out_width.is_null() || out_height.is_null() {
        return std::ptr::null_mut();
    }

    // SAFETY: CStr::from_ptr requires valid null-terminated string
    let text = match CStr::from_ptr(text_ptr).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    // Continue with safe Rust code...
}
```

### 2. Panic Catching

All FFI functions use `catch_unwind` to prevent panics from crossing boundaries:

```rust
#[no_mangle]
pub unsafe extern "C" fn typf_create_renderer(
    font_path: *const c_char,
    error_code: *mut i32,
) -> *mut TypfRenderer {
    use std::panic::catch_unwind;

    // Set default error code
    if !error_code.is_null() {
        *error_code = 0;
    }

    // Catch any panics and convert to error codes
    let result = catch_unwind(|| {
        // Validation and processing...
    });

    match result {
        Ok(renderer) => renderer,
        Err(_) => {
            if !error_code.is_null() {
                *error_code = ERROR_PANIC;
            }
            std::ptr::null_mut()
        }
    }
}
```

### 3. Memory Ownership Rules

Clear documentation of who owns allocated memory:

```rust
/// Allocates memory that must be freed by caller using typf_free()
///
/// # Safety
/// - Returned pointer is owned by caller
/// - Must call typf_free() when done
/// - Do not use after free
#[no_mangle]
pub unsafe extern "C" fn typf_render_alloc(
    /* parameters */
) -> *mut u8 {
    let buffer = render_internal();
    Box::into_raw(buffer.into_boxed_slice()) as *mut u8
}

/// Frees memory allocated by TYPF functions
///
/// # Safety
/// - ptr must have been allocated by a TYPF function
/// - ptr must not be used after this call
/// - Double-free is undefined behavior
#[no_mangle]
pub unsafe extern "C" fn typf_free(ptr: *mut u8, len: usize) {
    if !ptr.is_null() && len > 0 {
        let _ = Vec::from_raw_parts(ptr, len, len);
        // Vec is dropped here, freeing memory
    }
}
```

### 4. Opaque Types

Use opaque pointers for complex Rust types:

```rust
// Internal Rust type (not exposed)
pub struct RendererInner {
    font: Font,
    cache: GlyphCache,
    config: RenderConfig,
}

// Opaque handle for C
pub struct TypfRenderer {
    _private: [u8; 0],
}

#[no_mangle]
pub unsafe extern "C" fn typf_renderer_create() -> *mut TypfRenderer {
    let inner = Box::new(RendererInner::new());
    Box::into_raw(inner) as *mut TypfRenderer
}

#[no_mangle]
pub unsafe extern "C" fn typf_renderer_destroy(renderer: *mut TypfRenderer) {
    if !renderer.is_null() {
        let _ = Box::from_raw(renderer as *mut RendererInner);
        // Box is dropped here
    }
}
```

## Python FFI (PyO3) Safety

### 1. GIL Management

Properly acquire and release the Global Interpreter Lock:

```rust
use pyo3::prelude::*;

#[pyfunction]
fn render_text_parallel(texts: Vec<String>) -> PyResult<Vec<Vec<u8>>> {
    // Release GIL for CPU-bound work
    Python::with_gil(|py| {
        py.allow_threads(|| {
            // Parallel rendering without GIL
            texts.par_iter()
                .map(|text| render_single(text))
                .collect()
        })
    })
}
```

### 2. Exception Conversion

Convert Rust errors to Python exceptions:

```rust
use pyo3::exceptions::PyValueError;

#[pyfunction]
fn load_font(path: &str) -> PyResult<Font> {
    Font::from_file(path)
        .map_err(|e| PyValueError::new_err(format!("Failed to load font: {}", e)))
}
```

### 3. Memory Views

Efficient zero-copy data transfer:

```rust
#[pyfunction]
fn render_to_buffer<'py>(
    py: Python<'py>,
    text: &str,
) -> PyResult<&'py PyBytes> {
    let buffer = render_internal(text)?;

    // Create Python bytes without copying
    Ok(PyBytes::new(py, &buffer))
}
```

### 4. Type Safety

Use Python type hints and runtime validation:

```rust
#[pyclass]
struct PyRenderer {
    inner: Arc<Mutex<RendererInner>>,
}

#[pymethods]
impl PyRenderer {
    #[new]
    fn new(font_path: &str) -> PyResult<Self> {
        // Validate inputs
        if font_path.is_empty() {
            return Err(PyValueError::new_err("Font path cannot be empty"));
        }

        Ok(Self {
            inner: Arc::new(Mutex::new(RendererInner::new(font_path)?)),
        })
    }

    fn render(&self, text: &str, size: f32) -> PyResult<Vec<u8>> {
        // Validate parameters
        if size <= 0.0 || size > 10000.0 {
            return Err(PyValueError::new_err("Invalid font size"));
        }

        let inner = self.inner.lock()
            .map_err(|_| PyValueError::new_err("Renderer lock poisoned"))?;

        inner.render(text, size)
            .map_err(|e| PyValueError::new_err(format!("Render failed: {}", e)))
    }
}
```

## Thread Safety Patterns

### 1. Send + Sync Markers

Explicitly document thread safety:

```rust
// SAFETY: Renderer contains only Send+Sync types
unsafe impl Send for TypfRenderer {}
unsafe impl Sync for TypfRenderer {}

// Document in API:
/// Thread-safe renderer that can be shared between threads
/// Uses interior mutability with Arc<Mutex<_>> for safe concurrent access
```

### 2. Thread-Local Storage

Use thread-local caches safely:

```rust
thread_local! {
    static GLYPH_CACHE: RefCell<HashMap<GlyphKey, Vec<u8>>> = RefCell::new(HashMap::new());
}

pub fn render_with_cache(text: &str) -> Result<Vec<u8>> {
    GLYPH_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        // Use cache safely within thread
    })
}
```

## Error Handling Patterns

### 1. Error Codes for C

Consistent error code system:

```c
// typf.h
typedef enum {
    TYPF_SUCCESS = 0,
    TYPF_ERROR_NULL_POINTER = -1,
    TYPF_ERROR_INVALID_UTF8 = -2,
    TYPF_ERROR_FILE_NOT_FOUND = -3,
    TYPF_ERROR_INVALID_FONT = -4,
    TYPF_ERROR_RENDER_FAILED = -5,
    TYPF_ERROR_OUT_OF_MEMORY = -6,
    TYPF_ERROR_PANIC = -99,
} typf_error_t;
```

### 2. Result Types for Rust

Internal error handling:

```rust
#[derive(Debug, thiserror::Error)]
pub enum TypfError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Font error: {0}")]
    Font(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("FFI error: null pointer")]
    NullPointer,
}

// Convert to C error codes
impl From<TypfError> for i32 {
    fn from(err: TypfError) -> i32 {
        match err {
            TypfError::NullPointer => -1,
            TypfError::Io(_) => -3,
            TypfError::Font(_) => -4,
            TypfError::InvalidParameter(_) => -2,
        }
    }
}
```

## Testing FFI Safety

### 1. Miri Testing

Use Miri to detect undefined behavior:

```bash
# Install Miri
rustup +nightly component add miri

# Run FFI tests under Miri
MIRIFLAGS="-Zmiri-disable-isolation" cargo +nightly miri test --features ffi
```

### 2. Valgrind Testing

Check for memory leaks:

```bash
# Build test binary
cargo build --release --features ffi

# Run under Valgrind
valgrind --leak-check=full --show-leak-kinds=all \
    ./target/release/typf_ffi_tests
```

### 3. Property-Based Testing

Use proptest for FFI edge cases:

```rust
#[cfg(test)]
mod ffi_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_ffi_null_safety(
            text in prop::string::string_regex("[^\0]*").unwrap(),
            size in 1.0f32..1000.0,
        ) {
            unsafe {
                // Test with null pointers
                let result = typf_render(
                    std::ptr::null(),
                    text.as_ptr() as *const c_char,
                    size,
                    &mut 0,
                    &mut 0,
                );
                assert!(result.is_null());

                // Test with valid inputs
                let c_text = CString::new(text).unwrap();
                let mut width = 0u32;
                let mut height = 0u32;

                let result = typf_render(
                    c_text.as_ptr(),
                    FONT_PATH.as_ptr(),
                    size,
                    &mut width,
                    &mut height,
                );

                if !result.is_null() {
                    // Verify allocation
                    assert!(width > 0 && height > 0);
                    typf_free(result, (width * height) as usize);
                }
            }
        }
    }
}
```

## Best Practices Checklist

### For Every FFI Function:

- [ ] Validate all pointer parameters for null
- [ ] Use `catch_unwind` to prevent panic propagation
- [ ] Document memory ownership clearly
- [ ] Provide corresponding free/destroy functions
- [ ] Test with Miri and Valgrind
- [ ] Add property-based tests for edge cases
- [ ] Document thread safety guarantees
- [ ] Use `#[repr(C)]` for structs exposed to C
- [ ] Avoid `unsafe` in safe Rust API layers
- [ ] Provide error codes or Result types

### For Python Bindings:

- [ ] Release GIL for CPU-bound operations
- [ ] Convert errors to appropriate Python exceptions
- [ ] Use type hints in generated `.pyi` files
- [ ] Validate parameters at Python boundary
- [ ] Document memory management (who owns what)
- [ ] Test with Python's memory debugger
- [ ] Ensure proper cleanup in `__del__` methods

## Common Pitfalls to Avoid

1. **Forgetting null checks** - Always validate pointers
2. **Panicking across FFI** - Use `catch_unwind`
3. **Memory leaks** - Provide cleanup functions
4. **Use-after-free** - Clear ownership rules
5. **Data races** - Document thread safety
6. **ABI incompatibility** - Use `#[repr(C)]`
7. **String encoding** - Validate UTF-8
8. **Integer overflow** - Check size calculations
9. **Uninitialized memory** - Initialize all outputs
10. **Resource leaks** - RAII patterns in Rust

## References

- [Rust FFI Omnibus](http://jakegoulding.com/rust-ffi-omnibus/)
- [PyO3 User Guide](https://pyo3.rs/)
- [Rustonomicon FFI Chapter](https://doc.rust-lang.org/nomicon/ffi.html)
- [C++ Interop Guidelines](https://rust-lang.github.io/rfcs/2404-c++-ffi.html)