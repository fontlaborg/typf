#!/bin/bash
# ---------------------------------------------------------------------------
# publish.sh — Ship typf to the world.
#
# typf is a text rendering engine: it takes a font file, a string, and
# rendering parameters, then produces shaped, rasterized text as pixels
# or vector paths. Think of it as the pipeline between "here is a .otf"
# and "here are the glyphs on screen."
#
# This script publishes the typf Rust crates to crates.io (Rust's package
# registry) and the Python bindings to PyPI (Python's package registry).
#
# The single source of truth for the version number is the latest git tag
# matching vN.N.N — for example, v5.0.9. This script reads that tag, stamps
# every Cargo.toml and pyproject.toml with the version, verifies the
# workspace compiles, then publishes each crate in dependency order with
# automatic rate-limit handling.
#
# Typical workflow:
#   gitnextver          # bumps tag: v5.0.8 -> v5.0.9, pushes to remote
#   ./publish.sh        # syncs 5.0.9 into manifests, publishes everything
#
# made by FontLab https://www.fontlab.com/
# ---------------------------------------------------------------------------

# Fail on any error (-e), undefined variable (-u), or broken pipe (-o pipefail).
# This prevents the script from silently continuing after a failure.
set -euo pipefail

# Resolve the directory where this script lives, regardless of where it's
# called from. Every path in this script is relative to SCRIPT_DIR.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Set DRY_RUN=true (via env or --dry-run flag) to preview what would happen
# without uploading anything. Useful before a real release.
DRY_RUN="${DRY_RUN:-false}"

# ---------------------------------------------------------------------------
# Terminal colors — makes log output scannable at a glance.
# NC = "no color" resets back to the terminal default.
# ---------------------------------------------------------------------------
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

# ---------------------------------------------------------------------------
# Usage — printed with --help or on unrecognized input.
# ---------------------------------------------------------------------------
usage() {
    cat << 'EOF'
USAGE
    ./publish.sh [OPTIONS] [COMMAND]

    Publishes the typf text rendering engine to crates.io and PyPI.
    Reads the version from the latest git tag (e.g. v5.0.9), stamps it
    into every manifest, then uploads in dependency order.

COMMANDS
    publish       Publish all Rust crates + Python package. (Default.)
    rust-only     Publish Rust crates only — skip PyPI.
    python-only   Publish Python package only — skip crates.io.
    sync          Write the git-tag version into all Cargo.toml files,
                  but don't publish anything. Handy for local testing.
    check         Show which versions are already published and which
                  are missing from the registries.

OPTIONS
    --dry-run     Walk through every step, print what *would* happen,
                  but don't upload. The safest first move.
    -h, --help    Print this help and exit.

EXAMPLES
    ./publish.sh                  # Full publish: Rust + Python
    ./publish.sh --dry-run        # Preview without uploading
    ./publish.sh rust-only        # Crates.io only
    ./publish.sh sync             # Stamp versions, stop there
    ./publish.sh check            # Registry status report

PREREQUISITES
    - cargo        Rust toolchain (rustup.rs)
    - uv           Python package manager (astral.sh/uv)
    - A git tag    matching vN.N.N (create with: git tag v5.0.9)
    - Credentials  CARGO_REGISTRY_TOKEN for crates.io,
                   UV_PUBLISH_TOKEN for PyPI

EOF
}

# ===========================================================================
#  VERSION MANAGEMENT
#
#  typf uses git tags as the canonical version. The tag v5.0.9 means
#  "version 5.0.9." This script reads that tag and writes "5.0.9" into
#  every Cargo.toml — both the workspace root (which member crates inherit)
#  and the few crates that pin exact versions on their siblings.
#
#  Why not hardcode versions in Cargo.toml? Because then every release
#  requires editing 20+ files by hand. One tag, one script, zero drift.
# ===========================================================================

# Read the most recent vN.N.N tag from git history.
# Returns just the number part: "5.0.9" (no leading "v").
# Fails loudly if no tag exists — you can't publish what you haven't tagged.
get_git_version() {
    local tag
    tag=$(git -C "$SCRIPT_DIR" describe --tags --abbrev=0 --match 'v[0-9]*' 2>/dev/null) || {
        log_error "No vN.N.N git tag found."
        log_error "Create one first:  git tag v5.0.9 && git push --tags"
        return 1
    }

    # Strip the leading "v" — crates.io and PyPI want bare numbers.
    local version="${tag#v}"

    # Guard: must be exactly three dot-separated numbers.
    # "5.0.9" passes. "5.0.9-beta" does not — semver pre-release
    # labels need separate handling if you ever want them.
    if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        log_error "Tag '$tag' is not valid semver (expected vX.Y.Z, got $tag)"
        return 1
    fi

    echo "$version"
}

