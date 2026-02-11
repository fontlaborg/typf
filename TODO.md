<!-- this_file: TODO.md -->
# TODO (derived from TASKS.md)

**Version:** 5.0.2
**Updated:** 2026-02-11
**Source:** `PLANSTEPS/` (split from `TASKS.md`)

## Remaining Tasks

No remaining tasks for v5.0.2. All planned work complete.

## Completed (2026-02-11 script-normalization/limit-helper dedup + file-tracking hygiene micro-sprint)

- [x] Add shared ISO 15924 script-tag normalization utility (`crates/typf-cli/src/script.rs`) and reuse it for render CLI and JSONL `text.script` parsing
- [x] Add shared text-size validator helper (`validate_text_size_limit`) plus shared `MAX_TEXT_CONTENT_BYTES` in `crates/typf-cli/src/limits.rs`
- [x] Replace duplicate render/JSONL text-size validation logic with shared limits helper calls
- [x] Add boundary regression tests for text-size validation (exact-limit accept + over-limit reject)
- [x] Add missing `this_file` markers in `crates/typf-cli/src/*.rs` and `crates/typf-cli/src/commands/*.rs`
- [x] Re-verify with `cargo test -p typf-cli --all-features` and `./test.sh --quick`

## Completed (2026-02-11 render input-source/size-capped input-read/backend-normalization micro-sprint)

- [x] Reject ambiguous render CLI text inputs when multiple explicit sources are provided (`positional text`, `--text`, `--text-file`)
- [x] Switch render CLI `--text-file` and stdin ingestion to bounded reads using shared limits (`MAX_TEXT_CONTENT_BYTES=1_000_000`)
- [x] Normalize render CLI `--shaper`/`--renderer` tokens (trim + case-insensitive + blank defaults) for backend-selection parity with batch mode
- [x] Add regression coverage for ambiguous-source rejection, oversized text-file rejection, and case-insensitive backend selection
- [x] Re-verify with `cargo test -p typf-cli --all-features` and `./test.sh --quick`

## Completed (2026-02-11 BCP47 language-tag validation parity micro-sprint)

- [x] Add shared BCP 47 language-tag normalization utility for typf-cli inputs (`language-tags`-backed canonicalization)
- [x] Validate/canonicalize `typf render --language` and emit explicit invalid-language diagnostics
- [x] Validate/canonicalize JSONL `text.language` with explicit `Invalid text.language` error context
- [x] Validate/canonicalize `typf batch` per-job `language` with explicit `Invalid batch language tag` diagnostics
- [x] Re-verify with `cargo test -p typf-cli --all-features`, `./test.sh --quick`, and full `./test.sh`

## Completed (2026-02-11 resource/input-guardrail micro-sprint)

- [x] Validate render CLI `--color-palette` as a strict CPAL-compatible 16-bit index (`0..=65535`) and reject overflow values
- [x] Add font source file-size guardrail (`MAX_FONT_FILE_BYTES=100 MiB`) before font loading in render CLI and JSONL job processing
- [x] Add bounded structured JSONL read (`MAX_JSONL_BATCH_INPUT_BYTES=32 MiB`) before deserializing batch payloads
- [x] Add regression tests for palette overflow, oversized font source files, and capped JSON batch input reads
- [x] Re-verify with `cargo test -p typf-cli --all-features` and `./test.sh --quick`

## Completed (2026-02-11 render/JSONL script-hint + text-size parity micro-sprint)

- [x] Add render-CLI text payload-size cap (`MAX_TEXT_CONTENT_BYTES=1_000_000`) across positional/`--text`, `--text-file`, and stdin inputs
- [x] Normalize render-CLI `--language` (`trim`, blank→unset) and validate/canonicalize `--script` as ISO 15924-style 4-letter ASCII alpha tags (`auto`/blank→unset)
- [x] Add JSONL `text.script` normalization/validation parity with render CLI and emit explicit `Invalid text.script` diagnostics for invalid values
- [x] Re-verify parity sprint end-to-end with `cargo test -p typf-cli --all-features` and `./test.sh`

## Completed (2026-02-11 JSONL resource-limits/stream-id-cap micro-sprint)

- [x] Add JSONL batch-size cap (`MAX_BATCH_JOBS=10_000`) and reject oversized job lists with explicit validation errors
- [x] Add JSONL `text.content` payload-size cap (`MAX_TEXT_CONTENT_BYTES=1_000_000`) and fail fast before shaping
- [x] Improve stream duplicate-ID diagnostics with first-seen line context and enforce bounded unique stream IDs (`MAX_STREAM_UNIQUE_JOB_IDS=100_000`)

