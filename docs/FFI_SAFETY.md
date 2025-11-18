# FFI Safety Patterns in TYPF

## Overview

TYPF exposes its Rust functionality through multiple Foreign Function Interface (FFI) layers:
- C API for direct native integration
- Python bindings via PyO3
- CLI executable

This document describes the safety patterns and best practices used to ensure memory safety, prevent undefined behavior, and maintain API stability across FFI boundaries.

## Core Safety Principles

### 1. No Panics Across FFI Boundaries

**Pattern**: All functions exposed via FFI must catch panics and convert them to error codes.

```rust
#[no_mangle]
pub extern "C" fn typf_render_text(
    text: *const c_char,
    font_data: *const u8,
    font_len: usize,
) -> *mut TypfResult {
    // Catch any panics and return error
    std::panic::catch_unwind(|| {
        // Function implementation
    })
    .unwrap_or_else(|_| {
        // Return error result on panic
        Box::into_raw(Box::new(TypfResult::error("Internal error")))
    })
}
```

**Rationale**: Panics are Rust's mechanism for handling unrecoverable errors. When a panic crosses an FFI boundary, it causes undefined behavior. We must catch all panics at the boundary and convert them to proper error values.

### 2. Null Pointer Validation

**Pattern**: All pointer parameters must be validated before dereferencing.

```rust
#[no_mangle]
pub extern "C" fn typf_process(data: *const u8, len: usize) -> i32 {
    // Validate pointer before use
    if data.is_null() {
        return ERROR_NULL_POINTER;
    }

    // Validate length to prevent overflow
    if len == 0 || len > MAX_ALLOWED_SIZE {
        return ERROR_INVALID_SIZE;
    }

    // Safe to create slice
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    // Process...
}
```

**Rationale**: Null or invalid pointers from C code would cause segmentation faults. Always validate pointers and array bounds before creating slices or references.

### 3. Memory Ownership Transfer

**Pattern**: Use explicit ownership transfer patterns with clear documentation.

```rust
/// Creates a new buffer. Caller must free with typf_free_buffer().
#[no_mangle]
pub extern "C" fn typf_create_buffer(size: usize) -> *mut u8 {
    let mut vec = Vec::<u8>::with_capacity(size);
    let ptr = vec.as_mut_ptr();
    std::mem::forget(vec); // Prevent Rust from deallocating
    ptr
}

/// Frees a buffer created by typf_create_buffer().
#[no_mangle]
pub extern "C" fn typf_free_buffer(ptr: *mut u8, capacity: usize) {
    if !ptr.is_null() {
        unsafe {
            // Reconstruct Vec to properly deallocate
            let _ = Vec::from_raw_parts(ptr, 0, capacity);
        }
    }
}
```

**Rationale**: Memory allocated by Rust must be freed by Rust. Provide matching allocation/deallocation functions and clearly document ownership transfer.

### 4. String Handling

**Pattern**: Always validate UTF-8 when converting C strings to Rust strings.

```rust
use std::ffi::CStr;

#[no_mangle]
pub extern "C" fn typf_parse_font_name(name: *const c_char) -> i32 {
    if name.is_null() {
        return ERROR_NULL_POINTER;
    }

    let c_str = unsafe { CStr::from_ptr(name) };

    // Validate UTF-8
    match c_str.to_str() {
        Ok(rust_str) => {
            // Process valid UTF-8 string
            process_name(rust_str);
            SUCCESS
        }
        Err(_) => ERROR_INVALID_UTF8
    }
}
```

**Rationale**: C strings may contain invalid UTF-8. Always validate before converting to Rust strings to prevent panics.

## Python Bindings (PyO3) Safety

### 1. GIL Management

**Pattern**: Release the GIL for long-running operations.

```rust
use pyo3::prelude::*;

#[pyfunction]
fn render_font_py(font_data: &[u8], text: &str) -> PyResult<Vec<u8>> {
    // Release GIL for CPU-intensive work
    Python::with_gil(|py| {
        py.allow_threads(|| {
            // Long-running operation without GIL
            render_font_internal(font_data, text)
        })
    })
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}
```

