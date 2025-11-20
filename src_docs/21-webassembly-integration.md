# Chapter 21: WebAssembly Integration

## Overview

TYPF's WebAssembly (Wasm) integration brings high-performance text rendering to web browsers, enabling native-quality text rendering in web applications. Built with Rust's superior Wasm tooling and optimized for browser environments, the Wasm module provides the same text rendering capabilities as the native library while leveraging browser APIs for accelerated graphics. This chapter covers the complete WebAssembly integration, from basic setup to advanced optimization techniques.

## Architecture

### Wasm Module Structure

```rust
// lib.rs - Main Wasm entry point
use wasm_bindgen::prelude::*;
use js_sys::Promise;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

#[wasm_bindgen]
pub struct WasmRenderer {
    pipeline: typf::Pipeline,
    font_cache: FontCache,
}

#[wasm_bindgen]
impl WasmRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> Result<WasmRenderer, JsValue>;
    
    #[wasm_bindgen]
    pub fn render_to_canvas(
        &mut self, 
        text: &str, 
        canvas: &HtmlCanvasElement,
        options: JsValue
    ) -> Result<(), JsValue>;
    
    #[wasm_bindgen]
    pub async fn render_to_blob(
        &mut self,
        text: &str,
        options: JsValue
    ) -> Result<Promise, JsValue>;
    
    #[wasm_bindgen]
    pub fn shape_text(&self, text: &str, font_bytes: &[u8]) -> Result<JsValue, JsValue>;
}
```

### Browser Integration Layer

```javascript
// typf-wasm.js - JavaScript wrapper
class TypfRenderer {
    constructor(config = {}) {
        this.wasmModule = null;
        this.renderer = null;
        this.initialized = false;
        this.init(config);
    }
    
    async init(config) {
        // Load Wasm module
        this.wasmModule = await import('./typf_wasm.js');
        await this.wasmModule.default();
        
        // Create renderer instance
        this.renderer = new this.wasmModule.WasmRenderer(JSON.stringify(config));
        this.initialized = true;
    }
    
    async renderToCanvas(text, canvas, options = {}) {
        if (!this.initialized) {
            throw new Error('Renderer not initialized');
        }
        
        return this.renderer.render_to_canvas(
            text,
            canvas,
            JSON.stringify(options)
        );
    }
    
    async renderToBlob(text, format = 'png', options = {}) {
        if (!this.initialized) {
            throw new Error('Renderer not initialized');
        }
        
        const fullOptions = { ...options, format };
        return this.renderer.render_to_blob(
            text,
            JSON.stringify(fullOptions)
        );
    }
}
```

## Installation and Setup

### Building the Wasm Module

```bash
# Install Wasm target
rustup target add wasm32-unknown-unknown

# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build Wasm package
cd crates/typf-wasm
wasm-pack build --target web --out-dir pkg

# Build with optimization
wasm-pack build --target web --out-dir pkg --release
```

### Project Setup

```bash
# Initialize web project
mkdir typf-web-app
cd typf-web-app

# Setup basic project structure
mkdir -p {src,pkg,public}
touch {index.html,src/main.js,src/styles.css}

# Copy Wasm package
cp ../typf/crates/typf-wasm/pkg/* ./pkg/

# Setup development server
npm init -y
npm install --save-dev live-server
```

### Basic HTML Template

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>TYPF WebAssembly Demo</title>
    <link rel="stylesheet" href="src/styles.css">
</head>
<body>
    <div class="container">
        <h1>TYPF WebAssembly Text Rendering</h1>
        
        <div class="controls">
            <input type="text" id="textInput" placeholder="Enter text to render" value="Hello, WebAssembly!">
            <select id="fontSelect">
                <option value="default">Default Font</option>
                <option value="serif">Serif</option>
                <option value="sans-serif">Sans Serif</option>
            </select>
            <input type="number" id="fontSize" value="24" min="8" max="72">
            <button id="renderButton">Render</button>
        </div>
        
        <div class="canvas-container">
            <canvas id="renderCanvas" width="800" height="200"></canvas>
        </div>
        
        <div class="output">
            <h3>Rendered Output:</h3>
            <div id="outputContainer"></div>
        </div>
        
        <div class="info">
            <h3>Performance Info:</h3>
            <div id="performanceInfo"></div>
        </div>
    </div>
    
    <script type="module" src="src/main.js"></script>