# Write the resolved version into every Cargo.toml that needs it.
#
# The typf workspace has ~40 crates. Most inherit their version from the
# workspace root via `version.workspace = true` in their own Cargo.toml.
# But some declare explicit versions — especially crates that pin exact
# versions on siblings (e.g., typf-cli depends on typf-core = "=5.0.9").
#
# This function updates four layers, in order:
#
#   1. The workspace-wide version in the root Cargo.toml.
#      Every crate that says `version.workspace = true` picks this up
#      automatically — no per-crate edit needed.
#
#   2. Workspace dependency entries (root Cargo.toml, [workspace.dependencies]).
#      These look like: typf-core = { path = "core", version = "5.0.8" }
#      The `path` tells cargo "use the local checkout for builds."
#      The `version` tells crates.io "when someone installs this as a
#      dependency, require exactly this version." Both must agree.
#
#   3. Individual crate Cargo.toml files that declare their own `version = `.
#      Most crates inherit, but a few (bindings, CLI) set it explicitly.
#
#   4. Cross-crate pinned dependencies — lines like:
#        typf-core = { path = "../../core", version = "=5.0.8" }
#      The "=" prefix means "exactly this version, not newer." Necessary
#      so that crates.io resolves to the right sibling during verification.
sync_versions() {
    local version="$1"
    log_info "Syncing all Cargo.toml versions to $version ..."

    cd "$SCRIPT_DIR"

    # --- Layer 1: workspace root [workspace.package] version ---
    # This is the version every member crate inherits.
    perl -0pi -e "s/^(version\s*=\s*)\"[^\"]*\"/\${1}\"${version}\"/m" Cargo.toml

    # --- Layer 2: workspace dependency entries ---
    # The root Cargo.toml lists every internal crate under
    # [workspace.dependencies] with both a path (for local builds) and
    # a version (for crates.io resolution). Update each version in place.
    #
    # The list covers all publishable crates: core engine, export formats,
    # font database, input handling, Unicode processing, platform-native
    # backends (macOS CoreText, Windows DirectWrite), pluggable shapers
    # (HarfBuzz, CoreText, ICU) and renderers (Skia, Zeno, Vello, etc.).
    for dep in typf typf-bench typf-core typf-export typf-fontdb typf-input typf-unicode \
               typf-os typf-os-mac typf-os-win \
               typf-render-cg typf-render-color typf-render-json typf-render-opixa \
               typf-render-skia typf-render-svg typf-render-vello-cpu typf-render-vello \
               typf-render-zeno typf-shape-ct typf-shape-hb typf-shape-hr \
               typf-shape-icu-hb typf-shape-none; do
        perl -pi -e "s/(${dep}\s*=\s*\{[^}]*version\s*=\s*)\"[^\"]*\"/\${1}\"${version}\"/g" Cargo.toml
    done

    # --- Layer 3: per-crate package versions ---
    # Crates that set their own `version = "..."` instead of inheriting.
    # We match only the first `version =` line in each file — that's the
    # [package] version. Later lines (in [dependencies]) are left alone.
    for toml in cli/Cargo.toml core/Cargo.toml unicode/Cargo.toml fontdb/Cargo.toml \
                input/Cargo.toml export/Cargo.toml export-svg/Cargo.toml main/Cargo.toml \
                tools/typf-bench/Cargo.toml bindings/py/Cargo.toml; do
        if [[ -f "$toml" ]]; then
            perl -pi -e 'if (!$done && s/^(version\s*=\s*)"[^"]*"/${1}"'"${version}"'"/) { $done = 1 }' "$toml"
        fi
    done

    # --- Layer 4: exact-version pins in sibling dependencies ---
    # Some crates depend on typf-core with an exact pin: version = "=5.0.9".
    # The "=" prefix is a cargo convention meaning "this exact version, no
    # semver-compatible upgrades." Without it, cargo might resolve to a
    # different minor version on crates.io during the publish verification
    # step, and the build would fail.
    for toml in cli/Cargo.toml bindings/py/Cargo.toml backends/typf-os/Cargo.toml; do
        if [[ -f "$toml" ]]; then
            perl -pi -e "s/(typf-core\s*=\s*\{[^}]*version\s*=\s*)\"[^\"]*\"/\${1}\"=${version}\"/g" "$toml"
        fi
    done

    log_success "All versions synced to $version"
}

