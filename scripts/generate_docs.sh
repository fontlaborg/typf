#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REFERENCE_DIR="$ROOT_DIR/src_docs/reference"
OUTPUT_FILE="$REFERENCE_DIR/rust-api.md"
TEMP_OUTPUT="$(mktemp)"
CRATE_MANIFEST="$ROOT_DIR/crates/typf/Cargo.toml"
BACKUP_MANIFEST="$(mktemp)"
SANITIZED_MANIFEST="$(mktemp)"

WORKSPACE_VERSION_LINE="$(rg --max-count 1 --no-filename '^version = ' "$ROOT_DIR/Cargo.toml" || true)"
WORKSPACE_AUTHORS_LINE="$(rg --max-count 1 --no-filename '^authors = ' "$ROOT_DIR/Cargo.toml" || true)"
WORKSPACE_EDITION_LINE="$(rg --max-count 1 --no-filename '^edition = ' "$ROOT_DIR/Cargo.toml" || true)"
WORKSPACE_LICENSE_LINE="$(rg --max-count 1 --no-filename '^license = ' "$ROOT_DIR/Cargo.toml" || true)"

WORKSPACE_VERSION="${WORKSPACE_VERSION_LINE#*\"}"
WORKSPACE_VERSION="${WORKSPACE_VERSION%\"*}"
WORKSPACE_AUTHORS="${WORKSPACE_AUTHORS_LINE#*= }"
WORKSPACE_EDITION="${WORKSPACE_EDITION_LINE#*\"}"
WORKSPACE_EDITION="${WORKSPACE_EDITION%\"*}"
WORKSPACE_LICENSE="${WORKSPACE_LICENSE_LINE#*\"}"
WORKSPACE_LICENSE="${WORKSPACE_LICENSE%\"*}"

if [[ -z "$WORKSPACE_VERSION" || -z "$WORKSPACE_AUTHORS" || -z "$WORKSPACE_EDITION" || -z "$WORKSPACE_LICENSE" ]]; then
  echo "Unable to hydrate workspace metadata for cargo-readme" >&2
  exit 1
fi

sed \
  -e "s/^version\\.workspace = true\$/version = \"$WORKSPACE_VERSION\"/" \
  -e "s/^authors\\.workspace = true\$/authors = $WORKSPACE_AUTHORS/" \
  -e "s/^edition\\.workspace = true\$/edition = \"$WORKSPACE_EDITION\"/" \
  -e "s/^license\\.workspace = true\$/license = \"$WORKSPACE_LICENSE\"/" \
  "$CRATE_MANIFEST" > "$SANITIZED_MANIFEST"

cp "$CRATE_MANIFEST" "$BACKUP_MANIFEST"
restore_manifest() {
  mv "$BACKUP_MANIFEST" "$CRATE_MANIFEST"
}
trap restore_manifest EXIT
mv "$SANITIZED_MANIFEST" "$CRATE_MANIFEST"

mkdir -p "$REFERENCE_DIR"

pushd "$ROOT_DIR/crates/typf" >/dev/null
cargo readme \
  --no-title \
  --no-license \
  --no-badges \
  --output "$TEMP_OUTPUT"
popd >/dev/null

cat <<YAML > "$OUTPUT_FILE"
---
title: Rust API Reference
summary: API overview auto-generated from crate docs.
tags:
  - reference
  - api
warning: This file is generated via scripts/generate_docs.sh (do not edit manually).
---

> ⚠️ Auto-generated from Rust doc comments. Run `scripts/generate_docs.sh` to refresh.

YAML

cat "$TEMP_OUTPUT" >> "$OUTPUT_FILE"
rm "$TEMP_OUTPUT"
restore_manifest
trap - EXIT

echo "Generated $OUTPUT_FILE"