</body>
</html>
```

## Basic Usage

### Simple Canvas Rendering

```javascript
// src/main.js - Basic usage example
import { TypfRenderer } from './typf-wasm.js';

class TextRenderingApp {
    constructor() {
        this.renderer = new TypfRenderer();
        this.setupElements();
        this.setupEventListeners();
    }
    
    setupElements() {
        this.canvas = document.getElementById('renderCanvas');
        this.ctx = this.canvas.getContext('2d');
        this.textInput = document.getElementById('textInput');
        this.fontSelect = document.getElementById('fontSelect');
        this.fontSize = document.getElementById('fontSize');
        this.renderButton = document.getElementById('renderButton');
        this.outputContainer = document.getElementById('outputContainer');
        this.performanceInfo = document.getElementById('performanceInfo');
    }
    
    setupEventListeners() {
        this.renderButton.addEventListener('click', () => this.renderText());
        this.textInput.addEventListener('input', () => this.debounceRender());
        this.fontSize.addEventListener('change', () => this.renderText());
    }
    
    async renderText() {
        const text = this.textInput.value || 'Hello, WebAssembly!';
        const fontSize = parseInt(this.fontSize.value);
        const fontFamily = this.fontSelect.value;
        
        try {
            const startTime = performance.now();
            
            // Clear canvas
            this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
            
            // Render text to canvas
            await this.renderer.renderToCanvas(text, this.canvas, {
                fontSize: fontSize,
                fontFamily: fontFamily,
                color: '#000000',
                backgroundColor: '#ffffff',
                quality: 'high'
            });
            
            const endTime = performance.now();
            const renderTime = endTime - startTime;
            
            // Update performance info
            this.updatePerformanceInfo(renderTime);
            
            // Also create blob for download
            this.renderToBlob(text);
            
        } catch (error) {
            console.error('Rendering failed:', error);
            this.showError(error.message);
        }
    }
    
    async renderToBlob(text) {
        try {
            const blob = await this.renderer.renderToBlob(text, 'png', {
                fontSize: parseInt(this.fontSize.value),
                width: this.canvas.width,
                height: this.canvas.height
            });
            
            // Create download link
            const url = URL.createObjectURL(blob);
            const downloadLink = document.createElement('a');
            downloadLink.href = url;
            downloadLink.download = 'typf-render.png';
            downloadLink.textContent = 'Download PNG';
            
            // Clear previous links
            this.outputContainer.innerHTML = '';
            this.outputContainer.appendChild(downloadLink);
            
            // Also show image preview
            const img = document.createElement('img');
            img.src = url;
            img.style.maxWidth = '100%';
            img.style.height = 'auto';
            this.outputContainer.appendChild(img);
            
        } catch (error) {
            console.error('Blob rendering failed:', error);
        }
    }
    
    updatePerformanceInfo(renderTime) {
        this.performanceInfo.innerHTML = `
            <p><strong>Render Time:</strong> ${renderTime.toFixed(2)}ms</p>
            <p><strong>Canvas Size:</strong> ${this.canvas.width}x${this.canvas.height}px</p>
            <p><strong>Text Length:</strong> ${this.textInput.value.length} characters</p>
        `;
    }
    
    showError(message) {
        this.performanceInfo.innerHTML = `
            <p style="color: red;"><strong>Error:</strong> ${message}</p>
        `;
    }
    
    debounceRender() {
        clearTimeout(this.debounceTimeout);
        this.debounceTimeout = setTimeout(() => this.renderText(), 300);
    }
}

// Initialize app when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    new TextRenderingApp();
});
```

### Text Shaping in Browser

```javascript
// Text shaping example
async function demonstrateTextShaping() {
    const renderer = new TypfRenderer();
    
    // Sample texts for different scripts
    const texts = [
        'English text',
        'مرحبا بالعالم', // Arabic
        'こんにちは世界',  // Japanese
        '你好世界',       // Chinese
        'Привет мир'      // Russian
    ];
    
    for (const text of texts) {
        try {
            const shapingResult = await renderer.shapeText(text, {
                fontSize: 24,
                includeMetrics: true,
                includePositions: true,
                scriptDetection: true
            });
            
            console.log(`Text: ${text}`);
            console.log(`Script: ${shapingResult.script}`);
            console.log(`Direction: ${shapingResult.direction}`);
            console.log(`Glyphs: ${shapingResult.glyphs.length}`);
            console.log('---');
            
        } catch (error) {
            console.error(`Shaping failed for "${text}":`, error);
        }
    }
}
```

## Advanced Features

### Font Loading and Management

```javascript
// Font management system
class FontManager {
    constructor(renderer) {
        this.renderer = renderer;
        this.loadedFonts = new Map();
    }
    
