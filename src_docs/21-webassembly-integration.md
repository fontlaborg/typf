# WebAssembly Integration

Run TypF in browsers and JavaScript environments with WebAssembly.

## Quick Start

```javascript
import { Typf } from '@typf/wasm';

// Initialize TYPF
const typf = await Typf.create();

// Render text
const result = await typf.renderText("Hello WASM", {
  fontSize: 32,
  width: 400,
  height: 100
});

// Get PNG data
const pngData = result.asPng();
```

## Installation

### NPM Package

```bash
npm install @typf/wasm
```

### CDN Usage

```html
<script src="https://unpkg.com/@typf/wasm/dist/typf.js"></script>
<script>
  const typf = await Typf.create();
</script>
```

### Bundle Integration

```javascript
// Vite configuration
export default {
  optimizeDeps: {
    exclude: ['@typf/wasm']
  },
  server: {
    fs: {
      allow: ['.']
    }
  }
};
```

## Core API

### Initialization

```javascript
// Basic initialization
const typf = await Typf.create();

// With specific backends
const typf = await Typf.create({
  shaper: 'harfbuzz',
  renderer: 'orge'
});

// Preload fonts
const typf = await Typf.create({
  fonts: ['Roboto.ttf', 'OpenSans.ttf']
});
```

### Text Rendering

```javascript
// Simple rendering
const result = await typf.renderText("Hello World", {
  fontSize: 24,
  width: 300,
  height: 80
});

// With font
const result = await typf.renderText("Custom Font", {
  font: await loadFont('Roboto.ttf'),
  fontSize: 32,
  width: 400,
  height: 100
});

// Full options
const result = await typf.renderText("Advanced", {
  font: fontData,
  fontSize: 48,
  width: 600,
  height: 200,
  shaper: 'harfbuzz',
  renderer: 'skia',
  antialiasing: true,
  color: { r: 0, g: 0, b: 0, a: 255 },
  background: { r: 255, g: 255, b: 255, a: 0 }
});
```

### Output Formats

```javascript
const result = await typf.renderText("Output", options);

// PNG (default)
const pngData = result.asPng();
const blob = new Blob([pngData], { type: 'image/png' });

// SVG
const svgData = result.asSvg();
const svgString = new TextDecoder().decode(svgData);

// JSON (debug)
const jsonData = result.asJson();
const info = JSON.parse(new TextDecoder().decode(jsonData));

// Raw bitmap data
const bitmap = result.asBitmap();
const { width, height, data } = bitmap; // Uint8Array RGBA像素数据
```

## Font Loading

### Font Sources

```javascript
// From fetch
const fontBuffer = await fetch('Roboto.ttf').then(r => r.arrayBuffer());
const font = await typf.loadFont(fontBuffer);

// From file input
document.getElementById('font-file').addEventListener('change', async (e) => {
  const file = e.target.files[0];
  const fontBuffer = await file.arrayBuffer();
  const font = await typf.loadFont(fontBuffer);
});

// From base64
const base64Font = atob(base64Data);
const fontBytes = new Uint8Array(base64Font.length);
for (let i = 0; i < base64Font.length; i++) {
  fontBytes[i] = base64Font.charCodeAt(i);
}
const font = await typf.loadFont(fontBytes);
```

### Font Management

```javascript
// Load multiple fonts
const fonts = [
  await typf.loadFont(robotoBuffer, 'Roboto'),
  await typf.loadFont(openSansBuffer, 'OpenSans'),
  await typf.loadFont(arabicFont, 'Arabic')
];

// Font fallback testing
const availableFonts = await typf.listFonts();
const supportsArabic = availableFonts.some(font => font.supportsScript('Arabic'));

// Font information
const fontInfo = await font.getInfo();
console.log(fontInfo.family, fontInfo.style, fontInfo.supportsScripts);
```

## Canvas Integration

### Direct Canvas Rendering

```javascript
// Get canvas context
const canvas = document.getElementById('canvas');
const ctx = canvas.getContext('2d');

// Render to ImageData
const result = await typf.renderText("Canvas Text", {
  fontSize: 48,
  width: canvas.width,
  height: canvas.height
});

const bitmap = result.asBitmap();
const imageData = new ImageData(
  new Uint8ClampedArray(bitmap.data),
  bitmap.width,
  bitmap.height
);

ctx.putImageData(imageData, 0, 0);
```

### Responsive Rendering

```javascript
function renderResponsive(text, container) {
  const rect = container.getBoundingClientRect();
  const dpr = window.devicePixelRatio || 1;
  
  return typf.renderText(text, {
    fontSize: Math.floor(16 * dpr),
    width: Math.floor(rect.width * dpr),
    height: Math.floor(rect.height * dpr),
    renderer: 'skia' // Better for high DPI
  });
}

// Handle resize
window.addEventListener('resize', () => {
  renderResponsive("Responsive Text", container)
    .then(result => updateCanvas(result));
});
```

## Performance Optimization

### Memory Management

