Make an ambitious rewrite plan as follows: 

## 1. Backends

From the Python CLI and Python bindings and Rust CLI all the way to the Rust lib, we want to be able to specify two kinds of backends: 

### 1.1. backend_shaping

- 'mac' = the coretext implementation on macOS; if both shaping and render is 'mac', then we do a single optimized call to get the final render, otherwise we get the shaping results

- 'win' = the directwrite implementation on Windows, if both shaping and render is 'win', then we do a single optimized call to get the final render, otherwise we get the shaping results, 

- 'icu-hb' = the hb_icu implementation, 

- 'hb' = just HarfBuzz, without ICU preprocessing, 

- 'none' = very stupid simple shaping that's only using left-to-right horizontal glyph width advancement

- 'auto' = 'mac' on macOS, 'win' on Windows, 'icu-hb' elsewhere

### 1.2. backend_render

- 'json' = output shaping results as hb-shape compatible json, no rendering

- 'mac' = the coretext implementation on macOS; if both shaping and render is 'mac', then we do a single optimized call; otherwise we use the call the coretext rasterizer on single glyphs and composite the result using the shaping results ourselves

- 'win' = the directwrite implementation on Windows; if both shaping and render is 'win', then we do a single optimized call; otherwise we use the call the directwrite/cleartype rasterizer on single glyphs and composite the result using the shaping results ourselves

- 'orge' = our own orge rasterizer (bitmap only)

- 'skia' = tiny-skia with bitmap output

- 'skia-svg' = tiny-skia with svg output

- 'zeno' = zeno with bitmap output

- 'zeno-svg' = zeno with svg output

## 2. API / architecture

We don’t provide any backwards compatibility. We completely remodel the API so that it, in principle, separates the processing into five stages: 

1. Parsing user requirements: analyzing the provided text, font specification, formatting parameters such as font-size, letter-spacing, lang, font-variation-settings, font-feature-settings, font-optical-sizing, foreground color with transparency, background color with transparency, and other potential input params. Also parsing backend requirements and output format requirements. 

2. Unicode preprocessing: normalization, itemization into runs per directionality and writing language system (script). This is font-agnostic. 

3. Font selection: loading the font, selecting the face (TTC / font-variation-settings instance), checking its metadata. 

4. Shaping: for each run, font spec, size etc., initial resolution of Unicode codepoints to default glyph IDs, and script-specific OpenType shaping, applying font-feature-settings, and arriving at a shaping result

5. Rendering: producing the vector- or bitmap representation of each final rendered run composed together into a line, or the JSON shaping result representation

6. Export: generating the final requested format, depending on rendering result one of SVG, PNG, PNM (PBM/PGM/PPM), JPG, JSON, other

## 3. Ability to build selectively

We want to be able to build various combos of backend_shaping + backend_render + format support. 

For example we may want to only build backend_shaping=none + backend+render=orge + format=pnm for a minimal-dependency build. 

## 4. Dependencies

We want to use read-fonts + skrifa for font loading, never ttf-parser. 

## 5. Convenience API compatibility for major established Rust solutions used for text layout.

Research and document what these might be. 

## 6. Rust CLI

Make a rich Rust CLI app

## 7. Python API + CLI

Make a Python binding and Fire CLI app

## 8. Principles

- If something isn't supported, the lib should raise a 'not implemented', not silently provide fallbacks
- Focus on extreme performance
- Make the code benchmarkable
- Optimize the architecture
- Consider pre-existing PLAN.md and TODO.md accordingly

Output a unified, extensive, detailed, tiered, specific, example-illustrated PLAN.  
