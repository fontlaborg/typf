- [ ] Into README describe the 'feature mix' of shapers and renderers in terms of: 
  - [ ] font extensions support 
    - [ ] OpenType Layout features
    - [ ] OpenType Variations 
  - [ ] output formats
    - [ ] SVG export
    - [ ] PNG/PBM/PGM/PPM export
    - [ ] JSON export 
  - [ ] glyph flavors and color
    - [ ] TrueType outlines ('glyf'+'gvar')
    - [ ] static PostScript outlines ('CFF ')
    - [ ] variable PostScript outlines ('CFF2')
    - [ ] layered color glyphs ('COLR' v0 v1)
    - [ ] SVG glyphs ('SVG ')
    - [ ] bitmap glyphs ('EBDT'+'EBSC')
    - [ ] Google color bitmap glyphs ('CBDT'+'CBSC')
    - [ ] Apple color bitmap glyphs ('sbix')
- [ ] Perform extensive research on the feasibility of adding support for SVG glyphs, COLR v0 and COLR v1.
    - [ ] As much as possible, utilize existing Rust libraries like resvg for SVG and some other library for COLR v0 and COLR v1.
- [ ] Into PLAN write an extensive plan to improve the feature mix in two aspects: 
    - [ ] have at least the skia and zeno renderers being able to output SVG in addition to bitmaps
    - [ ] have at least the skia renderer to be able to render SVG glyphs
    - [ ] have at least the skia renderer to be able to render Google color bitmap glyphs and Apple color bitmap glyphs
    - [ ] add support for COLR v0 and COLR v1 (this is a biggie)



Ongoing:
- [ ] For dealing with python use `uv`, `uv add`, `uv pip`, `uv run python -m`
