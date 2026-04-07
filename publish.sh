#!/bin/bash
# publish.sh - robust publishing script for typf workspace
# made by FontLab https://www.fontlab.com/

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DRY_RUN="${DRY_RUN:-false}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info()    { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error()   { echo -e "${RED}[ERROR]${NC} $1"; }
log_dry()     { echo -e "${YELLOW}[DRY-RUN]${NC} $1"; }

usage() {
    cat << EOF
Usage: $0 [OPTIONS] [COMMAND]

Publish script for typf workspace — syncs versions from git tags,
publishes Rust crates to crates.io and Python package to PyPI.

COMMANDS:
    publish         Publish all packages (default)
    rust-only       Publish only Rust crates
    python-only     Publish only Python package
    sync            Sync versions from git tag to Cargo.toml (no publish)
    check           Check publishing status

OPTIONS:
    --dry-run       Show what would be published without actually publishing
    -h, --help      Show this help

EXAMPLES:
    $0                  # Publish all
    $0 --dry-run        # Show what would happen
    $0 rust-only        # Rust crates only
    $0 sync             # Just update version numbers

EOF
}

# ---------------------------------------------------------------------------
# Version management
# ---------------------------------------------------------------------------

# Derive version from the latest vN.N.N git tag
get_git_version() {
    local tag
    tag=$(git -C "$SCRIPT_DIR" describe --tags --abbrev=0 --match 'v[0-9]*' 2>/dev/null) || {
        log_error "No vN.N.N git tag found; create one first (e.g. git tag v5.0.8)"
        return 1
    }
    local version="${tag#v}"
    if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        log_error "Tag '$tag' is not valid semver (expected vX.Y.Z)"
        return 1
    fi
    echo "$version"
}

# Update workspace version and all internal path dependencies
sync_versions() {
    local version="$1"
    log_info "Syncing all Cargo.toml versions to $version ..."

    cd "$SCRIPT_DIR"

    # 1. Workspace root: [workspace.package] version
    perl -0pi -e "s/^(version\s*=\s*)\"[^\"]*\"/\${1}\"${version}\"/m" Cargo.toml

    # 2. Workspace dependency path+version entries for internal crates
    for dep in typf typf-bench typf-core typf-export typf-fontdb typf-input typf-unicode \
               typf-os typf-os-mac typf-os-win \
               typf-render-cg typf-render-color typf-render-json typf-render-opixa \
               typf-render-skia typf-render-svg typf-render-vello-cpu typf-render-vello \
               typf-render-zeno typf-shape-ct typf-shape-hb typf-shape-hr \
               typf-shape-icu-hb typf-shape-none; do
        # Update version in workspace dependency lines that have both path and version
        perl -pi -e "s/(${dep}\s*=\s*\{[^}]*version\s*=\s*)\"[^\"]*\"/\${1}\"${version}\"/g" Cargo.toml
    done

    # 3. Individual crate Cargo.toml files that have their own version = line
    #    (crates using version.workspace = true get it automatically)
    for toml in cli/Cargo.toml core/Cargo.toml unicode/Cargo.toml fontdb/Cargo.toml \
                input/Cargo.toml export/Cargo.toml export-svg/Cargo.toml main/Cargo.toml \
                tools/typf-bench/Cargo.toml bindings/py/Cargo.toml; do
        if [[ -f "$toml" ]]; then
            # Only update the first version = line (the [package] one)
            perl -pi -e 'if (!$done && s/^(version\s*=\s*)"[^"]*"/${1}"'"${version}"'"/) { $done = 1 }' "$toml"
        fi
    done

    # 4. Cross-crate path dependencies with pinned versions (e.g. in cli/Cargo.toml)
    for toml in cli/Cargo.toml bindings/py/Cargo.toml backends/typf-os/Cargo.toml; do
        if [[ -f "$toml" ]]; then
            perl -pi -e "s/(typf-core\s*=\s*\{[^}]*version\s*=\s*)\"[^\"]*\"/\${1}\"=${version}\"/g" "$toml"
        fi
    done

    log_success "All versions synced to $version"
}

# ---------------------------------------------------------------------------
# Publishing helpers
# ---------------------------------------------------------------------------

# Check if a crate version is already on crates.io
crate_published() {
    local crate_name="$1" version="$2"
    curl -sf "https://crates.io/api/v1/crates/$crate_name/$version" >/dev/null 2>&1
}

# Check if Python package version is on PyPI
pypi_published() {
    local package_name="$1" version="$2"
    curl -sf "https://pypi.org/pypi/$package_name/$version/json" >/dev/null 2>&1
}

# Publish a single Rust crate with rate-limit awareness
publish_crate() {
    local crate_path="$1"
    local crate_name="$2"
    local version="$3"

    if crate_published "$crate_name" "$version"; then
        log_warning "$crate_name@$version already on crates.io — skipping"
        return 0
    fi

    if [[ "$DRY_RUN" == "true" ]]; then
        log_dry "Would publish $crate_name@$version from $crate_path"
        return 0
    fi

    log_info "Publishing $crate_name@$version ..."
    cd "$SCRIPT_DIR/$crate_path"

    if ! cargo publish; then
        log_error "Failed to publish $crate_name@$version"
        cd "$SCRIPT_DIR"
        return 1
    fi

    cd "$SCRIPT_DIR"
    log_success "Published $crate_name@$version"

    # crates.io rate limit: wait for the registry to index before next publish
    log_info "Waiting 30s for crates.io rate limit ..."
    sleep 30
    return 0
}

