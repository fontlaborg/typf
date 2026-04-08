# typf Code Quality Metrics Report

## 1. Codebase Size (Lines of Code)
Based on `tokei` analysis, the codebase totals **154,729 lines** across **322 files**.

### Primary Languages
| Language | Files | Lines | Code | Comments | Blanks |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Rust** | 119 | 42,169 | 34,283 | 2,015 | 5,871 |
| **Markdown** | 79 | 18,641 | 0 | 13,282 | 5,359 |
| **Python** | 19 | 5,400 | 4,259 | 285 | 856 |
| **Shell** | 18 | 2,688 | 1,864 | 450 | 374 |
| **TOML** | 39 | 1,639 | 1,372 | 99 | 168 |

*Note: The HTML files (27 files, 71,498 lines) are mostly generated docs/coverage reports.*

## 2. Safety and Security

### Unsafe Rust Usage
There are **6** instances of `unsafe` blocks in the codebase, indicating a strong adherence to safe Rust practices given the project size.
Locations:
* `core/src/ffi.rs:9`
* `backends/typf-shape-ct/src/lib.rs:33`
* `backends/typf-render-opixa/src/simd.rs:3`
* `backends/typf-os-mac/src/lib.rs:26`
* `backends/typf-render-cg/src/lib.rs:11`
* `backends/typf-os-win/src/lib.rs:6`

### Security Audit (`cargo audit` / `pip-audit`)
* **Critical Vulnerabilities:** 0
* **High Vulnerabilities:** 0
* **Low Vulnerabilities:** 1
  * Package: `pyo3` (installed: `0.22.6`, patched: `>=0.24.1`)
  * Advisory: Risk of buffer overflow in `PyString::from_object` (CVE: GHSA-pph8-gcv7-4qj5)

## 3. Code Quality Indicators

### Linter Warnings (Clippy)
* Total `clippy` compiler warnings: **0** (Excellent!)

### Technical Debt (TODOs / FIXMEs)
Total instances found: **166**
* **High Priority (`FIXME`, `HACK`, `XXX`):** 14
* **Medium Priority (`TODO`):** 152
* **Low Priority:** 0

*Many of the high-priority `HACK` and `FIXME` comments are located within external dependencies vendored in the tree (`external/parley`, `external/vello`, `external/swash`).*

### Complexity Hotspots
The following files exhibit the highest cyclomatic complexity combined with recent churn:
1. `backends/typf-render-color/src/lib.rs` (Complexity: 96, Risk Score: 6.6)
2. `backends/typf-render-opixa/src/lib.rs` (Complexity: 44, Risk Score: 5.5)
3. `core/src/pipeline.rs` (Complexity: 42, Risk Score: 5.4)
4. `backends/typf-shape-hb/src/lib.rs` (Complexity: 34, Risk Score: 5.1)
5. `fontdb/src/lib.rs` (Complexity: 19, Risk Score: 4.2)

## 4. Test Suite Health

*Note: There appears to be an issue with missing test font files (`No such file or directory (os error 2)` for `NotoSans-Regular.ttf`) causing several CLI tests to fail.*

* **Unit/Integration Tests (Rust):**
  * Baseline Consistency: 1 passed
  * Color Font Regression: 13 passed
  * Integration Tests: 14 passed
  * Visual Regression: 21 passed
  * CLI Tests: 160 passed, **16 failed** (Due to missing test fixture `NotoSans-Regular.ttf`)

* **Python Tests (`pytest`):**
  * Collection failed due to import file mismatches (likely caused by duplicate files in `worktrees/` directory conflicting with `bindings/py/tests/`).
