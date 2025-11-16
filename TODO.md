# typf Todo List

This file lists tasks specific to the `typf` project. For the overall development plan, see the root [PLAN.md](../../PLAN.md) and [TODO.md](../../TODO.md).

---

## Active Development Tasks

See the root [TODO.md](../../TODO.md) for the comprehensive, prioritized task list. All active TYPF tasks are tracked there with the `**(typf)**` prefix.

---

## Project-Specific Notes

**Current Focus:** Performance hardening and safety improvements

**Key Areas:**
- Concurrency optimization (cache improvements)
- Rasterizer hot-path optimization (orge integration, SIMD)
- FFI safety (panic handling, unified errors)
- Testing infrastructure (visual regression, fuzzing, Miri)
- Documentation (comprehensive README, API docs)

**Completed Milestones:**
- ✅ All core backends implemented (CoreText, DirectWrite, HarfBuzz)
- ✅ Python bindings fully functional via PyO3
- ✅ CLI with batch/stream/render commands
- ✅ Multi-shard LRU caching
- ✅ Integration tests (38+ tests passing)
- ✅ SVG/PNG output with color font support

---

For detailed task tracking and dependencies, refer to the root TODO.md.