# Publish Python package
publish_python() {
    local version="$1"

    if pypi_published "typfpy" "$version"; then
        log_warning "typfpy@$version already on PyPI — skipping"
        return 0
    fi

    if [[ "$DRY_RUN" == "true" ]]; then
        log_dry "Would publish typfpy@$version to PyPI"
        return 0
    fi

    log_info "Building typfpy@$version ..."
    cd "$SCRIPT_DIR"

    if ! uv build; then
        log_error "Failed to build Python package"
        return 1
    fi

    log_info "Publishing typfpy@$version to PyPI ..."
    if ! uv publish; then
        log_error "Failed to publish to PyPI"
        return 1
    fi

    log_success "Published typfpy@$version to PyPI"
}

# ---------------------------------------------------------------------------
# Main commands
# ---------------------------------------------------------------------------

do_publish() {
    local rust_only="${1:-false}"
    local python_only="${2:-false}"

    # Resolve version from git tag
    local version
    version=$(get_git_version)
    log_info "Version from git tag: $version"

    # Sync versions into Cargo.toml files
    sync_versions "$version"

    # Verify workspace compiles
    log_info "Verifying workspace builds ..."
    if ! cargo check --workspace 2>&1 | tail -3; then
        log_error "Workspace does not compile after version sync"
        return 1
    fi

    local failed=()

    # Publish Rust crates in dependency order
    if [[ "$python_only" != "true" ]]; then
        log_info "Publishing Rust crates (in dependency order) ..."

        # Tier 1: no internal deps
        for spec in "core:typf-core" "unicode:typf-unicode"; do
            local path="${spec%:*}" name="${spec#*:}"
            if ! publish_crate "$path" "$name" "$version"; then
                failed+=("$name")
            fi
        done

        # Tier 2: depends on typf-core
        for spec in "fontdb:typf-fontdb" "input:typf-input" "export:typf-export"; do
            local path="${spec%:*}" name="${spec#*:}"
            if ! publish_crate "$path" "$name" "$version"; then
                failed+=("$name")
            fi
        done

        # Tier 3: depends on tier 2
        for spec in "export-svg:typf-export-svg"; do
            local path="${spec%:*}" name="${spec#*:}"
            if ! publish_crate "$path" "$name" "$version"; then
                failed+=("$name")
            fi
        done

        # Tier 4: CLI / main / bench
        for spec in "cli:typf-cli" "tools/typf-bench:typf-bench" "main:typf"; do
            local path="${spec%:*}" name="${spec#*:}"
            if ! publish_crate "$path" "$name" "$version"; then
                failed+=("$name")
            fi
        done
    fi

    # Publish Python package
    if [[ "$rust_only" != "true" ]]; then
        if ! publish_python "$version"; then
            failed+=("typfpy")
        fi
    fi

    # Report
    if [[ ${#failed[@]} -eq 0 ]]; then
        log_success "All packages published successfully!"
    else
        log_error "Failed to publish: ${failed[*]}"
        return 1
    fi
}

do_check() {
    local version
    version=$(get_git_version)
    log_info "Git tag version: $version"

    log_info "Rust crates:"
    for spec in "core:typf-core" "unicode:typf-unicode" "fontdb:typf-fontdb" \
                "input:typf-input" "export:typf-export" "export-svg:typf-export-svg" \
                "cli:typf-cli" "tools/typf-bench:typf-bench" "main:typf"; do
        local name="${spec#*:}"
        if crate_published "$name" "$version"; then
            echo -e "  $name@$version  ${GREEN}published${NC}"
        else
            echo -e "  $name@$version  ${YELLOW}not published${NC}"
        fi
    done

    log_info "Python:"
    if pypi_published "typfpy" "$version"; then
        echo -e "  typfpy@$version  ${GREEN}published${NC}"
    else
        echo -e "  typfpy@$version  ${YELLOW}not published${NC}"
    fi
}

# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

main() {
    local command="publish"
    local rust_only=false
    local python_only=false

    while [[ $# -gt 0 ]]; do
        case "$1" in
            --dry-run)    DRY_RUN=true; shift ;;
            -h|--help)    usage; exit 0 ;;
            publish)      command="publish"; shift ;;
            rust-only)    command="publish"; rust_only=true; shift ;;
            python-only)  command="publish"; python_only=true; shift ;;
            sync)         command="sync"; shift ;;
            check)        command="check"; shift ;;
            *)            log_error "Unknown option: $1"; usage; exit 1 ;;
        esac
    done

    case "$command" in
        publish) do_publish "$rust_only" "$python_only" ;;
        sync)
            local version; version=$(get_git_version)
            log_info "Version from git tag: $version"
            sync_versions "$version"
            ;;
        check) do_check ;;
    esac
}

main "$@"