## Completed (2026-02-11 render face-index/glyph-source + JSONL stream duplicate-id micro-sprint)

- [x] Make `typf render` honor `--face-index` by loading via `TypfFontFace::from_file_index(...)`
- [x] Include contextual `face_index=<n>` diagnostics in `typf render` font-load failures
- [x] Trim `--glyph-source` `prefer=`/`deny=` key+list whitespace and reject blank source lists with explicit errors
- [x] Reject duplicate JSONL stream `job.id` values across lines with line-aware validation errors

## Completed (2026-02-11 cross-CLI unicode/color parser-parity micro-sprint)

- [x] Add Rust CLI support for 8-digit `\UXXXXXXXX` Unicode escapes while preserving malformed uppercase escapes literally
- [x] Add Python CLI support for `\UXXXXXXXX` Unicode escapes with malformed-literal preservation parity
- [x] Align Python CLI color parsing with Rust (`RGB`/`RGBA` shorthand + trimmed input support) and add parser regression tests

## Completed (2026-02-11 JSONL font-loader/face-index/text-hint-normalization micro-sprint)

- [x] Replace JSONL job font loading shim with real `TypfFontFace::from_file_index()` loading
- [x] Respect JSONL `font.source.face_index` and include explicit `face_index=<n>` context in load-failure diagnostics
- [x] Normalize JSONL optional `text.language`/`text.script` hints (trim + blank to `None`) before shaping, with regression tests

## Completed (2026-02-11 stream-diagnostics/color-input micro-sprint)

- [x] Add line-aware JSONL stream diagnostics for parse and `job.id` validation failures (synthetic IDs + line-number context)
- [x] Prefix JSONL stream execution error messages with source line numbers while preserving original job IDs
- [x] Harden `typf render` input parsing by supporting trimmed shorthand hex colors (`RGB`/`RGBA`) and contextual invalid font-size diagnostics

## Completed (2026-02-11 JSONL job-identity/rendering-dimensions + batch field-normalization micro-sprint)

- [x] Reject blank, whitespace-padded, and duplicate JSONL `job.id` values before batch execution
- [x] Reject JSONL jobs with `rendering.width`/`rendering.height` set to `0` with explicit validation errors
- [x] Trim/validate per-job `font` path input in `typf batch` and reject blank values
- [x] Normalize per-job `shaper`/`renderer` tokens in `typf batch` (trim + lowercase + blank defaults), and reject blank per-job `format` values explicitly

## Completed (2026-02-11 unicode-escape reliability micro-sprint)

- [x] Decode UTF-16 surrogate-pair escapes (`\uXXXX\uXXXX`) in Rust CLI text input parsing
- [x] Preserve malformed Unicode escape literals verbatim (instead of consuming characters) in Rust and Python CLI decoders
- [x] Add regression tests in Rust and Python for basic `\uXXXX`, braced `\u{...}`, surrogate-pair, and malformed escape cases

## Completed (2026-02-11 finite-font-size validation consistency micro-sprint)

- [x] Reject non-finite font sizes (`NaN`, `+/-inf`) in `typf-core::ShapingParams::validate()` before positive/range checks
- [x] Remove duplicate JSONL non-finite `font.size` guard and rely on core shaping validation as the single authority
- [x] Add regression tests for non-finite `font.size` validation behavior across core and JSONL job processing

## Completed (2026-02-11 CLI input-normalization/output-path/JSONL-format micro-sprint)

- [x] Tighten `typf batch` output pattern validation (`trim`, reject blank, require exactly one `{}` placeholder) with regression tests
- [x] Trim `typf batch` per-job `output` values and reject whitespace-only filenames with regression tests
- [x] Normalize/validate JSONL `rendering.format` (`trim` + case-insensitive `ppm|pgm|pbm|metrics`) and emit canonical lowercase output format values

## Completed (2026-02-11 verification-integrity micro-sprint)

- [x] Canonicalize duplicate OpenType feature tags in render CLI and JSONL parsing with deterministic `last value wins` behavior
- [x] Tighten JSONL `version` parsing to reject malformed values (empty version, non-numeric minor, extra segments)
- [x] Make `scripts/test.sh` fail on Python lint/test failures when those checks run