# ===========================================================================
#  PUBLISHING HELPERS
#
#  crates.io is the Rust package registry. PyPI is Python's.
#  Both are append-only: once you publish version 5.0.9, you can never
#  overwrite it. If you need to fix something, bump to 5.0.10.
#
#  crates.io enforces rate limits on new crate publications. If you
#  publish too many crates in quick succession, it responds with
#  HTTP 429 ("Too Many Requests") and a Retry-After timestamp.
#  The publish_crate function handles this automatically: it catches
#  the 429 response, extracts the wait time, sleeps, and retries.
# ===========================================================================

# Returns 0 (success/true) if this exact crate version already exists
# on crates.io. Used to skip re-publishing — crates.io rejects duplicates
# anyway, but skipping avoids wasted time and rate-limit budget.
crate_published() {
    local crate_name="$1" version="$2"
    curl -sf "https://crates.io/api/v1/crates/$crate_name/$version" >/dev/null 2>&1
}

# Same check, but for the Python package on PyPI.
pypi_published() {
    local package_name="$1" version="$2"
    curl -sf "https://pypi.org/pypi/$package_name/$version/json" >/dev/null 2>&1
}

# Publish one Rust crate to crates.io with retry on rate limits.
#
# What `cargo publish` does under the hood:
#   1. Packages the crate into a .crate tarball (source code + Cargo.toml).
#   2. Verifies the package builds from the tarball alone — downloads
#      dependencies from crates.io, not the local workspace. This catches
#      missing version pins or undeclared dependencies.
#   3. Uploads the tarball to crates.io.
#   4. Waits for the registry index to update so the next crate in the
#      dependency chain can find this one.
#
# Rate-limit strategy:
#   - After each successful publish, wait 60 seconds. crates.io imposes
#     a cooldown on new crate publications (not updates to existing ones).
#   - On a 429 response, parse the server's Retry-After timestamp if
#     available, otherwise fall back to exponential backoff (60s, 120s,
#     180s ...). Retry up to 5 times before giving up.
#   - On any other error (missing dependency, bad manifest, auth failure),
#     fail immediately — retrying won't help.
publish_crate() {
    local crate_path="$1"   # directory relative to SCRIPT_DIR, e.g. "core"
    local crate_name="$2"   # crate name on crates.io, e.g. "typf-core"
    local version="$3"      # semver string, e.g. "5.0.9"
    local max_retries=5     # give up after this many 429 retries
    local wait_secs=60      # base wait between publishes (seconds)

    # Don't re-publish what's already there.
    if crate_published "$crate_name" "$version"; then
        log_warning "$crate_name@$version already on crates.io — skipping"
        return 0
    fi

    # In dry-run mode, just say what we would do.
    if [[ "$DRY_RUN" == "true" ]]; then
        log_dry "Would publish $crate_name@$version from $crate_path"
        return 0
    fi

    cd "$SCRIPT_DIR/$crate_path"

    local attempt=0
    while (( attempt < max_retries )); do
        attempt=$((attempt + 1))
        log_info "Publishing $crate_name@$version (attempt $attempt/$max_retries) ..."

        # Capture cargo's stdout+stderr so we can inspect errors.
        local output
        if output=$(cargo publish 2>&1); then
            # Success. cargo publish already waited for the index to update.
            cd "$SCRIPT_DIR"
            log_success "Published $crate_name@$version"

            # Pause before the next crate to stay under rate limits.
            log_info "Waiting ${wait_secs}s before next publish ..."
            sleep "$wait_secs"
            return 0
        fi

        # Publish failed. Print cargo's output so the user sees the raw error.
        echo "$output"

        # Is this a rate-limit error? If so, wait and try again.
        if echo "$output" | grep -q "429\|Too Many Requests\|rate.limit"; then
            # crates.io sometimes includes a human-readable timestamp:
            # "try again after Tue, 07 Apr 2026 22:17:27 GMT"
            # Try to parse it so we wait exactly long enough.
            local retry_after
            retry_after=$(echo "$output" | grep -oP 'try again after \K[^"]+' | head -1 || true)
            local secs_to_wait

            if [[ -n "$retry_after" ]]; then
                # Convert the HTTP date to epoch seconds and compute the delta.
                # The -j flag is macOS date; Linux would use date -d instead.
                local now_epoch retry_epoch
                now_epoch=$(date +%s)
                retry_epoch=$(date -j -f "%a, %d %b %Y %H:%M:%S %Z" "$retry_after" +%s 2>/dev/null || echo "0")
                if (( retry_epoch > now_epoch )); then
                    secs_to_wait=$((retry_epoch - now_epoch + 5))  # 5s buffer
                else
                    secs_to_wait=$((wait_secs * attempt))
                fi
            else
                # No parseable timestamp — escalate: 60s, 120s, 180s ...
                secs_to_wait=$((wait_secs * attempt))
            fi

            log_warning "Rate limited. Waiting ${secs_to_wait}s before retry ..."
            sleep "$secs_to_wait"
        else
            # Some other error: missing dependency, bad license, auth problem.
            # Retrying won't help. Bail out and let the user fix it.
            log_error "Failed to publish $crate_name@$version"
            cd "$SCRIPT_DIR"
            return 1
        fi
    done

    log_error "Failed to publish $crate_name@$version after $max_retries attempts (rate limit)"
    cd "$SCRIPT_DIR"
    return 1
}