    async loadFont(fontUrl, fontName) {
        if (this.loadedFonts.has(fontName)) {
            return this.loadedFonts.get(fontName);
        }
        
        try {
            // Fetch font file
            const response = await fetch(fontUrl);
            if (!response.ok) {
                throw new Error(`Failed to fetch font: ${response.statusText}`);
            }
            
            const fontBuffer = await response.arrayBuffer();
            const fontBytes = new Uint8Array(fontBuffer);
            
            // Register font with renderer
            await this.renderer.registerFont(fontName, fontBytes);
            this.loadedFonts.set(fontName, fontBytes);
            
            console.log(`Font loaded: ${fontName}`);
            return fontBytes;
            
        } catch (error) {
            console.error(`Failed to load font ${fontName}:`, error);
            throw error;
        }
    }
    
    async preloadFonts(fontList) {
        const loadPromises = fontList.map(({ url, name }) => 
            this.loadFont(url, name).catch(error => {
                console.warn(`Failed to preload ${name}:`, error);
                return null;
            })
        );
        
        await Promise.all(loadPromises);
        console.log(`Preloaded ${this.loadedFonts.size} fonts`);
    }
    
    getLoadedFonts() {
        return Array.from(this.loadedFonts.keys());
    }
}

// Usage example
async function setupCustomFonts() {
    const renderer = new TypfRenderer();
    const fontManager = new FontManager(renderer);
    
    // Preload custom fonts
    await fontManager.preloadFonts([
        { url: './fonts/noto-sans-regular.ttf', name: 'Noto Sans' },
        { url: './fonts/noto-sans-bold.ttf', name: 'Noto Sans Bold' },
        { url: './fonts/noto-arabic-regular.ttf', name: 'Noto Arabic' },
        { url: './fonts/noto-japanese-regular.ttf', name: 'Noto Japanese' }
    ]);
    
    // Use custom font
    await renderer.renderToCanvas('Custom font rendering', canvas, {
        fontFamily: 'Noto Sans',
        fontSize: 32
    });
}
```

### Performance Optimization

```javascript
// Performance optimization utilities
class PerformanceOptimizer {
    constructor() {
        this.renderCache = new Map();
        this.metricsHistory = [];
        this.maxCacheSize = 100;
    }
    
    // Generate cache key for render request
    generateCacheKey(text, options) {
        const stableOptions = {
            fontSize: options.fontSize,
            fontFamily: options.fontFamily,
            width: options.width,
            height: options.height,
            color: options.color
        };
        return `${text}:${JSON.stringify(stableOptions)}`;
    }
    
    // cached rendering
    async cachedRender(renderer, text, canvas, options) {
        const cacheKey = this.generateCacheKey(text, options);
        
        // Check cache
        if (this.renderCache.has(cacheKey)) {
            const cachedResult = this.renderCache.get(cacheKey);
            return this.applyCachedResult(cachedResult, canvas);
        }
        
        // Render and cache
        const startTime = performance.now();
        await renderer.renderToCanvas(text, canvas, options);
        const endTime = performance.now();
        
        // Cache the result
        const imageData = canvas.toDataURL();
        this.renderCache.set(cacheKey, {
            imageData,
            renderTime: endTime - startTime,
            timestamp: Date.now()
        });
        
        // Manage cache size
        this.manageCacheSize();
        
        // Track performance
        this.trackPerformance(endTime - startTime);
        
        return imageData;
    }
    
    applyCachedResult(cachedResult, canvas) {
        const img = new Image();
        img.onload = () => {
            canvas.getContext('2d').drawImage(img, 0, 0);
        };
        img.src = cachedResult.imageData;
        return cachedResult.imageData;
    }
    
    manageCacheSize() {
        if (this.renderCache.size > this.maxCacheSize) {
            // Remove oldest entries
            const entries = Array.from(this.renderCache.entries());
            entries.sort((a, b) => a[1].timestamp - b[1].timestamp);
            
            const toRemove = entries.slice(0, this.renderCache.size - this.maxCacheSize);
            toRemove.forEach(([key]) => this.renderCache.delete(key));
        }
    }
    
