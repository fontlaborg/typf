<!-- this_file: PLANSTEPS/06-color-font-integration.md -->

The mandate is to standardize and complete complex font support across the relevant rendering backends, excluding `typf-render-opixa` which must remain monochrome. This effort focuses on robustly handling OpenType layered vector formats (`COLRv0`, `COLRv1`), scalable vector graphics (`SVG` table), and embedded bitmaps (`sbix`, `CBDT`/`EBDT`).

## Backends Requiring Enhanced Color Support

The following rendering backends, already architected for bitmap or vector output, require verification and potentially enhanced integration with the dedicated `typf-render-color` features to achieve parity in color font handling:

1.  **`typf-render-skia`**: Currently supports COLR/SVG/bitmap but needs robust handling, especially for complex cases like CBDT.
2.  **`typf-render-zeno`**: Similar status to Skia, relying on `typf-render-color`.
3.  **`typf-render-svg`**: As a vector-only output, it must be updated to embed or export rasterized color glyphs (SVG table, bitmaps) when pure outlines are unavailable or inappropriate.

The CoreGraphics (`typf-render-cg`) backends rely heavily on the underlying macOS platform APIs for color support, which makes internal enhancement difficult, but they serve as a critical reference for correctness and pixel matching.

## Detailed Plan for Color Font Integration

The strategy is to leverage the existing `typf-render-color` crate, which is designed as the centralized factory for complex glyph rasterization, encapsulating the logic for font features and choosing the appropriate rendering technique.

### Phase 1: Standardize Glyph Source Preference and Lookup

The core of effective color rendering is guaranteeing the correct glyph data (outline, COLRv1, bitmap, etc.) is selected based on a defined priority order. This logic must be centralized in `typf-core` and implemented within `typf-render-color`.

**1.1. Define Comprehensive Glyph Sources (in `typf-core/src/types.rs`):**

The existing `GlyphSource` enum must be comprehensively defined to include all known OpenType color flavors, aligning with font parsing libraries like `skrifa`:

*   **Action:** Ensure explicit variants exist for: `Glyf`, `Cff`, `Cff2` (outlines), `Colr1`, `Colr0` (layered vector colors), `Svg` (SVG table vector), `Sbix`, `Cbdt`, `Ebdt` (bitmap sources).

**1.2. Implement Unified Source Selection (in `typf-render-color`):**

*   **Action:** Refine the logic in `typf-render-color` to iterate through the user-provided `GlyphSourcePreference` (from `RenderParams`).
*   The system **MUST** attempt to fetch the glyph data sequentially based on priority until a valid representation is found, facilitating seamless fallback. This logic replaces direct calls to bitmap or outline parsers within the main renderer loops.

**1.3. Implement `FontRef` Accessors:**

*   **Action:** Ensure the `FontRef` trait (implemented by `typf-fontdb`) exposes necessary low-level accessors, potentially providing a unified interface to request either vector outlines (for CFF/glyf/COLR) or raw strike data (for `sbix`/`CBDT`/`EBDT`).

### Phase 2: Complete Bitmap Glyph Handling (CBDT/EBDT Fix)

The key documented failure point is the unreliable handling of bitmap-only glyph formats, specifically `CBDT`.

**2.1. Centralize Bitmap Decoding (in `typf-render-color/src/bitmap.rs`):**

*   **Action:** Implement robust decoding functions that use the raw byte slice retrieved via the `FontRef` accessor in Phase 1.
*   The logic must handle the internal image formats specified by `sbix` (typically PNG) and `CBDT`/`EBDT` (raw bitmap formats), leveraging existing dependencies like `png` and `tiny-skia` primitives.

**2.2. Address Outline Conflicts:**

*   **Rationale:** `CBDT`/`sbix` fonts often have degenerate or empty outlines (glyf table entries are often null, but the glyph ID is valid). The upstream fix noted in the `PLAN.md` removing `&& !outline_empty` for Skia/Zeno must be generalized.
*   **Action:** The source selection logic in `typf-render-color` **MUST NOT** rely on the presence of outlines when checking availability for bitmap sources (`sbix`, `CBDT`, `EBDT`). It should strictly prioritize the highest-ranked available source based on `GlyphSourcePreference`.

### Phase 3: Vector Renderer (`typf-render-svg`) Color Embedding

The SVG export backend must be enhanced to properly support color glyphs, which cannot be expressed as simple paths.

**3.1. Enable Bitmap Embedding Feature:**

*   **Action:** Ensure the `typf-export-svg` crate builds with the `bitmap-embed` feature, which relies on `typf-render-color` and `base64` to embed rasterized color glyphs as PNG images within the SVG output.
*   **Minimalism Check:** This feature should be opt-in, respecting the constraint that SVG files are sometimes required to be pure vector.

**3.2. Implement Rasterization Fallback in `typf-render-svg`:**

*   **Action:** When `typf-render-svg` receives a glyph that corresponds to a color source (COLR, SVG table, or bitmap) and the `bitmap-embed` feature is enabled:
    1.  Call the centralized color rasterizer logic (from `typf-render-color`).
    2.  Obtain the resulting `RenderOutput::Bitmap` data.
    3.  Convert the bitmap data to a base64-encoded PNG image using `typf-export-svg` utilities.
    4.  Embed the image data within an SVG `<image>` tag at the correct position and size, preserving the vector nature of the surrounding monochrome text.
*   **Error Handling:** If the bitmap embedding feature is disabled and the glyph is a color type, the SVG renderer **SHOULD** fall back to rendering the monochrome outline (if available) or render a placeholder, rather than failing.

### Phase 4: Final Integration and Verification

**4.1. Update Skia/Zeno Pipelines:**

*   **Action:** Verify that `typf-render-skia` and `typf-render-zeno` delegate all glyph outline loading and color glyph composition exclusively to `typf-render-color`. The role of `typf-render-skia`/`typf-render-zeno` should be limited to acting as the final canvas target for the resulting rasterized or vector paths/bitmaps provided by `typf-render-color`.

**4.2. Run Regression Testing:**

*   **Action:** Execute the comprehensive test suites, specifically targeting the color font fixtures (`Nabla-Regular-CBDT.ttf`, `Nabla-Regular-COLR.ttf`, etc.).
*   **Verification:** Ensure that:
    1.  Bitmap color glyphs (`sbix`/`CBDT`) render correctly in Skia/Zeno backends (Fix Phase 2).
    2.  SVG output embeds bitmap glyphs when requested (Fix Phase 3).
    3.  The performance regressions identified in Zeno are not exacerbated by the color integration.
*   **Goal:** Achieve consistency across all non-platform color-capable renderers and move CBDT support from "partially supported/failing" to "functional rasterization".