**Rationale**: Holding the GIL during long operations blocks all Python threads. Release it for CPU-intensive work.

### 2. Exception Conversion

**Pattern**: Convert all Rust errors to Python exceptions.

```rust
use pyo3::exceptions::PyValueError;

#[pyfunction]
fn load_font(path: &str) -> PyResult<Font> {
    Font::load(path)
        .map_err(|e| match e {
            FontError::FileNotFound => {
                PyErr::new::<pyo3::exceptions::PyFileNotFoundError, _>(
                    format!("Font file not found: {}", path)
                )
            }
            FontError::InvalidFormat => {
                PyErr::new::<PyValueError, _>("Invalid font format")
            }
            _ => PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                e.to_string()
            )
        })
}
```

**Rationale**: Rust `Result` types must be converted to Python exceptions for proper error handling in Python code.

### 3. Memory Views

**Pattern**: Use Python buffer protocol for zero-copy data transfer.

```rust
use pyo3::buffer::PyBuffer;

#[pyfunction]
fn process_image_buffer(buffer: &PyBuffer<u8>) -> PyResult<()> {
    // Zero-copy access to Python buffer
    let data = unsafe {
        std::slice::from_raw_parts(
            buffer.buf_ptr() as *const u8,
            buffer.len_bytes()
        )
    };

    // Process without copying
    process_image(data)?;
    Ok(())
}
```

**Rationale**: Copying large buffers between Python and Rust is expensive. Use buffer protocol for zero-copy access.

## Thread Safety

### 1. Send and Sync Boundaries

**Pattern**: Ensure types crossing FFI boundaries implement Send/Sync appropriately.

```rust
/// Wrapper that can be safely shared across threads
#[repr(C)]
pub struct TypfContext {
    inner: Arc<Mutex<TypfContextInner>>,
}

// Safe to send across threads
unsafe impl Send for TypfContext {}
unsafe impl Sync for TypfContext {}

#[no_mangle]
pub extern "C" fn typf_context_create() -> *mut TypfContext {
    Box::into_raw(Box::new(TypfContext {
        inner: Arc::new(Mutex::new(TypfContextInner::new())),
    }))
}
```

**Rationale**: FFI contexts may be accessed from multiple threads. Use proper synchronization primitives.

### 2. Callback Safety

**Pattern**: Wrap callbacks in safe abstractions.

```rust
type ProgressCallback = unsafe extern "C" fn(progress: f32, user_data: *mut c_void);

#[no_mangle]
pub extern "C" fn typf_render_with_progress(
    data: *const u8,
    len: usize,
    callback: Option<ProgressCallback>,
    user_data: *mut c_void,
) -> i32 {
    // Wrap unsafe callback in safe closure
    let safe_callback = |progress: f32| {
        if let Some(cb) = callback {
            unsafe { cb(progress, user_data) };
        }
    };

    // Use safe callback internally
    render_with_callback(data, len, safe_callback)
}
```

**Rationale**: C callbacks are inherently unsafe. Wrap them in safe abstractions before use.

## Error Handling

### 1. Error Codes

**Pattern**: Define clear error codes for C API.

```rust
#[repr(C)]
pub enum TypfError {
    Success = 0,
    NullPointer = -1,
    InvalidArgument = -2,
    OutOfMemory = -3,
    IoError = -4,
    ParseError = -5,
    // ... more specific errors
}

impl From<std::io::Error> for TypfError {
    fn from(_: std::io::Error) -> Self {
        TypfError::IoError
    }
}
```

**Rationale**: C code expects integer error codes. Define a clear mapping from Rust errors.

### 2. Last Error Pattern

**Pattern**: Store last error message for retrieval.

```rust
thread_local! {
    static LAST_ERROR: RefCell<Option<String>> = RefCell::new(None);
}

fn set_last_error(err: impl ToString) {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = Some(err.to_string());
    });
}

#[no_mangle]
pub extern "C" fn typf_get_last_error() -> *const c_char {
    LAST_ERROR.with(|e| {
        e.borrow()
            .as_ref()
            .map(|s| s.as_ptr() as *const c_char)
            .unwrap_or(std::ptr::null())
    })
}
```

