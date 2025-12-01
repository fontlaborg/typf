# Typf Roadmap

## Quality Sprint (Dec 2025) — COMPLETE

All P0-P3 milestones achieved. The six-stage pipeline, bidi correctness, and export/render paths are now reliable for release.

### Completed Milestones

- **M1** (P0): All correctness blockers fixed with tests; CLI uses Pipeline; no stub font fallback; bidi/PNG regressions covered.
- **M2** (P1): Renderer sizing, cache wiring, fontdb hygiene, 32-bit glyph propagation complete with goldens.
- **M3** (P2-P3): Export/CLI/Python parity tests and CI matrix running; docs updated to match reality.

---

## Phase 5 — Vector Output Enhancements (Future)

Optional enhancements for advanced SVG output:

- Implement Skia renderer SVG output mode — DONE (Dec 1, 2025)
- Implement Zeno renderer SVG output mode — DONE (Dec 1, 2025)
- Preserve gradients and embed bitmaps for SVG output when needed — DONE (Dec 1, 2025)
- CPAL color palette support in SVG (requires ColorPainter implementation) — DONE (Dec 1, 2025)

---

## Execution Checklist (per change set)

- Write failing test first, then minimal fix.
- Run `cargo fmt && cargo clippy -D warnings && cargo test --workspace` (or targeted subsets when platform-gated).
- Record findings and test results in WORK.md during work, then clear it at end.
- Update TODO.md checkboxes to match progress; keep NEXTTASK.md untouched.
