# Vello GPU Platform Support

Platform support matrix and requirements for the Vello GPU renderer backend.

## Overview

The `typf-render-vello` backend provides GPU-accelerated text rendering using the [Vello](https://github.com/linebender/vello) hybrid renderer. It requires a GPU with compute shader support and uses `wgpu` for cross-platform GPU access.

## Requirements

### Minimum GPU Requirements

Vello requires GPUs with compute shader support. The following GPU generations are supported:

| Platform | Minimum GPU |
|----------|-------------|
| **Windows** | DirectX 12 capable (GTX 600+, HD 4000+, Vega+) |
| **Linux** | Vulkan 1.0 capable (GTX 600+, HD 4000+, GCN+) |
| **macOS** | Metal 2 capable (all Apple Silicon, Intel 2012+) |
| **WebGPU** | WebGPU-enabled browser (see browser matrix below) |

### Software Requirements

| Platform | Backend | Driver/Runtime |
|----------|---------|----------------|
| **Windows** | DirectX 12 | Windows 10 1903+ |
| **Windows** | Vulkan | Vulkan 1.0 driver |
| **Linux** | Vulkan | Mesa 21.0+ or proprietary driver |
| **macOS** | Metal | macOS 10.14+ |
| **iOS** | Metal | iOS 12+ |
| **Android** | Vulkan | Android 7.0+ with Vulkan support |

### Rust Toolchain

- Minimum Rust version: 1.75 (typf workspace MSRV)
- wgpu dependency requires: 1.82+ for core crates

## Platform Test Matrix

### Desktop Platforms

| Platform | Backend | Status | Notes |
|----------|---------|--------|-------|
| **macOS (Apple Silicon)** | Metal | ✅ Verified | Primary development platform |
| **macOS (Intel)** | Metal | ✅ Verified | Requires Metal 2 support |
| **Windows 10/11** | DX12 | 🔶 Expected | Requires testing |
| **Windows 10/11** | Vulkan | 🔶 Expected | Via Vulkan SDK |
| **Ubuntu 22.04** | Vulkan | 🔶 Expected | Requires Mesa 21.0+ |
| **Ubuntu 24.04** | Vulkan | 🔶 Expected | Better driver support |
| **Fedora 38+** | Vulkan | 🔶 Expected | Recent Mesa versions |

Legend: ✅ Verified | 🔶 Expected to work | ❌ Not supported | ⚠️ Known issues

### WebGPU Browser Support

| Browser | Status | Notes |
|---------|--------|-------|
| **Chrome 113+** | ✅ Enabled by default | Best compatibility |
| **Edge 113+** | ✅ Enabled by default | Chromium-based |
| **Firefox** | ⚠️ Behind flag | `dom.webgpu.enabled` in about:config |
| **Safari 17+** | ✅ Enabled by default | Limited to macOS/iOS |
| **Safari 16** | ⚠️ Behind flag | Developer menu enable |
| **Mobile Chrome** | ⚠️ Limited | Android 12+ required |
| **Mobile Safari** | ⚠️ Limited | iOS 17+ required |

### CI/Testing Recommendations

For comprehensive testing, we recommend:

```yaml
# GitHub Actions matrix example
strategy:
  matrix:
    include:
      # macOS - Metal backend
      - os: macos-14        # Apple Silicon runner
        name: macOS-ARM64

      - os: macos-13        # Intel runner
        name: macOS-x64

      # Windows - DirectX 12 backend
      - os: windows-2022
        name: Windows-DX12

      # Linux - Vulkan backend (requires GPU runner)
      - os: ubuntu-22.04
        name: Linux-Vulkan
        # Note: GitHub's standard runners don't have GPUs
        # Use self-hosted runners or GPU CI services
```

## Known Limitations

### GPU-Specific Issues

1. **Intel Integrated Graphics (Linux)**
   - Some older Intel drivers have NULL function pointers for certain extensions
   - Workaround: Update Mesa to 23.0+ or use `MESA_LOADER_DRIVER_OVERRIDE`

2. **AMD (Windows)**
   - Some AMD drivers may require Vulkan SDK installation
   - DirectX 12 backend typically more reliable

3. **NVIDIA (Linux)**
   - Proprietary driver recommended over Nouveau
   - Ensure `libnvidia-gl` package is installed

### Feature Limitations

| Feature | Vello GPU | Notes |
|---------|-----------|-------|
| **Color fonts (COLR v0/v1)** | ⚠️ Limited | Upstream WIP; use `vello-cpu` |
| **SVG-in-OpenType** | ⚠️ Limited | Complex paths may fail |
| **Bitmap glyphs (sbix/CBDT)** | ⚠️ Limited | Texture upload overhead |
| **Very large text** | ⚠️ | GPU memory limits apply |

For color fonts, the CLI will warn users to prefer `vello-cpu` or other CPU renderers.

## Performance Characteristics

### Startup Overhead

Vello GPU has initialization costs:

| Phase | Typical Time |
|-------|--------------|
| wgpu Instance creation | 5-20ms |
| Adapter discovery | 10-50ms |
| Device/Queue creation | 20-100ms |
| Shader compilation (first use) | 50-200ms |
| **Total cold start** | ~100-400ms |

After initialization, rendering is significantly faster for repeated operations.

### Rendering Performance

Approximate throughput on Apple M1 Max:

| Workload | Frames/sec | Notes |
|----------|------------|-------|
| Simple text (100 glyphs) | 500+ fps | GPU-bound |
| Complex text (1000 glyphs) | 200+ fps | GPU-bound |
| Large document (10K glyphs) | 50+ fps | Memory-bound |

### Memory Usage

GPU memory consumption depends on scene complexity:

| Workload | GPU Memory |
|----------|------------|
| Small text | ~10-50 MB |
| Medium document | ~50-200 MB |
| Large document | ~200-500 MB |

## Fallback Behavior

If GPU initialization fails, typf can fall back to CPU rendering:

```rust
use typf_render_vello::VelloConfig;

let config = VelloConfig {
    use_cpu_fallback: true,  // Allow software rasterization
    power_preference: wgpu::PowerPreference::LowPower,
};

let renderer = VelloRenderer::with_config(config)?;
```

The CPU fallback uses wgpu's software rasterizer (`llvmpipe` on Linux, WARP on Windows).

## Recommended Configurations

### High Performance (Desktop)

```rust
let config = VelloConfig {
    use_cpu_fallback: false,
    power_preference: wgpu::PowerPreference::HighPerformance,
};
```

### Battery Efficient (Laptop)

```rust
let config = VelloConfig {
    use_cpu_fallback: true,
    power_preference: wgpu::PowerPreference::LowPower,
};
```

### Maximum Compatibility

```rust
// Use vello-cpu backend instead
use typf_render_vello_cpu::VelloCpuRenderer;
let renderer = VelloCpuRenderer::new();
```

## Testing Your GPU Support

Use the typf CLI to verify GPU support:

```bash
# Check if Vello GPU is available
typf info --renderers

# Test rendering with Vello GPU
typf render "Test" --renderer vello -o test.png

# If GPU fails, try CPU fallback
typf render "Test" --renderer vello-cpu -o test.png
```

## Troubleshooting

### "Failed to find a suitable GPU adapter"

1. Check GPU driver is installed
2. Verify Vulkan/Metal/DX12 support:
   - Windows: `dxdiag` or `vulkaninfo`
   - Linux: `vulkaninfo`
   - macOS: Metal is built-in

### "Shader compilation failed"

1. Update GPU drivers to latest version
2. Try `use_cpu_fallback: true`
3. Report issue with driver/GPU information

### Poor Performance

1. Check `power_preference` setting
2. Ensure discrete GPU is being used (not integrated)
3. Monitor GPU memory usage
4. Consider `vello-cpu` for smaller workloads

---

For the most up-to-date information, see the [wgpu compatibility matrix](https://github.com/gfx-rs/wgpu/wiki) and [Vello project](https://github.com/linebender/vello).
