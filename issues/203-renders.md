# Renders 251119-1400

Results of ./build.sh execution #251119-1430 stored in ./typf/typf-tester/output/ 


- [x] render-coretext-coregraphics-arab.png  render-coretext-coregraphics-latn.png PERFECT

- [ ] render-coretext-coregraphics-mixd.png Should be notdefs and not random glyphs as fallback

- [ ] render-coretext-orge-mixd.png render-coretext-orge-latn.png render-coretext-orge-arab.png render shifted downwards (too much space on top, cropped at bottom), and STILL "dirt artefacts" due to bugs in our custom software rasterization. Compare render-coretext-coregraphics-latn.png (good) with render-coretext-orge-latn.png (dirt)

- [ ] render-coretext-skia-mixd.png render-coretext-skia-latn.png render-coretext-skia-arab.png render shifted downwards (STILL too much space on top, cropped at bottom), otherwise very good

- [ ] render-coretext-zeno-mixd.png render-coretext-zeno-latn.png render-coretext-zeno-arab.png render shifted downwards (too much space on top, cropped at bottom), AND each glyph is "inverted": the glyph's bounding box is drawn as a black rectangle and inside it the glyph is rasterized as "white pixels". It’s a clear case of inversion. 

- [ ] render-harfbuzz-coregraphics-mixd.png render-harfbuzz-coregraphics-latn.png PERFECT

- [ ] render-harfbuzz-coregraphics-arab.png PERFECT and we have notdefs here as it should be

The remaining PNGs have analogical problems (coretext shaping produces "random glyph IDs" but is otherwise fine, orge is vertically shifted and "dirty", is vertically shifted down but rendering is otherwise perfect, zeno is shifted and all glyphs are inverted, white ink on black filled bounding box)
