
# Renders 251119-0800

Results of ./build.sh execution #251119-0800 stored in ./typf/typf-tester/output/ 

## PNG

- [ ] render-coretext-coregraphics-arab.png PERFECT
- [ ] render-coretext-coregraphics-latn.png PERFECT
- [ ] render-coretext-coregraphics-mixd.png Decent. The Arabic and Chinese glyphs don’t exist in the font, so there is some font fallback happening, but then the font fallback probably referenced (and used shaping of) another font, but then the rasterized filled it the glyphs from the current font, so it looks kind of random
- [ ] render-coretext-orge-arab.png the shaping is fine, rendering: the result is moved too much up (top is cropped, bottom is a white area), and our custom orge software rasterizer is buggy: generally you can make out the letterforms just fine, but the inner whitespaces ("counters") are wrongly filled with a pattern of thin horizontal lines
- [ ] render-coretext-orge-latn.png same orge bug
- [ ] render-coretext-orge-mixd.png same orge bug
- [ ] render-coretext-skia-arab.png the Arabic glyphs are all isolated forms (no Arabic joining, as if the shaping results are incorrectly interpreted), the text is upside down and it’s cropped at one edge (at the top edge of the text but bottom edge of the image because it’s upside down) and has too much margin on the opposite side 
- [ ] render-coretext-skia-latn.png also upside down and cropped
- [ ] render-coretext-skia-mixd.png same, also before the equal sign weirdly huge gap
- [ ] render-coretext-zeno-arab.png same problem as render-coretext-skia-arab.png but IN ADDITION to being upside down, each glyph is also INVERSED (has a black solid filled bounding box and then is drawn as white inside)
- [ ] render-coretext-zeno-latn.png same 
- [ ] render-coretext-zeno-mixd.png same
- [ ] render-harfbuzz-coregraphics-arab.png PERFECT
- [ ] render-harfbuzz-coregraphics-latn.png PERFECT
- [ ] render-harfbuzz-coregraphics-mixd.png GOOD better than render-coretext-coregraphics-mixd.png because we didn’t get weird fallback, we just got notdefs as expected
- [ ] render-harfbuzz-orge-arab.png same orge problems as earlier
- [ ] render-harfbuzz-orge-latn.png same orge problems as earlier, look at this closely to understand orge problems
- [ ] render-harfbuzz-orge-mixd.png same orge problems
- [ ] render-harfbuzz-skia-arab.png same skia problems as earlier (disjointed upside down letters)
- [ ] render-harfbuzz-skia-latn.png upside down & cropped
- [ ] render-harfbuzz-skia-mixd.png upside down & cropped
- [ ] render-harfbuzz-zeno-arab.png upside down & cropped & inverted bounding boxes
- [ ] render-harfbuzz-zeno-latn.png upside down & cropped & inverted bounding boxes
- [ ] render-harfbuzz-zeno-mixd.png upside down & cropped & inverted bounding boxes
- [ ] render-icu-hb-coregraphics-arab.png PERFECT
- [ ] render-icu-hb-coregraphics-latn.png PERFECT
- [ ] render-icu-hb-coregraphics-mixd.png PERFECT
- [ ] render-icu-hb-zeno-mixd.png render-icu-hb-orge-arab.png render-icu-hb-orge-latn.png render-icu-hb-orge-mixd.png render-icu-hb-skia-arab.png render-icu-hb-skia-latn.png render-icu-hb-skia-mixd.png render-icu-hb-zeno-arab.png render-icu-hb-zeno-latn.png same problems as we had earlier, analogically
- [ ] render-none-coregraphics-arab.png disjointed Arabic letters, but here it is expected because we used NONE SHAPING, and render is PERFECT

## SVG

ALL SVGs are correct. I believe that this is due to the fact that despite specifying various renderers, the SVG path is using some one and the same code path, because for example render-icu-hb-coregraphics-latn.svg and render-icu-hb-skia-latn.svg and render-icu-hb-zeno-latn.svg are literally identical. 

I don’t like that. We should attempt to produce the SVG from the vector output of each vector-capable renderer (= coretxt, skia, zeno), and orge should not produce any SVG. 