    trackPerformance(renderTime) {
        this.metricsHistory.push({
            renderTime,
            timestamp: Date.now()
        });
        
        // Keep only recent history
        if (this.metricsHistory.length > 100) {
            this.metricsHistory = this.metricsHistory.slice(-100);
        }
    }
    
    getPerformanceStats() {
        if (this.metricsHistory.length === 0) return null;
        
        const times = this.metricsHistory.map(m => m.renderTime);
        const avg = times.reduce((sum, time) => sum + time, 0) / times.length;
        const min = Math.min(...times);
        const max = Math.max(...times);
        
        return {
            averageRenderTime: avg,
            minRenderTime: min,
            maxRenderTime: max,
            totalRenders: this.metricsHistory.length,
            cacheSize: this.renderCache.size
        };
    }
}
```

### Responsive Canvas Handling

```javascript
// Responsive canvas management
class ResponsiveCanvas {
    constructor(canvasId, containerId) {
        this.canvas = document.getElementById(canvasId);
        this.container = document.getElementById(containerId);
        this.pixelRatio = window.devicePixelRatio || 1;
        
        this.setupResizeHandling();
        this.updateCanvasSize();
    }
    
    setupResizeHandling() {
        // Use ResizeObserver for modern browsers
        if ('ResizeObserver' in window) {
            const resizeObserver = new ResizeObserver(() => {
                this.updateCanvasSize();
            });
            resizeObserver.observe(this.container);
        } else {
            // Fallback to window resize
            window.addEventListener('resize', () => {
                this.updateCanvasSize();
            });
        }
    }
    
    updateCanvasSize() {
        const rect = this.container.getBoundingClientRect();
        
        // Set canvas size accounting for device pixel ratio
        this.canvas.width = rect.width * this.pixelRatio;
        this.canvas.height = rect.height * this.pixelRatio;
        
        // Scale canvas back to CSS size
        this.canvas.style.width = `${rect.width}px`;
        this.canvas.style.height = `${rect.height}px`;
        
        // Scale context for sharp rendering
        const ctx = this.canvas.getContext('2d');
        ctx.scale(this.pixelRatio, this.pixelRatio);
        
        // Notify listeners
        this.onResize(rect);
    }
    
    onResize(rect) {
        // Override in subclasses
        console.log(`Canvas resized to ${rect.width}x${rect.height}`);
    }
    
    getContext() {
        return this.canvas.getContext('2d');
    }
    
    getWidth() {
        return this.canvas.width / this.pixelRatio;
    }
    
    getHeight() {
        return this.canvas.height / this.pixelRatio;
    }
}

// Usage with text rendering
class TextRenderingCanvas extends ResponsiveCanvas {
    constructor(canvasId, containerId, renderer) {
        super(canvasId, containerId);
        this.renderer = renderer;
    }
    
    async renderText(text, options = {}) {
        const renderOptions = {
            ...options,
            width: this.getWidth(),
            height: this.getHeight(),
            pixelRatio: this.pixelRatio
        };
        
        await this.renderer.renderToCanvas(text, this.canvas, renderOptions);
    }
    
    onResize(rect) {
        super.onResize(rect);
        // Re-render content after resize
        if (this.currentText && this.currentOptions) {
            this.renderText(this.currentText, this.currentOptions);
        }
    }
}
```

## Integration Examples

### React Integration

```jsx
// TypfCanvas.jsx - React component
import React, { useRef, useEffect, useState, useCallback } from 'react';
import { TypfRenderer } from '../typf-wasm.js';