**Rationale**: C code needs access to error messages. Store them thread-locally for retrieval.

## Resource Management

### 1. RAII Wrappers

**Pattern**: Use RAII for automatic resource cleanup.

```rust
pub struct TypfFont {
    handle: NonNull<FontInternal>,
}

impl Drop for TypfFont {
    fn drop(&mut self) {
        unsafe {
            // Cleanup resources
            let _ = Box::from_raw(self.handle.as_ptr());
        }
    }
}

#[no_mangle]
pub extern "C" fn typf_font_create() -> *mut TypfFont {
    Box::into_raw(Box::new(TypfFont {
        handle: NonNull::new(Box::into_raw(Box::new(FontInternal::new())))
            .expect("allocation failed"),
    }))
}
```

**Rationale**: RAII ensures resources are properly cleaned up even in error cases.

### 2. Reference Counting

**Pattern**: Use Arc for shared ownership across FFI.

```rust
#[repr(C)]
pub struct SharedFont {
    inner: Arc<Font>,
}

#[no_mangle]
pub extern "C" fn typf_font_clone(font: *const SharedFont) -> *mut SharedFont {
    if font.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let font = &*font;
        Box::into_raw(Box::new(SharedFont {
            inner: Arc::clone(&font.inner),
        }))
    }
}
```

**Rationale**: Reference counting allows safe sharing of resources across FFI boundaries.

## Testing FFI Safety

### 1. Fuzz Testing

```rust
#[cfg(test)]
mod fuzz_tests {
    use super::*;

    #[test]
    fn fuzz_null_pointers() {
        // Test all functions with null pointers
        assert_eq!(typf_process(std::ptr::null(), 0), ERROR_NULL_POINTER);
        assert_eq!(typf_process(std::ptr::null(), 100), ERROR_NULL_POINTER);
    }

    #[test]
    fn fuzz_invalid_utf8() {
        let invalid_utf8 = [0xFF, 0xFE, 0x00];
        let result = typf_parse_string(invalid_utf8.as_ptr() as *const c_char);
        assert_eq!(result, ERROR_INVALID_UTF8);
    }
}
```

### 2. Valgrind Testing

```bash
# Run tests under valgrind to detect memory issues
cargo test --release
valgrind --leak-check=full ./target/release/typf_test
```

### 3. Thread Sanitizer

```bash
# Build with thread sanitizer
RUSTFLAGS="-Z sanitizer=thread" cargo test --target x86_64-unknown-linux-gnu
```

## Best Practices Summary

1. **Always validate inputs** - Check all pointers, lengths, and strings
2. **Catch all panics** - Use `catch_unwind` at FFI boundaries
3. **Document ownership** - Be explicit about who owns memory
4. **Use safe abstractions** - Wrap unsafe code in safe Rust APIs
5. **Test extensively** - Fuzz test, use sanitizers, check edge cases
6. **Provide error information** - Use error codes and messages
7. **Version your API** - Plan for backward compatibility
8. **Zero-copy when possible** - Use buffer protocols and memory views
9. **Release the GIL** - For Python bindings during long operations
10. **Thread-safe by default** - Use Arc, Mutex, and proper synchronization

## Common Pitfalls to Avoid

1. **Forgetting to validate pointers** - Always check for null
2. **Panicking across FFI** - Causes undefined behavior
3. **Mismatched allocators** - Memory allocated by Rust must be freed by Rust
4. **Assuming UTF-8** - C strings may not be valid UTF-8
5. **Holding the GIL too long** - Blocks all Python threads
6. **Not handling errors** - Convert all Results to appropriate error codes
7. **Race conditions** - Ensure thread safety for shared resources
8. **Memory leaks** - Use RAII and proper cleanup functions
9. **Buffer overflows** - Always validate array bounds
10. **ABI mismatches** - Use `#[repr(C)]` for FFI structs

---

*Last Updated: November 17, 2025*
*TYPF Version: 0.8.0*