# Build and publish the Python package (typfpy) to PyPI.
#
# typfpy wraps the Rust typf engine via PyO3 bindings, giving Python
# users access to the same shaping and rendering pipeline. The build
# step compiles the Rust code into a native Python extension (.so / .pyd),
# then packages it as a wheel (.whl).
#
# `uv build` invokes maturin (the Rust-to-Python build tool) behind the
# scenes. `uv publish` uploads the resulting wheel to PyPI.
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
        log_error "Failed to build Python package."
        log_error "Check that maturin and uv are installed, and that the"
        log_error "Rust workspace compiles (cargo check --workspace)."
        return 1
    fi

    log_info "Publishing typfpy@$version to PyPI ..."
    if ! uv publish; then
        log_error "Failed to publish to PyPI."
        log_error "Check UV_PUBLISH_TOKEN is set, or run: uv publish --token <token>"
        return 1
    fi

    log_success "Published typfpy@$version to PyPI"
}

# ===========================================================================
#  MAIN COMMANDS
#
#  do_publish orchestrates the full release. It publishes crates in
#  "tiers" — groups ordered by dependency. Every crate in Tier 2
#  depends on at least one crate in Tier 1, so Tier 1 must be fully
#  published before Tier 2 can begin.
#
#  If any crate in a tier fails, all downstream tiers are skipped.
#  There's no point uploading typf-cli if typf-core never made it
#  to the registry — cargo publish would reject it anyway (missing dep).
#
#  Tier layout for the typf workspace:
#
#    Tier 1 — Foundation. No internal dependencies.
#      typf-core       Core engine: the pipeline, caching, traits, FFI.
#      typf-unicode    Unicode segmentation, bidi, normalization.
#
#    Tier 2 — Depends on typf-core only.
#      typf-fontdb     Font discovery and metadata (wraps skrifa/read-fonts).
#      typf-input      Input text preprocessing.
#      typf-export     Output formats: PNG, PNM, raw pixels.
#      typf-render-color  Color glyph renderer (COLR, SVG, bitmap).
#      typf-render-opixa  Pure Rust rasterizer with SIMD.
#      typf-render-json   JSON data exporter.
#      typf-render-cg     CoreGraphics renderer (macOS).
#      typf-shape-none    Minimal passthrough shaper.
#      typf-shape-hb      HarfBuzz C shaper.
#      typf-shape-hr      Pure Rust shaper via harfrust.
#      typf-shape-ct      CoreText shaper (macOS).
#      typf-shape-icu-hb  ICU + HarfBuzz shaper.
#      typf-os-mac        CoreText linra backend (macOS).
#      typf-os-win        DirectWrite linra backend (Windows).
#
#    Tier 3 — Depends on Tier 2 crates.
#      typf-export-svg    SVG output (needs typf-render-color).
#      typf-render-svg    SVG vector renderer (needs typf-export, render-color).
#      typf-render-vello-cpu  Vello CPU renderer (needs typf-fontdb).
#      typf-render-vello  Vello GPU renderer (needs typf-fontdb, shape-none).
#      typf-os            Platform linra dispatcher (needs os-mac, os-win).
#
#    Tier 4 — Depends on Tier 3 crates.
#      typf-render-skia   Skia renderer (needs typf-render-svg).
#      typf-render-zeno   Zeno renderer (needs typf-render-svg).
#
#    Tier 5 — Top of the tree. Depends on everything below.
#      typf            The main "batteries included" crate that re-exports
#                      the full pipeline for users who want one dependency.
#      typf-cli        Command-line tool: `typf render "Hello" -o hello.png`
#      typf-bench      Benchmarking harness for shaper x renderer combos.
#
#  cargo publish requires ALL dependencies (including optional ones) to
#  exist on crates.io during verification, so every backend crate must
#  be published before any crate that references it.
# ===========================================================================