const TypfCanvas = ({ 
    text, 
    fontSize = 24, 
    fontFamily = 'sans-serif',
    color = '#000000',
    backgroundColor = '#ffffff',
    onRenderComplete = null 
}) => {
    const canvasRef = useRef(null);
    const rendererRef = useRef(null);
    const [isRendering, setIsRendering] = useState(false);
    const [renderError, setRenderError] = useState(null);
    
    // Initialize renderer
    useEffect(() => {
        const initRenderer = async () => {
            try {
                const renderer = new TypfRenderer();
                await renderer.init({
                    enableCaching: true,
                    maxCacheSize: 100
                });
                rendererRef.current = renderer;
            } catch (error) {
                setRenderError(error.message);
            }
        };
        
        initRenderer();
    }, []);
    
    // Handle text rendering
    const renderText = useCallback(async () => {
        if (!rendererRef.current || !canvasRef.current || isRendering) {
            return;
        }
        
        setIsRendering(true);
        setRenderError(null);
        
        try {
            const startTime = performance.now();
            
            await rendererRef.current.renderToCanvas(text, canvasRef.current, {
                fontSize,
                fontFamily,
                color,
                backgroundColor,
                quality: 'high'
            });
            
            const endTime = performance.now();
            const renderTime = endTime - startTime;
            
            if (onRenderComplete) {
                onRenderComplete({ renderTime, success: true });
            }
            
        } catch (error) {
            setRenderError(error.message);
            if (onRenderComplete) {
                onRenderComplete({ error: error.message, success: false });
            }
        } finally {
            setIsRendering(false);
        }
    }, [text, fontSize, fontFamily, color, backgroundColor, isRendering, onRenderComplete]);
    
    // Render when props change
    useEffect(() => {
        if (rendererRef.current) {
            renderText();
        }
    }, [renderText]);
    
    // Handle download
    const handleDownload = useCallback(async () => {
        if (!rendererRef.current) return;
        
        try {
            const blob = await rendererRef.current.renderToBlob(text, 'png', {
                fontSize,
                fontFamily,
                color,
                backgroundColor
            });
            
            const url = URL.createObjectURL(blob);
            const link = document.createElement('a');
            link.href = url;
            link.download = 'typf-render.png';
            link.click();
            
            URL.revokeObjectURL(url);
        } catch (error) {
            setRenderError(error.message);
        }
    }, [text, fontSize, fontFamily, color, backgroundColor]);
    
    return (
        <div className="typf-canvas-container">
            <canvas
                ref={canvasRef}
                width={800}
                height={200}
                className="typf-canvas"
                style={{
                    width: '100%',
                    height: 'auto',
                    border: '1px solid #ccc',
                    borderRadius: '4px'
                }}
            />
            
            {isRendering && (
                <div className="rendering-indicator">
                    Rendering...
                </div>
            )}
            
            {renderError && (
                <div className="render-error">
                    Error: {renderError}
                </div>
            )}
            
            <button 
                onClick={handleDownload}
                disabled={isRendering || !rendererRef.current}
                className="download-button"
            >
                Download PNG
            </button>
        </div>
    );
};

export default TypfCanvas;
```

### Vue.js Integration

```vue
<!-- TypfRenderer.vue - Vue component -->
<template>
  <div class="typf-renderer">
    <div class="controls">
      <input 
        v-model="textInput" 
        @input="debounceRender"
        placeholder="Enter text to render"
        class="text-input"
      />
      
      <select v-model="selectedFont" @change="renderText" class="font-select">
        <option value="sans-serif">Sans Serif</option>
        <option value="serif">Serif</option>
        <option value="monospace">Monospace</option>
      </select>
      
      <input 
        v-model.number="fontSize" 
        type="range" 
        min="12" 
        max="72"
        @input="renderText"
        class="font-size-slider"
      />
      
      <span class="font-size-label">{{ fontSize }}px</span>
    </div>
    
    <canvas
      ref="canvas"
      :width="canvasWidth"
      :height="canvasHeight"
      class="render-canvas"
    ></canvas>
    
    <div v-if="isRendering" class="status rendering">
      Rendering...
    </div>
    
    <div v-if="renderError" class="status error">
      Error: {{ renderError }}
    </div>
    
    <div v-if="performanceInfo" class="performance-info">
      <p>Render time: {{ performanceInfo.renderTime.toFixed(2) }}ms</p>
      <p>Canvas: {{ canvasWidth }}x{{ canvasHeight }}px</p>
    </div>
    
    <button @click="downloadImage" :disabled="isRendering" class="download-btn">
      Download PNG
    </button>
  </div>
</template>

<script>
import { TypfRenderer } from '../typf-wasm.js';

