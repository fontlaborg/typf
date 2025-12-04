use typf_render_color::svg::{get_svg_document, render_svg_glyph_with_palette_and_ppem};
use skrifa::raw::TableProvider;

/// Extracts just the glyph content - mirrors the internal extract_glyph_svg function
fn extract_glyph_svg_debug(svg_document: &str, glyph_id: u32) -> Option<String> {
    let glyph_pattern = format!(r#"id="glyph{}""#, glyph_id);
    if !svg_document.contains(&glyph_pattern) {
        return Some(svg_document.to_string());
    }

    // Extract defs
    let mut defs_content = String::new();
    let mut search_from = 0;
    while let Some(start) = svg_document[search_from..].find("<defs") {
        let abs_start = search_from + start;
        if let Some(end) = svg_document[abs_start..].find("</defs>") {
            let abs_end = abs_start + end + 7;
            defs_content.push_str(&svg_document[abs_start..abs_end]);
            search_from = abs_end;
        } else { break; }
    }

    // Find glyph group
    let start_tag = format!(r#"<g id="glyph{}""#, glyph_id);
    let start_pos = svg_document.find(&start_tag)?;
    let tag_end = start_pos + svg_document[start_pos..].find('>')? + 1;

    // Extract content up to closing </g>
    let after_tag = &svg_document[tag_end..];
    let mut depth = 1;
    let mut pos = 0;
    let bytes = after_tag.as_bytes();
    while pos < bytes.len() && depth > 0 {
        if pos + 2 < bytes.len() && &bytes[pos..pos+2] == b"<g" {
            if pos + 3 >= bytes.len() || bytes[pos+2] == b' ' || bytes[pos+2] == b'>' {
                depth += 1;
            }
        } else if pos + 3 < bytes.len() && &bytes[pos..pos+4] == b"</g>" {
            depth -= 1;
            if depth == 0 {
                let content = &after_tag[..pos];
                return Some(format!(r#"<svg xmlns="http://www.w3.org/2000/svg">{}{}</svg>"#, defs_content, content));
            }
        }
        pos += 1;
    }
    None
}

fn main() {
    let font_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-fonts/Nabla-Regular-SVG.ttf");
    let font_data = std::fs::read(font_path).expect("Failed to read font");
    let font = skrifa::FontRef::new(&font_data).expect("Failed to parse font");

    let upem = font.head().map(|h| h.units_per_em()).unwrap_or(1000);
    println!("Font UPM: {}", upem);

    // Test a few glyphs
    for gid in [2, 3, 36, 37] {
        println!("\n=== Glyph {} ===", gid);

        match get_svg_document(&font_data, gid) {
            Ok(svg) => {
                println!("Full SVG length: {} bytes", svg.len());

                // Check for xlink namespace
                let has_xlink_decl = svg.contains("xmlns:xlink");
                let xlink_usage_count = svg.matches("xlink:").count();
                println!("Has xmlns:xlink declaration: {}", has_xlink_decl);
                println!("Uses xlink: prefix: {} times", xlink_usage_count);

                // Print first 500 chars of original
                println!("Original SVG first 500 chars:");
                println!("{}", &svg[..svg.len().min(500)]);

                // Check for viewBox
                if let Some(start) = svg.find("viewBox=") {
                    let end = svg[start..].find('"').unwrap_or(0)
                        + svg[start+8..].find('"').unwrap_or(0) + 10;
                    println!("Original viewBox: {}", &svg[start..start+end.min(100)]);
                } else {
                    println!("Original viewBox: NONE");
                }

                // Check the glyph group's attributes (transform)
                let glyph_tag = format!(r#"<g id="glyph{}""#, gid);
                if let Some(start) = svg.find(&glyph_tag) {
                    let tag_end = start + svg[start..].find('>').unwrap_or(100) + 1;
                    let group_preview_end = (tag_end + 300).min(svg.len());
                    println!("Glyph group: {}", &svg[start..group_preview_end]);
                }

                // Parse full document
                let options = usvg::Options::default();
                match usvg::Tree::from_str(&svg, &options) {
                    Ok(tree) => {
                        println!("Full tree size: {}x{}", tree.size().width(), tree.size().height());
                    }
                    Err(e) => println!("Full parse error: {}", e),
                }

                // Now extract just this glyph
                if let Some(glyph_svg) = extract_glyph_svg_debug(&svg, gid) {
                    println!("Extracted glyph SVG length: {} bytes", glyph_svg.len());
                    println!("First 500 chars: {}", &glyph_svg[..glyph_svg.len().min(500)]);

                    // Find the glyph content after defs (look for </defs>)
                    if let Some(defs_end) = glyph_svg.find("</defs>") {
                        let content_start = defs_end + 7;
                        let content_preview = &glyph_svg[content_start..glyph_svg.len().min(content_start + 500)];
                        println!("Glyph content after defs: {}", content_preview);
                    }

                    match usvg::Tree::from_str(&glyph_svg, &options) {
                        Ok(tree) => {
                            println!("Extracted tree size: {}x{}", tree.size().width(), tree.size().height());

                            // Calculate expected output at ppem=48
                            let ppem = 48.0;
                            let scale = ppem / upem as f32;
                            println!("Scale at ppem={}: {}", ppem, scale);
                            println!("Expected from extracted: {}x{}",
                                (tree.size().width() * scale).ceil(),
                                (tree.size().height() * scale).ceil());
                        }
                        Err(e) => println!("Extracted parse error: {}", e),
                    }
                }

                // Actually render
                match render_svg_glyph_with_palette_and_ppem(&font_data, gid, 100, 100, &[], 48.0) {
                    Ok(pixmap) => {
                        println!("Final rendered pixmap: {}x{}", pixmap.width(), pixmap.height());
                    }
                    Err(e) => println!("Render error: {:?}", e),
                }
            }
            Err(e) => println!("Not found: {:?}", e),
        }
    }
}
