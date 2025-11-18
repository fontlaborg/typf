#!/bin/bash
# Build script for TYPF
# Made by FontLab https://www.fontlab.com/

set -e

echo "Building TYPF workspace (excluding Python bindings)..."
cargo build --release --workspace --exclude typf-python

echo ""
echo "Installing typf-cli..."
cargo install --path typf-cli

echo ""
echo "Building Python bindings with maturin..."
cd python
uv run maturin develop --release --features "python,icu,mac,orge,skiahb"
cd ..

echo ""
echo "Installing Python package system-wide..."
# uv pip install --system --upgrade .
uv venv --python 3.12 --clear
source .venv/bin/activate
uv pip install --upgrade .

echo ""
echo "✅ Build and installation complete!"
echo ""
echo "Installed components:"
echo "  - typf-cli (Rust CLI tool)"
echo "  - typf (Python package with native bindings)"

python toy.py render
python toy.py bench
