# Typf Improvement Plan

**Current Grade:** A- (88/100)
**Target Grade:** A (92/100)

## Focus Areas

Based on the code review (REVIEW.md), improvements should focus on:
1. Security hardening (font fuzzing, input validation)
2. Windows backend completion
3. Visual regression testing
4. Cross-platform CI

---

## Phase 1: Security & Reliability

### 1.1 Font Fuzzing

**Current:** Fuzz targets exist for Unicode and pipeline, but not font parsing.

**Tasks:**
- Add fuzz target for skrifa font parsing
- Add fuzz target for read-fonts table parsing
- Add corpus of malformed font files
- Integrate with cargo-fuzz CI

### 1.2 Input Validation

**Tasks:**
- Add font size limits (prevent DoS via huge allocations)
- Add glyph count limits for rendering
- Add dimension validation with helpful error messages (already partial)
- Add timeout for font operations

### 1.3 Resource Limits

**Tasks:**
- Add memory limits for font loading
- Add timeout configuration for operations
- Document resource limit configuration

---

## Phase 2: Windows Backend Completion

### 2.1 typf-os-win Parity

**Current:** DirectWrite linra exists but needs feature parity with macOS.

**Tasks:**
- Audit feature gaps vs typf-os-mac
- Add missing DirectWrite features
- Add Windows CI testing
- Add Windows-specific documentation

### 2.2 typf-render-win (Optional)

**Tasks:**
- Evaluate Direct2D standalone renderer need
- Implement if valuable for non-linra use cases

---

## Phase 3: Testing Infrastructure

### 3.1 Visual Regression Testing

**Tasks:**
- Add image comparison library (pixelmatch or similar)
- Generate golden images for all test fonts
- Add visual diff CI step
- Add script rendering tests (Arabic, CJK, Devanagari)

### 3.2 Cross-Platform CI

**Tasks:**
- Add Windows CI runner (GitHub Actions)
- Add Linux CI runner
- Add macOS CI runner (already exists?)
- Test all backends on each platform

### 3.3 Performance Regression

**Tasks:**
- Add criterion benchmark baselines
- Add CI step to detect performance regressions
- Document performance expectations

---

## Phase 4: Developer Experience

### 4.1 Documentation Improvements

**Tasks:**
- Add API stability markers (stable/experimental)
- Add backend development guide
- Add troubleshooting FAQ
- Add migration guide template

### 4.2 Feature Flag Simplification

**Tasks:**
- Audit feature flag dependencies
- Remove unused feature combinations
- Document recommended feature sets
- Consider feature flag presets

### 4.3 Observability

**Tasks:**
- Add optional tracing integration
- Add structured logging option
- Add metrics export option

---

## Non-Goals

The following are explicitly out of scope:
- Async support (sync API is sufficient for current use cases)
- ML-based cache prediction (over-engineering)
- Plugin marketplace (not needed)

---

## Success Criteria

| Metric | Current | Target |
|--------|---------|--------|
| Security Score | 80/100 | 88/100 |
| Windows Backend | B (82) | A- (90) |
| Testing Score | 85/100 | 90/100 |
| Overall Grade | A- (88) | A (92) |

---

## Implementation Notes

- Prioritize security items first
- Windows work can proceed in parallel
- Visual regression can wait for security
- Documentation improvements are ongoing

All tasks should follow the project's existing patterns:
- Use `thiserror` for new error types
- Add tests for new functionality
- Update relevant documentation
- Run `cargo fmt` and `cargo clippy`