do_publish() {
    local rust_only="${1:-false}"
    local python_only="${2:-false}"

    # Step 1: Read the version from the latest git tag.
    local version
    version=$(get_git_version)
    log_info "Version from git tag: $version"

    # Step 2: Stamp that version into every Cargo.toml.
    sync_versions "$version"

    # Step 3: Verify the workspace still compiles after version changes.
    # This catches typos in the sync (e.g., a crate whose version wasn't
    # updated) before we spend time uploading.
    log_info "Verifying workspace builds ..."
    if ! cargo check --workspace 2>&1 | tail -3; then
        log_error "Workspace does not compile after version sync"
        return 1
    fi

    # Step 4: Verify every publishable crate has the metadata crates.io requires.
    # crates.io rejects uploads missing `description`, `license`, or `repository`.
    # Catching this here avoids burning rate-limit budget on doomed publishes and
    # prevents a mid-tier failure from cascading into skipped downstream tiers.
    log_info "Validating crate metadata ..."
    local metadata_ok=true
    for spec in "core:typf-core" "unicode:typf-unicode" \
                "fontdb:typf-fontdb" "input:typf-input" "export:typf-export" \
                "backends/typf-render-color:typf-render-color" \
                "backends/typf-render-opixa:typf-render-opixa" \
                "backends/typf-render-json:typf-render-json" \
                "backends/typf-render-cg:typf-render-cg" \
                "backends/typf-shape-none:typf-shape-none" \
                "backends/typf-shape-hb:typf-shape-hb" \
                "backends/typf-shape-hr:typf-shape-hr" \
                "backends/typf-shape-ct:typf-shape-ct" \
                "backends/typf-shape-icu-hb:typf-shape-icu-hb" \
                "backends/typf-os-mac:typf-os-mac" \
                "backends/typf-os-win:typf-os-win" \
                "export-svg:typf-export-svg" \
                "backends/typf-render-svg:typf-render-svg" \
                "backends/typf-render-vello-cpu:typf-render-vello-cpu" \
                "backends/typf-render-vello:typf-render-vello" \
                "backends/typf-os:typf-os" \
                "backends/typf-render-skia:typf-render-skia" \
                "backends/typf-render-zeno:typf-render-zeno" \
                "main:typf" "cli:typf-cli" "tools/typf-bench:typf-bench"; do
        local path="${spec%:*}" name="${spec#*:}"
        local toml="$SCRIPT_DIR/$path/Cargo.toml"
        for field in description license repository; do
            # Match both own field (description = "...") and workspace inheritance
            # (description.workspace = true). The dot-form is how Cargo.toml
            # delegates metadata to the workspace root.
            if ! grep -qE "^${field}(\\.workspace)?\s*=" "$toml"; then
                log_error "$name ($path/Cargo.toml): missing required field '$field'"
                metadata_ok=false
            fi
        done
    done
    if [[ "$metadata_ok" != "true" ]]; then
        log_error "Fix the missing metadata above, then re-run."
        return 1
    fi
    log_success "All crate metadata valid"

    local failed=()
    local tier_failed=false

    # publish_tier: publish a group of crates. If an earlier tier already
    # failed (tier_failed=true), skip this group entirely and mark all
    # its crates as failed — they'd fail anyway due to missing deps.
    publish_tier() {
        local tier_name="$1"; shift
        if [[ "$tier_failed" == "true" ]]; then
            log_warning "Skipping $tier_name — earlier dependency tier failed"
            for spec in "$@"; do
                failed+=("${spec#*:}")
            done
            return
        fi
        log_info "--- $tier_name ---"
        for spec in "$@"; do
            # Each spec is "directory:crate-name", e.g. "core:typf-core".
            local path="${spec%:*}" name="${spec#*:}"
            if ! publish_crate "$path" "$name" "$version"; then
                failed+=("$name")
                tier_failed=true
            fi
        done
    }

    # Publish Rust crates in dependency order.
    if [[ "$python_only" != "true" ]]; then
        log_info "Publishing Rust crates (in dependency order) ..."

        publish_tier "Tier 1: foundation (no internal deps)" \
            "core:typf-core" \
            "unicode:typf-unicode"

        publish_tier "Tier 2: depends on typf-core" \
            "fontdb:typf-fontdb" \
            "input:typf-input" \
            "export:typf-export" \
            "backends/typf-render-color:typf-render-color" \
            "backends/typf-render-opixa:typf-render-opixa" \
            "backends/typf-render-json:typf-render-json" \
            "backends/typf-render-cg:typf-render-cg" \
            "backends/typf-shape-none:typf-shape-none" \
            "backends/typf-shape-hb:typf-shape-hb" \
            "backends/typf-shape-hr:typf-shape-hr" \
            "backends/typf-shape-ct:typf-shape-ct" \
            "backends/typf-shape-icu-hb:typf-shape-icu-hb" \
            "backends/typf-os-mac:typf-os-mac" \
            "backends/typf-os-win:typf-os-win"

        publish_tier "Tier 3: depends on Tier 2 crates" \
            "export-svg:typf-export-svg" \
            "backends/typf-render-svg:typf-render-svg" \
            "backends/typf-render-vello-cpu:typf-render-vello-cpu" \
            "backends/typf-render-vello:typf-render-vello" \
            "backends/typf-os:typf-os"

        publish_tier "Tier 4: depends on Tier 3 crates" \
            "backends/typf-render-skia:typf-render-skia" \
            "backends/typf-render-zeno:typf-render-zeno"

        publish_tier "Tier 5: top-level crates" \
            "main:typf" \
            "cli:typf-cli" \
            "tools/typf-bench:typf-bench"
    fi

    # Publish the Python package last — it wraps the Rust crates,
    # so those must be available on crates.io first (maturin pulls
    # them during the build verification step).
    if [[ "$rust_only" != "true" ]]; then
        if [[ "$tier_failed" == "true" ]]; then
            log_warning "Skipping Python publish — Rust crate failures"
            failed+=("typfpy")
        elif ! publish_python "$version"; then
            failed+=("typfpy")
        fi
    fi

    # Final report.
    if [[ ${#failed[@]} -eq 0 ]]; then
        log_success "All packages published successfully!"
    else
        log_error "Failed to publish: ${failed[*]}"
        log_error "Fix the errors above, bump the tag (gitnextver), and re-run."
        return 1
    fi
}

# Show which crate versions are published and which are missing.
# Quick sanity check after a partial publish or to verify a release.
do_check() {
    local version
    version=$(get_git_version)
    log_info "Git tag version: $version"

    log_info "Rust crates:"
    for spec in "core:typf-core" "unicode:typf-unicode" \
                "fontdb:typf-fontdb" "input:typf-input" "export:typf-export" \
                "backends/typf-render-color:typf-render-color" \
                "backends/typf-render-opixa:typf-render-opixa" \
                "backends/typf-render-json:typf-render-json" \
                "backends/typf-render-cg:typf-render-cg" \
                "backends/typf-shape-none:typf-shape-none" \
                "backends/typf-shape-hb:typf-shape-hb" \
                "backends/typf-shape-hr:typf-shape-hr" \
                "backends/typf-shape-ct:typf-shape-ct" \
                "backends/typf-shape-icu-hb:typf-shape-icu-hb" \
                "backends/typf-os-mac:typf-os-mac" \
                "backends/typf-os-win:typf-os-win" \
                "export-svg:typf-export-svg" \
                "backends/typf-render-svg:typf-render-svg" \
                "backends/typf-render-vello-cpu:typf-render-vello-cpu" \
                "backends/typf-render-vello:typf-render-vello" \
                "backends/typf-os:typf-os" \
                "backends/typf-render-skia:typf-render-skia" \
                "backends/typf-render-zeno:typf-render-zeno" \
                "main:typf" "cli:typf-cli" "tools/typf-bench:typf-bench"; do
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

# ===========================================================================
#  ENTRY POINT — parse arguments, dispatch to the right command.
# ===========================================================================

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
