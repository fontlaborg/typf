#!/bin/bash

# Publish script for typf project
# Publishes Rust crates to crates.io and Python package to PyPI

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to abort with error message
abort() {
    print_error "$1"
    exit 1
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check required tools
check_dependencies() {
    print_status "Checking dependencies..."
    
    command_exists cargo || abort "cargo not found. Please install Rust."
    command_exists uv || abort "uv not found. Please install uv."
    
    print_success "All dependencies found"
}

# Function to get current version from workspace
get_workspace_version() {
    grep -E '^\s*version\s*=\s*' Cargo.toml | head -1 | sed -E 's/.*version\s*=\s*"([^"]+)".*/\1/'
}

# Function to check if crate version exists on crates.io
check_crate_published() {
    local crate_name="$1"
    local version="$2"
    
    print_status "Checking if $crate_name@$version exists on crates.io..."
    
    if curl -s -f "https://crates.io/api/v1/crates/$crate_name/$version" >/dev/null 2>&1; then
        return 0  # Version exists
    else
        return 1  # Version doesn't exist
    fi
}

# Function to check if Python package version exists on PyPI
check_python_published() {
    local package_name="$1" 
    local version="$2"
    
    print_status "Checking if $package_name@$version exists on PyPI..."
    
    if curl -s -f "https://pypi.org/pypi/$package_name/$version" >/dev/null 2>&1; then
        return 0  # Version exists
    else
        return 1  # Version doesn't exist
    fi
}

# Function to publish Rust crate
publish_crate() {
    local crate_path="$1"
    local crate_name="$2"
    local version="$3"
    
    if check_crate_published "$crate_name" "$version"; then
        print_warning "$crate_name@$version is already published on crates.io"
        return 0
    fi
    
    print_status "Publishing $crate_name@$version to crates.io..."
    
    cd "$crate_path"
    if cargo publish --dry-run; then
        cargo publish
        print_success "Successfully published $crate_name@$version to crates.io"
    else
        abort "Failed to dry-run publish $crate_name"
    fi
    cd - >/dev/null
    
    # Wait a moment for crates.io to process
    sleep 10
}

# Function to publish Python package
publish_python() {
    local version="$1"
    
    if check_python_published "typfpy" "$version"; then
        print_warning "typfpy@$version is already published on PyPI"
        return 0
    fi
    
    print_status "Building and publishing typfpy@$version to PyPI..."
    
    # Build the Python package
    if ! uv build; then
        abort "Failed to build Python package with uv"
    fi
    
    # Check if build was successful
    local wheel_path
    wheel_path=$(find dist -name "*.whl" -type f | head -1)
    if [[ -z "$wheel_path" ]]; then
        abort "No wheel file found in dist/ directory"
    fi
    
    print_status "Publishing $(basename "$wheel_path") to PyPI..."
    
    # Publish to PyPI
    if uv publish; then
        print_success "Successfully published typfpy@$version to PyPI"
    else
        abort "Failed to publish Python package to PyPI"
    fi
}

# Main function
main() {
    print_status "Starting publish process for typf project..."
    
    # Check dependencies
    check_dependencies
    
    # Get current version
    local version
    version=$(get_workspace_version)
    if [[ -z "$version" ]]; then
        abort "Could not extract version from workspace Cargo.toml"
    fi
    
    print_status "Workspace version: $version"
    
    # Check if we're on the right branch and if working directory is clean
    local current_branch
    current_branch=$(git rev-parse --abbrev-ref HEAD)
    if [[ "$current_branch" != "main" ]] && [[ "$current_branch" != "master" ]]; then
        print_warning "Not on main/master branch (current: $current_branch)"
    fi
    
    if ! git diff-index --quiet HEAD --; then
        abort "Working directory is not clean. Please commit or stash changes before publishing."
    fi
    
    # List of crates to publish (in order of dependency)
    local crates=(
        "crates/typf:typf"
        "crates/typf-core:typf-core"
        "crates/typf-unicode:typf-unicode"
        "crates/typf-fontdb:typf-fontdb"
        "crates/typf-input:typf-input"
        "crates/typf-export:typf-export"
        "crates/typf-export-svg:typf-export-svg"
        "crates/typf-cli:typf-cli"
        "crates/typf-bench:typf-bench"
    )
    
    local any_published=false
    
    # Publish Rust crates
    print_status "Publishing Rust crates..."
    for crate_info in "${crates[@]}"; do
        local crate_path="${crate_info%:*}"
        local crate_name="${crate_info#*:}"
        
        if [[ -d "$crate_path" ]]; then
            local crate_version
            crate_version=$(grep -E '^\s*version\s*=\s*' "$crate_path/Cargo.toml" | head -1 | sed -E 's/.*version\s*=\s*"([^"]+)".*/\1/' || echo "$version")
            
            if publish_crate "$crate_path" "$crate_name" "$crate_version"; then
                any_published=true
            fi
        else
            print_warning "Crate directory $crate_path does not exist"
        fi
    done
    
    # Publish Python package
    print_status "Publishing Python package..."
    if publish_python "$version"; then
        any_published=true
    fi
    
    if [[ "$any_published" == "true" ]]; then
        print_success "Publishing completed successfully!"
        print_status "Version $version has been published to crates.io and/or PyPI"
    else
        print_warning "Nothing was published - all versions were already available"
    fi
}

# Run main function
main "$@"