## Completed (2026-02-11 JSONL determinism/input-normalization micro-sprint)

- [x] Validate JSONL `font.instance.variations` in stable sorted axis-tag order so diagnostics are deterministic
- [x] Trim surrounding whitespace for JSONL `version` and `text.direction` parsing (blank `text.direction` defaults to LTR)
- [x] Reject JSONL jobs with non-finite/non-positive `font.size` before shaping and report explicit `font.size` validation errors

## Completed (2026-02-11 feature-tag diagnostics micro-sprint)

- [x] Make render CLI feature-tag validation report non-ASCII violations before tag-length errors for multibyte input
- [x] Align JSONL feature-tag validation ordering with render CLI (ASCII-range check before length check)
- [x] Add multibyte non-ASCII feature-tag regression tests for both render CLI and JSONL parsers

## Completed (2026-02-11 input-validation parity micro-sprint)

- [x] Reject non-finite shaping font sizes (`NaN`, `+/-inf`) in `typf-core::ShapingParams::validate()`
- [x] Support `png1`, `png4`, and `png8` in `typf batch` per-job `format` parsing
- [x] Trim surrounding whitespace in JSONL `rendering.encoding` (`base64|plain`) parsing and add regression tests

## Completed (2026-02-11 parser consistency micro-sprint)

- [x] Make CLI variation axis-tag validation report non-ASCII violations before length errors for deterministic diagnostics
- [x] Align JSONL `font.instance.variations` axis-tag validation order with CLI (`ASCII range` before `length`)
- [x] Accept mixed comma/tab/newline delimiters in JSONL `text.features` parsing with regression tests

## Completed (2026-02-11 CLI/JSONL hardening micro-sprint)

- [x] Reject unsupported named-instance tokens in CLI `--instance` parsing (`axis=value`/`axis:value` only)
- [x] Validate CLI variation axis tags as 4 printable ASCII bytes and canonicalize parsed variations (sorted, duplicate tags deterministic)
- [x] Validate JSONL `rendering.encoding` (`base64|plain`) plus `font.instance.variations` axis tags/values with deterministic sorted output

## Completed (2026-02-11 batch hardening micro-sprint)

- [x] Validate batch output filename pattern requires `{}` placeholder
- [x] Validate batch output paths are confined to `--output` (reject `..`, absolute paths, and directory-only values)
- [x] Reject unsupported batch output `format` values and unknown JSON fields with regression tests

## Completed (2026-02-11 parser delimiter micro-sprint)

- [x] Accept mixed comma/tab/newline separators for render CLI OpenType feature parsing
- [x] Accept mixed comma/tab/newline separators for render CLI variation-axis parsing
- [x] Accept mixed comma/tab/newline separators for glyph-source list parsing, with regression tests

## Completed (2026-02-11 CLI validation micro-sprint)

- [x] Validate JSONL batch `version` and reject unsupported major versions (require `2.x`)
- [x] Validate JSONL `text.direction` values (`ltr|rtl|ttb|btt`) and fail fast on unknown values
- [x] Validate `typf render` OpenType feature tags as 4 printable ASCII characters with regression tests

## Completed (2026-02-11 quality hygiene sprint)

- [x] Validate JSONL OpenType feature tags against printable ASCII byte range (`0x20..0x7E`)
- [x] Add high-cardinality JSONL parallel ordering regression test (`process_jobs` deterministic order)
- [x] Assert `DeprecationWarning` for all Python `render_simple` test calls to keep test output warning-clean

## Completed (2026-02-11 JSONL quality sprint)

- [x] Parse `text.features` in JSONL jobs and feed validated values into `ShapingParams.features`
- [x] Make JSONL `JobSpec` accept `version` plus legacy `_version` compatibility alias
- [x] Parallelize JSONL batch job execution with deterministic output ordering and regression tests
- [x] Remove redundant index/sort pass from parallel JSONL collection (preserve order via indexed `par_iter` + `collect`)

## Completed (2026-02-11 maintenance sprint)

- [x] Add repo-root `./test.sh` wrapper as the canonical verification entrypoint
- [x] Update `scripts/test.sh` to use `cargo fmt --check` instead of `cargo fmt --all --check`
- [x] Align `.github/workflows/ci.yml` lint formatting check with local command (`cargo fmt --check`)

## Completed (v5.0.2)

<details>
<summary>Click to expand completed tasks</summary>

