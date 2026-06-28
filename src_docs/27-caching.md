<!-- this_file: src_docs/27-caching.md -->
# Caching

Typf can cache the two most expensive intermediate results in the pipeline:
shaping output and rendered glyph output. Caching is **off by default** so that
one-shot renders stay predictable and allocate no extra memory.

## What is cached

| Cache | Crate type | Keyed on |
|-------|------------|----------|
| **Shaping cache** | `typf_core::shaping_cache::ShapingCache` | text + font hash + size + features + variations + language + script |
| **Glyph cache** | `typf_core::glyph_cache::GlyphCache` | shaped glyph stream + render params + font identity + renderer name |

A cache hit on the shaping cache skips the (relatively expensive) HarfBuzz/ICU
shaping step. A glyph-cache hit skips rasterization entirely and returns the
previously produced `RenderOutput`.

## The eviction strategy: Moka TinyLFU

Both caches are backed by a single [Moka](https://github.com/moka-rs/moka)
cache using the **TinyLFU** admission/eviction policy. This is a deliberate
choice over a plain LRU:

- **Scan resistance.** TinyLFU tracks access *frequency* (of both hits and
  misses), so a one-off burst of unique inputs — e.g. fingerprinting hundreds of
  candidate fonts during font matching — cannot evict the genuinely hot entries.
  A pure LRU would happily flush its useful contents on such a scan.
- **Byte-weighted capacity.** Entries are weighted by their real memory
  footprint, not counted as "1 entry each". A 4 MB colour-emoji bitmap consumes
  roughly 4000× the quota of a 1 KB monochrome glyph, which prevents a few
  pathological glyphs from blowing the memory budget.
- **Time-to-idle.** Entries untouched for 10 minutes are dropped, bounding
  memory in long-running processes.

> **A note on naming.** Some internal identifiers still say `MultiLevelCache`,
> `l1_*` and `l2_*`. These are historical: an earlier design used a two-tier
> L1/L2 layout. There is now a *single* Moka TinyLFU cache; the old names are
> kept only for API compatibility and do **not** indicate separate cache levels.

The default capacity (512 MB, see `DEFAULT_CACHE_MAX_BYTES`) can be overridden
with the `TYPF_CACHE_MAX_BYTES` environment variable.

## Enabling caching

Caching is controlled by a global switch in `typf_core::cache_config`.

```rust
use typf::cache_config;

cache_config::set_caching_enabled(true);   // turn on
assert!(cache_config::is_caching_enabled());
cache_config::set_caching_enabled(false);  // turn off
```

```python
import typf

typf.set_caching_enabled(True)
typf.is_caching_enabled()
typf.set_caching_enabled(False)
```

```bash
TYPF_CACHE=1 ./your_app   # enable at startup via environment
```

Tests should use `cache_config::scoped_caching_enabled(...)` so that enabling
the cache in one test cannot leak into another.

## When to turn it on

| Scenario | Caching | Why |
|----------|---------|-----|
| One-shot CLI renders | Off (default) | Nothing is repeated |
| Interactive UI re-rendering the same string | On | Frequent re-shaping of identical input |
| Batch over *different* texts | Off | Every input is unique — pure overhead |
| Batch over the *same* text/font set | On | Cache hits pay for themselves |
| Memory-constrained host | Off | Caches reserve memory |

## Rule of thumb

Turn caching on only when the same `(text, font, params)` tuple is genuinely
rendered more than once. For everything else, the default off setting is both
faster overall and lighter on memory.
