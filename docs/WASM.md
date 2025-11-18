# WebAssembly Support

TYPF can be compiled to WebAssembly for use in web browsers.

## Features

- Minimal build size with `wasm-minimal` feature
- JavaScript API through `wasm-bindgen`
- Text rendering to RGBA bitmaps
- Panic hook for better error messages in browser console

## Building

### Prerequisites

```bash
# Install Rust WASM target
rustup target add wasm32-unknown-unknown

# Install wasm-pack (optional, for easier builds)
cargo install wasm-pack
```

### Build Commands

```bash
# Basic WASM build
cargo build --target wasm32-unknown-unknown \
  --package typf \
  --no-default-features \
  --features minimal

# Build with wasm-bindgen support
cargo build --target wasm32-unknown-unknown \
  --package typf \
  --no-default-features \
  --features minimal,wasm

# Build with wasm-pack (recommended)
cd crates/typf
wasm-pack build \
  --target web \
  --features wasm,minimal \
  --no-default-features
```

## JavaScript API

The WASM module exposes the following API:

```javascript
import init, { WasmRenderer, measure_text } from './typf.js';

// Initialize the WASM module
await init();

// Create a renderer
const renderer = new WasmRenderer();

// Render text to RGBA pixels
const pixels = renderer.render_text(
    "Hello WASM!",  // text
    24.0,          // font size
    null,          // optional width
    null           // optional height
);

// Measure text width
const width = measure_text("Hello", 24.0);

// Get version
console.log(renderer.version());
```

## Example HTML

See `scripts/build-wasm.sh` which generates a complete example HTML file demonstrating the WASM API.

## Size Optimization

For minimal bundle size:

1. Use `wasm-minimal` feature (no Unicode processing, basic shaping only)
2. Build with size optimization: `--profile wasm`
3. Use `wee_alloc` for smaller allocator (optional)
4. Strip debug symbols in release builds

## Browser Compatibility

- Chrome 89+
- Firefox 89+
- Safari 14.1+
- Edge 89+

WebAssembly SIMD is not used to maintain broader compatibility.

## Limitations

Current WASM build limitations:

- No file system access (fonts must be embedded or loaded via JS)
- No parallel rendering (single-threaded in browser)
- Basic shaping only (no HarfBuzz in WASM yet)
- No direct GPU access (CPU rendering only)

## Future Improvements

- [ ] HarfBuzz WASM build
- [ ] Font loading from JS ArrayBuffer
- [ ] WebGL/WebGPU rendering backend
- [ ] SIMD support when available
- [ ] Streaming text rendering