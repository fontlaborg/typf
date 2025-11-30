#!/bin/bash
# Build Typf for WebAssembly

set -e

echo "Building Typf for WebAssembly..."

# Install wasm-pack if not present
if ! command -v wasm-pack &> /dev/null; then
    echo "Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Install wasm32 target if not present
rustup target add wasm32-unknown-unknown 2>/dev/null || true

# Build with wasm-pack
echo "Building WASM package..."
cd crates/typf
wasm-pack build \
    --target web \
    --out-dir ../../pkg \
    --features wasm,wasm-minimal \
    --no-default-features

echo "WASM build complete! Output in pkg/"

# Create example HTML file
cat > ../../pkg/example.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Typf WASM Example</title>
    <style>
        body { font-family: sans-serif; padding: 20px; }
        #canvas { border: 1px solid #ccc; }
        #controls { margin: 20px 0; }
        input, button { margin: 5px; padding: 5px; }
    </style>
</head>
<body>
    <h1>Typf WebAssembly Demo</h1>

    <div id="controls">
        <input type="text" id="text" value="Hello WASM!" placeholder="Enter text...">
        <input type="number" id="fontSize" value="24" min="8" max="72">
        <button id="render">Render</button>
    </div>

    <canvas id="canvas"></canvas>

    <script type="module">
        import init, { WasmRenderer, measure_text } from './typf.js';

        async function run() {
            await init();

            const renderer = new WasmRenderer();
            const canvas = document.getElementById('canvas');
            const ctx = canvas.getContext('2d');

            function render() {
                const text = document.getElementById('text').value;
                const fontSize = parseFloat(document.getElementById('fontSize').value);

                // Measure text
                const width = Math.ceil(measure_text(text, fontSize)) + 20;
                const height = Math.ceil(fontSize * 1.5) + 20;

                // Set canvas size
                canvas.width = width;
                canvas.height = height;

                // Render text
                try {
                    const pixels = renderer.render_text(text, fontSize);

                    // Convert to ImageData
                    const imageData = new ImageData(
                        new Uint8ClampedArray(pixels),
                        width,
                        height
                    );

                    // Draw to canvas
                    ctx.putImageData(imageData, 0, 0);
                } catch (e) {
                    console.error('Render error:', e);
                    ctx.fillStyle = 'red';
                    ctx.fillText('Error: ' + e, 10, 20);
                }
            }

            document.getElementById('render').addEventListener('click', render);
            document.getElementById('text').addEventListener('input', render);
            document.getElementById('fontSize').addEventListener('input', render);

            // Initial render
            render();
        }

        run();
    </script>
</body>
</html>
EOF

echo "Example HTML created at pkg/example.html"
echo "To test, run: python3 -m http.server --directory pkg"