export default {
  name: 'TypfRenderer',
  
  props: {
    initialText: {
      type: String,
      default: 'Hello, Vue + WebAssembly!'
    },
    width: {
      type: Number,
      default: 800
    },
    height: {
      type: Number,
      default: 200
    }
  },
  
  data() {
    return {
      renderer: null,
      textInput: this.initialText,
      selectedFont: 'sans-serif',
      fontSize: 24,
      isRendering: false,
      renderError: null,
      performanceInfo: null,
      canvasWidth: this.width,
      canvasHeight: this.height,
      debounceTimer: null
    };
  },
  
  async mounted() {
    await this.initializeRenderer();
    this.renderText();
  },
  
  methods: {
    async initializeRenderer() {
      try {
        this.renderer = new TypfRenderer();
        await this.renderer.init({
          enableCaching: true,
          performanceMode: true
        });
      } catch (error) {
        this.renderError = error.message;
        console.error('Renderer initialization failed:', error);
      }
    },
    
    async renderText() {
      if (!this.renderer || this.isRendering) return;
      
      this.isRendering = true;
      this.renderError = null;
      
      try {
        const startTime = performance.now();
        
        await this.renderer.renderToCanvas(this.textInput, this.$refs.canvas, {
          fontSize: this.fontSize,
          fontFamily: this.selectedFont,
          color: '#000000',
          backgroundColor: '#ffffff'
        });
        
        const endTime = performance.now();
        this.performanceInfo = {
          renderTime: endTime - startTime
        };
        
      } catch (error) {
        this.renderError = error.message;
        console.error('Rendering failed:', error);
      } finally {
        this.isRendering = false;
      }
    },
    
    debounceRender() {
      clearTimeout(this.debounceTimer);
      this.debounceTimer = setTimeout(() => this.renderText(), 300);
    },
    
    async downloadImage() {
      if (!this.renderer) return;
      
      try {
        const blob = await this.renderer.renderToBlob(this.textInput, 'png', {
          fontSize: this.fontSize,
          fontFamily: this.selectedFont,
          width: this.canvasWidth,
          height: this.canvasHeight
        });
        
        const url = URL.createObjectURL(blob);
        const link = document.createElement('a');
        link.href = url;
        link.download = 'typf-vue-render.png';
        link.click();
        
        URL.revokeObjectURL(url);
      } catch (error) {
        this.renderError = error.message;
      }
    }
  }
};
</script>

<style scoped>
.typf-renderer {
  max-width: 800px;
  margin: 0 auto;
  padding: 20px;
}

.controls {
  display: flex;
  gap: 10px;
  margin-bottom: 20px;
  align-items: center;
}

.text-input {
  flex: 1;
  padding: 8px;
  border: 1px solid #ccc;
  border-radius: 4px;
}

.font-select {
  padding: 8px;
  border: 1px solid #ccc;
  border-radius: 4px;
}

.font-size-slider {
  width: 150px;
}

.render-canvas {
  width: 100%;
  height: auto;
  border: 1px solid #ccc;
  border-radius: 4px;
  margin-bottom: 10px;
}

.status {
  padding: 10px;
  border-radius: 4px;
  margin-bottom: 10px;
}

.status.rendering {
  background-color: #e3f2fd;
  color: #1976d2;
}

.status.error {
  background-color: #ffebee;
  color: #c62828;
}

.performance-info {
  font-size: 12px;
  color: #666;
  margin-bottom: 10px;
}

.download-btn {
  padding: 10px 20px;
  background-color: #1976d2;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

.download-btn:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}
</style>
```

## Performance Optimization

### Memory Management

```javascript
// Memory optimization utilities
class MemoryManager {
    constructor() {
        this.fontCache = new Map();
        this.imageCache = new Map();
        this.maxMemoryUsage = 50 * 1024 * 1024; // 50MB
        this.currentMemoryUsage = 0;
    }
    
    // Estimate memory usage
    estimateMemoryUsage(data) {
        return data.length || data.size || 0;
    }
    
    // Add item to cache with memory tracking
    addToCache(cache, key, data) {
        const dataSize = this.estimateMemoryUsage(data);
        
        // Check if we need to free memory
        if (this.currentMemoryUsage + dataSize > this.maxMemoryUsage) {
            this.freeMemory(dataSize);
        }
        
        // Remove existing item if present
        if (cache.has(key)) {
            const existingData = cache.get(key);
            this.currentMemoryUsage -= this.estimateMemoryUsage(existingData);
        }
        
        cache.set(key, data);
        this.currentMemoryUsage += dataSize;
    }
    
