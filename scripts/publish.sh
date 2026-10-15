#!/usr/bin/env bash
# this_file: scripts/publish.sh
# Publish every workspace crate in dependency order, or upload audited CI wheels.
set -euo pipefail
cd "$(dirname "$0")/.."
dry_run=false
crates=true
pypi=true
dist=dist
while [[ $# -gt 0 ]]; do
    case "$1" in
        --dry-run) dry_run=true ;;
        --crates) pypi=false ;;
        --pypi) crates=false ;;
        --dist) shift; dist="${1:?--dist requires a directory}" ;;
        *) echo "Usage: $0 [--dry-run] [--crates|--pypi] [--dist DIR]" >&2; exit 2 ;;
    esac
    shift
done
python="${PYTHON:-python3}"
"$python" scripts/release.py validate
if "$crates"; then
    order=$("$python" scripts/release.py order)
    while IFS= read -r package; do
        if "$dry_run"; then
            echo "cargo publish --locked -p $package"
        else
            cargo publish --locked -p "$package"
            sleep 5
        fi
    done <<< "$order"
fi
if "$pypi"; then
    shopt -s nullglob
    artifacts=("$dist"/*.whl "$dist"/*.tar.gz)
    if [[ ${#artifacts[@]} -eq 0 ]]; then
        echo "No built distributions in $dist; download the Release workflow artifacts first." >&2
        exit 1
    fi
    "$python" scripts/release.py audit "${artifacts[@]}"
    if "$dry_run"; then
        echo "uvx twine check and upload ${#artifacts[@]} audited artifacts"
    else
        uvx twine check "${artifacts[@]}"
        uvx twine upload "${artifacts[@]}"
    fi
fi