### Project Structure
- [x] Keep `TASKS.md` as an index (TLDR + links) and keep `PLANSTEPS/` authoritative
- [x] Add/maintain a single flat backlog here; avoid nested lists
- [x] Add `DEPENDENCIES.md` (major dependencies + rationale)
- [x] Align Cargo workspace versioning with git tags/docs (v5.0.1)

### Baseline Standardization
- [x] Inventory baseline math for Opixa/Skia/Zeno/Vello(-cpu) and document deltas vs CoreGraphics
- [x] Decide on one baseline contract (font metrics vs per-glyph bounds) and write it down
- [x] Implement the chosen contract consistently across renderers (or document why not)
- [x] Add regression tests that compare baseline placement across at least 2 renderers

### Vello-GPU Color Fonts
- [x] Confirm current behavior (blank output) and ensure CLI/docs steer users to `vello-cpu`
- [x] Add a clear runtime warning/error when `vello` is selected with bitmap/COLR glyphs
- [x] Track upstream status (issue link + minimal reproduction)

### Glyph Source & Color Fonts
- [x] Confirm `GlyphSource` covers Glyf, Cff, Cff2, Colr0, Colr1, Svg, Sbix, Cbdt, Ebdt
- [x] Ensure `typf-render-color` tries sources in `GlyphSourcePreference` priority order
- [x] Bitmap availability checks don't depend on outline presence
- [x] Centralize and harden bitmap decoding in `typf-render-color/src/bitmap.rs`
- [x] SVG renderer: opt-in bitmap embedding for color glyphs
- [x] SVG renderer: placeholder fallback when bitmap embedding disabled
- [x] Skia/Zeno: delegate complex glyph composition to `typf-render-color`
- [x] Color fixtures: expand regression coverage for COLR/SVG/sbix/CBDT

### Stage 4 (Shaping) Contract
- [x] Define a stable shaped-glyph output contract for zero-copy consumers
- [x] Define a C-ABI-safe glyph struct (repr(C), alignment, no padding surprises)
- [x] Add a "decoupled glyph iterator" API for layout engines

### Stage 5 (Rendering) Contract
- [x] Define an optional geometry output path (mesh/path ops) for GPU pipelines
- [x] Define a minimal path-op iterator API (PathOp enum + GlyphPath + GeometryData)
- [x] Define RenderMesh/vertex ABI for zero-copy GPU upload (Vertex2D, VertexUV, VertexColor, GlyphMesh, RenderMesh)

### API Extensions
- [x] Font bytes access: `FontRef::data_shared()` for zero-copy downstream access
- [x] Font metadata access: `FontRef::metrics()` for ascent/descent/units_per_em

### Python FFI
- [x] Expose vector path primitives (PathOp + GlyphPath types)
- [x] Expose font metrics/variations metadata (FontInfo enhancements)
- [x] Expose zero-copy shaped-glyph view for Pycairo-style consumers (ShapedGlyphs class with for_cairo(), iteration, indexing)

### Rust Integration
- [x] Validate typf ↔ cosmic-text/parley integration patterns
- [x] Add example showing typf as rasterization backend (external_layout_integration.rs)

### SDF Decision
- [x] Decide whether SDF is in-scope → OUT OF SCOPE for v5.x
- [-] Implement outline→SDF generation (SKIPPED)

### Platform Support
- [x] Evaluate WASM/WebGPU constraints and document in src_docs/21-webassembly-integration.md

### WGPU Integration
- [x] Prototype zero-copy mesh upload path with types + example (wgpu_mesh_upload.rs)

### Platform Support
- [x] Define vello-gpu test matrix for Linux/Windows (src_docs/26-vello-gpu-platform-support.md)

### Verification
- [x] Run full workspace tests + clippy and record results in `WORK.md`

</details>

## SDF Scoping Decision (2025-12-16)

**Decision**: SDF is OUT OF SCOPE for typf v5.x

**Rationale**:
1. SDF is primarily a GPU optimization technique; typf focuses on shaping + rasterization
2. Existing Rust solutions (kaku, easy-signed-distance-field) already serve this niche
3. SDF generation is better done offline (msdfgen CLI) for production use
4. Vello GPU renderer already provides high-quality GPU text without SDF
5. Adding SDF would introduce significant complexity for a narrow use case

**Recommendation**: Use msdfgen offline + shader-side MSDF sampling, or kaku for wgpu