    // Free memory by removing least recently used items
    freeMemory(requiredBytes) {
        const items = Array.from(this.imageCache.entries());
        
        // Sort by last accessed (assuming we track access time)
        items.sort((a, b) => (a[1].accessTime || 0) - (b[1].accessTime || 0));
        
        let freedBytes = 0;
        for (const [key, data] of items) {
            if (freedBytes >= requiredBytes) break;
            
            const dataSize = this.estimateMemoryUsage(data);
            this.imageCache.delete(key);
            freedBytes += dataSize;
            this.currentMemoryUsage -= dataSize;
        }
    }
    
    // Clear all caches
    clearAll() {
        this.fontCache.clear();
        this.imageCache.clear();
        this.currentMemoryUsage = 0;
    }
    
    // Get memory statistics
    getMemoryStats() {
        return {
            currentUsage: this.currentMemoryUsage,
            maxUsage: this.maxMemoryUsage,
            fontCacheSize: this.fontCache.size,
            imageCacheSize: this.imageCache.size,
            usagePercentage: (this.currentMemoryUsage / this.maxMemoryUsage * 100).toFixed(2)
        };
    }
}
```

### Parallel Processing

```javascript
// Web Worker for parallel text processing
// worker.js
self.importScripts('./typf-wasm.js');

class WorkerRenderer {
    constructor() {
        this.renderer = null;
    }
    
    async init() {
        if (!this.renderer) {
            this.renderer = new TypfRenderer();
            await this.renderer.init();
        }
    }
    
    async renderText(data) {
        const { text, options, id } = data;
        
        try {
            await this.init();
            
            // Create offscreen canvas
            const canvas = new OffscreenCanvas(options.width, options.height);
            
            // Render to canvas
            await this.renderer.renderToCanvas(text, canvas, options);
            
            // Convert to blob
            const blob = await canvas.convertToBlob({ type: 'image/png' });
            
            // Send result back
            self.postMessage({
                id,
                success: true,
                blob: blob
            });
            
        } catch (error) {
            self.postMessage({
                id,
                success: false,
                error: error.message
            });
        }
    }
}

const workerRenderer = new WorkerRenderer();

self.onmessage = async (event) => {
    const { type, data } = event.data;
    
    switch (type) {
        case 'render':
            await workerRenderer.renderText(data);
            break;
        
        default:
            console.warn('Unknown worker message type:', type);
    }
};

// Main thread usage
class ParallelRenderer {
    constructor(numWorkers = 4) {
        this.workers = [];
        this.taskQueue = [];
        this.taskCallbacks = new Map();
        this.nextTaskId = 0;
        
        this.initializeWorkers(numWorkers);
    }
    
    initializeWorkers(numWorkers) {
        for (let i = 0; i < numWorkers; i++) {
            const worker = new Worker('./worker.js');
            
            worker.onmessage = (event) => {
                const { id, success, blob, error } = event.data;
                const callback = this.taskCallbacks.get(id);
                
                if (callback) {
                    if (success) {
                        callback.onSuccess(blob);
                    } else {
                        callback.onError(error);
                    }
                    
                    this.taskCallbacks.delete(id);
                }
                
                // Process next task
                this.processNextTask();
            };
            
            this.workers.push({
                worker,
                busy: false
            });
        }
    }
    
    async renderParallel(textList, options = {}) {
        const promises = textList.map((text, index) => 
            this.renderText(text, options, index)
        );
        
        return Promise.all(promises);
    }
    
    renderText(text, options, index = 0) {
        return new Promise((resolve, reject) => {
            const taskId = this.nextTaskId++;
            
            this.taskCallbacks.set(taskId, {
                onSuccess: resolve,
                onError: reject
            });
            
            this.taskQueue.push({
                id: taskId,
                text,
                options,
                index
            });
            
            this.processNextTask();
        });
    }
    
    processNextTask() {
        // Find available worker
        const availableWorker = this.workers.find(w => !w.busy);
        
        if (availableWorker && this.taskQueue.length > 0) {
            const task = this.taskQueue.shift();
            
            availableWorker.busy = true;
            availableWorker.worker.postMessage({
                type: 'render',
                data: task
            });
        }
    }
    
    terminate() {
        this.workers.forEach(({ worker }) => worker.terminate());
        this.workers = [];
        this.taskQueue = [];
        this.taskCallbacks.clear();
    }
}
```

The TYPF WebAssembly integration provides native-quality text rendering capabilities in web browsers while maintaining the performance and flexibility of the Rust engine, enabling sophisticated text processing applications directly in the browser environment.