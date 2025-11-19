#!/bin/bash
# Build script for TYPF v2.0
# Made by FontLab https://www.fontlab.com/

set -e

echo "Building TYPF v2.0 workspace (excluding Python bindings)..."
cargo build --release --workspace --exclude typf-py

echo ""
echo "Installing typf-cli..."
cargo install --path crates/typf-cli

echo ""
echo "Setting up Python environment..."
# Create or update virtual environment
if [ ! -d ".venv" ]; then
    uv venv --python 3.12
fi
source .venv/bin/activate

# Install maturin and dependencies
echo "Installing Python dependencies..."
uv pip install maturin fire pillow

echo ""
echo "Building Python bindings with maturin..."
cd bindings/python
maturin develop --release --features "shaping-hb,export-png,export-svg"
cd ../..

echo ""
echo "✅ Build and installation complete!"
echo ""
echo "Installed components:"
echo "  - typf-cli (Rust CLI tool)"
echo "  - typf (Python package with native bindings)"
echo ""

echo "Running TYPF tester..."
echo ""
python typf-tester/typfme.py render
echo ""
python typf-tester/typfme.py bench
