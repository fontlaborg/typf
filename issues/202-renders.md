# Renders 251119-1400

Results of ./build.sh execution #251119-1400 stored in ./typf/typf-tester/output/ 


- [ ] render-coretext-coregraphics-arab.png  render-coretext-coregraphics-latn.png PERFECT

- [ ] render-coretext-coregraphics-mixd.png Should be notdefs and not random glyphs as fallback

- [ ] render-coretext-orge-mixd.png render-coretext-orge-latn.png render-coretext-orge-arab.png render shifted downwards (too much space on top, cropped at bottom), and "dirt artefacts" due to bugs in our custom software rasterization

- [ ] render-coretext-skia-mixd.png render-coretext-skia-latn.png render-coretext-skia-arab.png render shifted downwards (too much space on top, cropped at bottom), otherwise very good

- [ ] render-coretext-zeno-mixd.png render-coretext-zeno-latn.png render-coretext-zeno-arab.png all pixels squashed vertically to one line, as if the Y coordinate is always =0

- [ ] render-harfbuzz-coregraphics-mixd.png render-harfbuzz-coregraphics-latn.png PERFECT

- [ ] render-harfbuzz-coregraphics-arab.png PERFECT and we have notdefs here as it should be

The remaining PNGs have analogical problems (coretext shaping produces "random glyph IDs" but is otherwise fine, orge is vertically shifted and "dirty", is vertically shifted down but rendering is otherwise perfect, zero is vertically collapsed / useleess while in previous iterations zeno was rendering OK just all glyphs were inverted, white ink on black filled bounding box, so in principle zeno should be fixable)