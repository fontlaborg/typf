use skrifa::color::ColorPalettes;
use skrifa::FontRef;
use typf_render_color::svg::render_svg_glyph_with_palette;

fn main() {
    let font_data = std::fs::read("test-fonts/Nabla-Regular-SVG.ttf").unwrap();
    let font = FontRef::new(&font_data).unwrap();

    // Get palette colors
    let palettes = ColorPalettes::new(&font);
    let palette_colors: Vec<_> = palettes
        .get(0)
        .map(|p| p.colors().to_vec())
        .unwrap_or_default();

    println!("Rendering with {} palette colors", palette_colors.len());

    // Render with palette - no flip
    let pixmap = render_svg_glyph_with_palette(&font_data, 2, 256, 256, &palette_colors).unwrap();
    pixmap.save_png("/tmp/svg_no_flip.png").unwrap();
    println!("Saved /tmp/svg_no_flip.png");

    // Now flip it (simulating what Skia does)
    let mut rgba = pixmap.data().to_vec();
    let w = pixmap.width();
    let h = pixmap.height();
    for y in 0..(h / 2) {
        let top = (y * w * 4) as usize;
        let bot = ((h - 1 - y) * w * 4) as usize;
        for x in 0..(w * 4) as usize {
            rgba.swap(top + x, bot + x);
        }
    }

    // Create new pixmap from flipped data
    let flipped =
        tiny_skia::Pixmap::from_vec(rgba, tiny_skia::IntSize::from_wh(w, h).unwrap()).unwrap();
    flipped.save_png("/tmp/svg_flipped.png").unwrap();
    println!("Saved /tmp/svg_flipped.png");

    // Also check colors
    let non_black = pixmap
        .data()
        .chunks(4)
        .filter(|p| p[3] > 0 && (p[0] > 10 || p[1] > 10 || p[2] > 10))
        .count();
    println!("Non-black pixels: {}", non_black);
}