```javascript
// Reuse Typf instance
const typf = await Typf.create();

// Clear font cache
await typf.clearFontCache();

// Dispose of large results
const result = await typf.renderText(bigText, options);
const imageData = result.asBitmap();
// Process imageData...
result.free(); // Free WASM memory
```

### Parallel Processing

```javascript
// Web Worker integration
// worker.js
importScripts('https://unpkg.com/@typf/wasm/dist/typf.js');

let typf;

self.addEventListener('message', async (e) => {
  if (!typf) {
    typf = await Typf.create();
  }
  
  const { text, options } = e.data;
  const result = await typf.renderText(text, options);
  
  self.postMessage({
    id: e.data.id,
    result: result.asPng()
  });
});

// Main thread
function renderInWorker(text, options) {
  return new Promise((resolve) => {
    const worker = new Worker('worker.js');
    const id = Math.random();
    
    worker.postMessage({ id, text, options });
    worker.onmessage = (e) => {
      if (e.data.id === id) {
        resolve(e.data.result);
        worker.terminate();
      }
    };
  });
}
```

### Streaming Large Texts

```javascript
// Chunked rendering for large documents
async function renderLargeText(text, options) {
  const words = text.split(' ');
  const lines = [];
  let currentLine = '';
  
  for (const word of words) {
    const testLine = currentLine + (currentLine ? ' ' : '') + word;
    const result = await typf.measureText(testLine, options);
    
    if (result.width <= options.width) {
      currentLine = testLine;
    } else {
      if (currentLine) lines.push(currentLine);
      currentLine = word;
    }
  }
  
  if (currentLine) lines.push(currentLine);
  
  // Render each line
  const renderedLines = await Promise.all(
    lines.map(line => typf.renderText(line, options))
  );
  
  return renderedLines;
}
```

## Advanced Features

### Text Shaping Analysis

```javascript
// Get shaping information
const analysis = await typf.analyzeText("Complex text 😊", {
  font: fontData,
  fontSize: 16
});

console.log(`
  Glyphs: ${analysis.glyphs.length}
  Scripts: ${analysis.scripts.join(', ')}
  Direction: ${analysis.direction}
  Cluster breaks: ${analysis.clusters.length}
`);

// Glyph-level information
for (const glyph of analysis.glyphs) {
  console.log(`
    ID: ${glyph.id}
    Position: (${glyph.x}, ${glyph.y})
    Advance: ${glyph.advance}
    Cluster: ${glyph.cluster}
  `);
}
```

### Font Fallback

```javascript
// Configure font fallback
const fallbackFonts = [
  await loadFont('LatinFont.ttf'),
  await loadFont('ArabicFont.ttf'),
  await loadFont('EmojiFont.ttf')
];

await typf.setFontFallback(fallbackFonts);

// Text with mixed scripts
const result = await typf.renderText("Hello مرحبا 👋", options);
```

### Variable Fonts

```javascript
// Load variable font
const varFont = await typf.loadFont(fontBuffer);

// Set variation settings
await varFont.setVariation({
  weight: 700,
  width: 100,
  slant: 0
});

// Render with variation
const result = await typf.renderText("Variable Font", {
  font: varFont,
  fontSize: 48
});

// Adjust variations dynamically
async function animateWeight() {
  for (let weight = 100; weight <= 900; weight += 50) {
    await varFont.setVariation({ weight });
    const result = await typf.renderText("Animated", options);
    updateDisplay(result);
    await new Promise(r => setTimeout(r, 50));
  }
}
```

## Framework Integration

### React Component

```jsx
import React, { useState, useEffect, useRef } from 'react';
import { Typf } from '@typf/wasm';

function TextRenderer({ text, fontSize = 24, font }) {
  const canvasRef = useRef(null);
  const [typf, setTypf] = useState(null);
  
  useEffect(() => {
    Typf.create().then(setTypf);
  }, []);
  
  useEffect(() => {
    if (!typf || !canvasRef.current) return;
    
    const render = async () => {
      const result = await typf.renderText(text, {
        fontSize,
        font,
        width: canvasRef.current.width,
        height: canvasRef.current.height
      });
      
      const bitmap = result.asBitmap();
      const ctx = canvasRef.current.getContext('2d');
      const imageData = new ImageData(
        new Uint8ClampedArray(bitmap.data),
        bitmap.width,
        bitmap.height
      );
      
      ctx.putImageData(imageData, 0, 0);
    };
    
    render();
  }, [typf, text, fontSize, font]);
  
  return <canvas ref={canvasRef} width={400} height={100} />;
}
```

### Vue Component

```vue
<template>
  <canvas ref="canvas" :width="width" :height="height"></canvas>
</template>

<script>
import { Typf } from '@typf/wasm';

export default {
  props: ['text', 'fontSize', 'width', 'height'],
  data: () => ({ typf: null }),
  
  async mounted() {
    this.typf = await Typf.create();
    await this.render();
  },
  
  watch: {
    text() { this.render(); },
    fontSize() { this.render(); }
  },
  
  methods: {
    async render() {
      if (!this.typf) return;
      
      const result = await this.typf.renderText(this.text, {
        fontSize: this.fontSize,
        width: this.width,
        height: this.height
      });
      
      const bitmap = result.asBitmap();
      const ctx = this.$refs.canvas.getContext('2d');
      const imageData = new ImageData(
        new Uint8ClampedArray(bitmap.data),
        bitmap.width,
        bitmap.height
      );
      
      ctx.putImageData(imageData, 0, 0);
    }
  }
};
</script>
```

### Svelte Integration

```svelte
<script>
  import { onMount } from 'svelte';
  import { Typf } from '@typf/wasm';
  
  export let text = 'Hello Svelte';
  export let fontSize = 24;
  
  let canvas;
  let typf;
  
  onMount(async () => {
    typf = await Typf.create();
    render();
  });
  
  async function render() {
    if (!typf || !canvas) return;
    
    const result = await typf.renderText(text, {
      fontSize,
      width: canvas.width,
      height: canvas.height
    });
    
    const bitmap = result.asBitmap();
    const ctx = canvas.getContext('2d');
    const imageData = new ImageData(
      new Uint8ClampedArray(bitmap.data),
      bitmap.width,
      bitmap.height
    );
    
    ctx.putImageData(imageData, 0, 0);
  }
  
  $: if (typf && canvas) render();
</script>

<canvas bind:this={canvas} width={400} height={100}></canvas>
```

## Error Handling

### Common WASM Issues

```javascript
try {
  const typf = await Typf.create();
  const result = await typf.renderText("Test", options);
} catch (error) {
  if (error.message.includes('WASM')) {
    console.error('WASM loading failed:', error);
    // Fallback to server-side rendering
  } else if (error.message.includes('Font')) {
    console.error('Font loading failed:', error);
    // Use default font
  } else {
    console.error('Rendering error:', error);
  }
}
```

### Memory Limits

```javascript
// Monitor memory usage
const memoryUsage = typf.getMemoryUsage();
console.log(`Used: ${memoryUsage.used} bytes`);
console.log(`Available: ${memoryUsage.available} bytes`);

// Handle memory pressure
if (memoryUsage.used / memoryUsage.available > 0.8) {
  console.warn('High memory usage, clearing cache');
  await typf.clearFontCache();
}
```

## Browser Compatibility

### Supported Browsers

- Chrome 57+ (WebAssembly 1.0)
- Firefox 52+ (WebAssembly 1.0)
- Safari 11+ (WebAssembly 1.0)
- Edge 16+ (WebAssembly 1.0)

### Feature Detection

```javascript
// Check WebAssembly support
function supportsWasm() {
  try {
    if (typeof WebAssembly === 'object') {
      const module = new WebAssembly.Module(Uint8Array.of(0x0, 0x61, 0x73, 0x6d));
      if (module instanceof WebAssembly.Module) {
        return new WebAssembly.Instance(module) instanceof WebAssembly.Instance;
      }
    }
  } catch (e) {}
  return false;
}

// Check SharedArrayBuffer support (for performance)
function supportsSharedMemory() {
  return typeof SharedArrayBuffer !== 'undefined';
}

// Fallback handling
if (!supportsWasm()) {
  // Use server-side rendering or canvas 2D fallback
}
```

## Security Considerations

### Loading Safely

```javascript
// Validate font data before loading
async function loadFontSafely(fontData) {
  // Check file size (limit to 50MB)
  if (fontData.byteLength > 50 * 1024 * 1024) {
    throw new Error('Font file too large');
  }
  
  // Validate font header
  const header = new Uint32Array(fontData.slice(0, 12));
  if (header[0] !== 0x10000 && header[0] !== 0x74727565) {
    throw new Error('Invalid font format');
  }
  
  return await typf.loadFont(fontData);
}

// Use CSP headers
// Content-Security-Policy: script-src 'self' 'wasm-unsafe-eval';
```

### Sandboxing

```javascript
// Run in Web Worker for isolation
const worker = new Worker('typf-worker.js', { 
  type: 'module',
  credentials: 'omit'
});

// Use Transferable objects for efficiency
worker.postMessage({
  fontBuffer: fontData,
  text: "Sandboxed text"
}, [fontData]);
```

## Performance Tips

### Optimization Checklist

- [ ] Reuse Typf instances instead of recreating
- [ ] Preload commonly used fonts
- [ ] Use Web Workers for CPU-intensive tasks
- [ ] Clear font cache periodically
- [ ] Choose optimal renderers for target platforms
- [ ] Use appropriate output formats for use cases
- [ ] Implement progressive rendering for large texts

### Benchmarking

```javascript
// Performance measurement
const startTime = performance.now();
const result = await typf.renderText("Benchmark", options);
const endTime = performance.now();

console.log(`Render time: ${endTime - startTime}ms`);

// Memory usage before/after
const memBefore = typf.getMemoryUsage();
const largeResult = await typf.renderText(hugeText, options);
const memAfter = typf.getMemoryUsage();

console.log(`Memory delta: ${memAfter.used - memBefore.used} bytes`);
```

---

WebAssembly brings TYPF's text rendering to browsers with near-native performance. Use it for dynamic typography tools, real-time text effects, and font-intensive web applications.
