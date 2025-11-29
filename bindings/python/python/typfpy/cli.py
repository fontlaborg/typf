"""
TYPF Command Line Interface - Beautiful text from your terminal

Linra CLI using Click for consistent interface with Rust CLI.
"""

import sys
from pathlib import Path
from typing import Optional

import click

# Our Rust-Python bridge must be available
try:
    from typfpy import Typf, __version__, export_image, render_simple
    # Try to import linra support (may not be available on all platforms)
    try:
        from typfpy import TypfLinra, __linra_available__
        LINRA_AVAILABLE = __linra_available__
    except ImportError:
        TypfLinra = None
        LINRA_AVAILABLE = False
except ImportError:
    print("Error: TYPF extension not built. Run: maturin develop", file=sys.stderr)
    sys.exit(1)


def detect_available_shapers():
    """Detect which shaping backends are actually available"""
    shapers = []

    # Always available
    shapers.append(("none", "No shaping (direct character mapping)"))

    # Try to create each backend to see if it's available
    test_backends = [
        ("hb", "harfbuzz", "HarfBuzz (Unicode-aware text shaping)"),
        ("icu-hb", "icu-hb", "ICU + HarfBuzz (advanced Unicode + shaping)"),
        ("mac", "mac", "CoreText (macOS native)"),
    ]

    for shaper_id, shaper_name, description in test_backends:
        try:
            Typf(shaper=shaper_name, renderer="opixa")
            shapers.append((shaper_id, description))
        except ValueError:
            # Backend not available
            pass

    return shapers


def detect_available_renderers():
    """Detect which rendering backends are actually available"""
    renderers = []

    # Always available
    renderers.append(("opixa", "Opixa (pure Rust, monochrome/grayscale)"))

    # Try to create each backend to see if it's available
    test_backends = [
        ("json", "json", "JSON (structured glyph data)"),
        ("cg", "coregraphics", "CoreGraphics (macOS native)"),
        ("mac", "mac", "CoreGraphics (macOS native, alias)"),
        ("skia", "skia", "TinySkia (cross-platform, antialiased)"),
        ("zeno", "zeno", "Zeno (cross-platform vector rasterizer)"),
    ]

    for renderer_id, renderer_name, description in test_backends:
        try:
            Typf(shaper="none", renderer=renderer_name)
            renderers.append((renderer_id, description))
        except ValueError:
            # Backend not available
            pass

    return renderers


def detect_available_linra_renderers():
    """Detect which linra (single-pass) backends are available"""
    renderers = []

    if not LINRA_AVAILABLE or TypfLinra is None:
        return renderers

    # Try to create each linra backend
    test_backends = [
        ("linra-mac", "mac", "CoreText CTLineDraw (macOS, optimal performance)"),
        ("linra-win", "win", "DirectWrite DrawTextLayout (Windows, optimal performance)"),
    ]

    for renderer_id, renderer_name, description in test_backends:
        try:
            TypfLinra(renderer=renderer_name)
            renderers.append((renderer_id, description))
        except ValueError:
            # Backend not available
            pass

    return renderers


def is_linra_renderer(renderer_name: str) -> bool:
    """Check if the renderer name is a linra renderer"""
    return renderer_name in ("linra", "linra-mac", "linra-win", "linra-os")


@click.group()
@click.version_option(version=__version__, prog_name="typfpy")
def cli():
    """TYPF - Professional text rendering from the command line"""
    pass


@cli.command(name="info")
@click.option("--shapers", is_flag=True, help="List available shaping backends")
@click.option("--renderers", is_flag=True, help="List available rendering backends")
@click.option("--formats", is_flag=True, help="List available output formats")
def info(shapers: bool, renderers: bool, formats: bool):
    """Display information about available backends and formats"""

    # If no specific flags, show all info
    show_all = not (shapers or renderers or formats)

    click.echo(f"TYPF v{__version__}")
    click.echo()

    if show_all or shapers:
        click.echo("Shapers:")
        available_shapers = detect_available_shapers()
        for shaper_id, description in available_shapers:
            click.echo(f"  {shaper_id:18s} - {description}")
        if show_all:
            click.echo()

    if show_all or renderers:
        click.echo("Renderers (traditional - separate shaping step):")
        available_renderers = detect_available_renderers()
        for renderer_id, description in available_renderers:
            click.echo(f"  {renderer_id:18s} - {description}")

        click.echo()
        click.echo("Linra Renderers (single-pass shaping+rendering):")
        linra_renderers = detect_available_linra_renderers()
        if linra_renderers:
            for renderer_id, description in linra_renderers:
                click.echo(f"  {renderer_id:18s} - {description}")
        else:
            click.echo("  (none available)")
        if show_all:
            click.echo()

    if show_all or formats:
        click.echo("Output Formats:")
        click.echo("  pbm               - Portable Bitmap (monochrome, no antialiasing)")
        click.echo("  png1              - PNG monochrome (1-bit)")
        click.echo("  pgm               - Portable Graymap (8-bit grayscale)")
        click.echo("  png4              - PNG grayscale (4-bit)")
        click.echo("  png8              - PNG grayscale (8-bit)")
        click.echo("  png               - PNG RGBA (full color with alpha)")
        click.echo("  svg               - Scalable Vector Graphics")
        click.echo("  ppm               - Portable Pixmap (RGB, legacy)")


@cli.command(name="render")
@click.argument("text", required=False)
@click.option("-f", "--font-file", type=click.Path(exists=True), help="Font file path (.ttf, .otf, .ttc, .otc)")
@click.option("-y", "--face-index", type=int, default=0, help="Face index for TTC/OTC collections")
@click.option("-i", "--instance", help="Named/dynamic instance spec")
@click.option("-t", "--text-arg", "text_opt", help="Input text (alternative to positional argument)")
@click.option("-T", "--text-file", type=click.Path(exists=True), help="Read input text from file")
@click.option("--shaper", default="auto", help="Shaping backend: auto, none, hb, icu-hb, mac, win (ignored for linra)")
@click.option("--renderer", default="auto", help="Rendering backend: auto, opixa, skia, zeno, mac, win, json, linra-mac, linra-win")
@click.option("-d", "--direction", default="auto", help="Text direction: auto, ltr, rtl, ttb, btt")
@click.option("-l", "--language", help="Language tag (BCP 47), e.g., en, ar, zh-Hans")
@click.option("-S", "--script", default="auto", help="Script tag (ISO 15924), e.g., Latn, Arab, Hans")
@click.option("-F", "--features", help="Font feature settings (comma or space separated)")
@click.option("-s", "--font-size", default="200", help="Font size in pixels (or 'em' for UPM)")
@click.option("-L", "--line-height", type=int, default=120, help="Line height as %% of font size")
@click.option("-W", "--width-height", default="none", help="Canvas size spec: <width>x<height>, <width>x, x<height>, or none")
@click.option("-m", "--margin", type=int, default=10, help="Margin in pixels")
@click.option("--font-optical-sizing", default="auto", help="Optical sizing: auto, none")
@click.option("-c", "--foreground", default="000000FF", help="Text color (RRGGBB or RRGGBBAA)")
@click.option("-b", "--background", default="FFFFFF00", help="Background color (RRGGBB or RRGGBBAA)")
@click.option("-p", "--color-palette", type=int, default=0, help="Font CPAL palette index")
@click.option("-o", "--output-file", type=click.Path(), help="Output file path (stdout if omitted)")
@click.option("-O", "--format", "output_format", default="png", help="Output format: pbm, png1, pgm, png4, png8, png, svg")
@click.option("-q", "--quiet", is_flag=True, help="Silent mode (no progress info)")
@click.option("--verbose", is_flag=True, help="Verbose output")
def render(
    text: Optional[str],
    font_file: Optional[str],
    face_index: int,
    instance: Optional[str],
    text_opt: Optional[str],
    text_file: Optional[str],
    shaper: str,
    renderer: str,
    direction: str,
    language: Optional[str],
    script: str,
    features: Optional[str],
    font_size: str,
    line_height: int,
    width_height: str,
    margin: int,
    font_optical_sizing: str,
    foreground: str,
    background: str,
    color_palette: int,
    output_file: Optional[str],
    output_format: str,
    quiet: bool,
    verbose: bool,
):
    """Render text to an image file"""

    try:
        # 1. Get input text
        input_text = get_input_text(text, text_opt, text_file)

        # Check if using linra renderer
        # "auto" now defaults to linra if available (unless SVG output requested)
        # Track actual renderer to use after any fallback
        actual_renderer = renderer
        if renderer == "auto":
            # Auto-select: use linra if available, but not for SVG output
            use_linra = LINRA_AVAILABLE and output_format.lower() != "svg" and font_file is not None
            if not use_linra:
                actual_renderer = "opixa"  # Default traditional renderer
        else:
            use_linra = is_linra_renderer(renderer)
            # SVG export extracts glyph outlines from font after shaping.
            # Linra combines shaping+rendering atomically, so we can't get shaping result.
            if use_linra and output_format.lower() == "svg":
                click.echo(
                    "⚠ SVG export needs shaping results. Falling back to HarfBuzz shaper "
                    "(linra combines shaping+rendering atomically).",
                    err=True
                )
                use_linra = False
                actual_renderer = "opixa"  # Fall back to traditional renderer

        if not quiet:
            if use_linra:
                click.echo("TYPF Python CLI (linra mode)", err=True)
                click.echo("Rendering text with single-pass pipeline...", err=True)
            else:
                click.echo("TYPF Python CLI", err=True)
                click.echo("Rendering text...", err=True)

        # 2. Parse font size
        if font_size == "em":
            size = 1000.0
        else:
            size = float(font_size)

        # 3. Parse colors
        fg_color = parse_color(foreground)
        bg_color = parse_color(background) if background else None

        # 4. Render using linra or traditional pipeline
        if use_linra:
            # Linra mode: single-pass shaping + rendering
            if not LINRA_AVAILABLE or TypfLinra is None:
                raise ValueError(
                    f"Linra renderer '{renderer}' requested but linra is not available. "
                    "Ensure the extension was built with linra support."
                )

            if not font_file:
                raise ValueError("Linra rendering requires a font file (-f/--font-file)")

            # Note: SVG is handled before entering this branch - we fall back to traditional pipeline

            # Map renderer name to linra backend name ("auto" uses platform default)
            linra_backend = "auto" if renderer == "auto" else (
                "mac" if renderer in ("linra", "linra-mac", "linra-os") else "win"
            )
            linra = TypfLinra(renderer=linra_backend)

            if verbose:
                click.echo(f"Using linra renderer: {linra.get_renderer()}", err=True)
                click.echo(f"Loading font from {font_file}", err=True)

            # Parse features if provided
            parsed_features = None
            if features:
                parsed_features = parse_features(features)

            image_data = linra.render_text(
                input_text,
                font_path=font_file,
                size=size,
                color=fg_color,
                background=bg_color,
                padding=margin,
                features=parsed_features,
                language=language,
                script=script if script != "auto" else None,
            )
        else:
            # Traditional mode: separate shaper + renderer
            typf = Typf(shaper=shaper if shaper != "auto" else "hb",
                        renderer=actual_renderer if actual_renderer != "auto" else "opixa")

            if font_file:
                if verbose:
                    click.echo(f"Loading font from {font_file}", err=True)

                image_data = typf.render_text(
                    input_text,
                    font_path=font_file,
                    size=size,
                    color=fg_color,
                    background=bg_color,
                    padding=margin,
                )
            else:
                if verbose:
                    click.echo("Using stub font (no font file provided)", err=True)

                image_data = render_simple(input_text, size=size)

        # 6. Export to requested format
        if verbose:
            click.echo(f"Exporting to {output_format} format...", err=True)

        # SVG needs special handling - it uses vector output from shaping, not bitmap
        # (linra+svg already rejected at line 256-257)
        if output_format.lower() == "svg":
            if not font_file:
                raise ValueError("SVG export requires a font file (-f/--font-file)")

            # Use render_to_svg for proper vector output
            output_data = typf.render_to_svg(
                input_text,
                font_path=font_file,
                size=size,
                color=fg_color,
                padding=margin,
            ).encode('utf-8')
        else:
            output_data = export_image(image_data, output_format)

        # 7. Write output
        if output_file:
            with open(output_file, "wb") as f:
                f.write(output_data)

            if not quiet:
                click.echo(f"✓ Successfully rendered to {output_file}", err=True)
                click.echo(f"  Format: {output_format.upper()}", err=True)
                click.echo(f"  Size: {len(output_data)} bytes", err=True)
                if use_linra:
                    click.echo("  Mode: linra (single-pass)", err=True)
        else:
            # Write to stdout
            sys.stdout.buffer.write(output_data)

            if not quiet:
                click.echo("✓ Successfully rendered to stdout", err=True)
                if use_linra:
                    click.echo("  Mode: linra (single-pass)", err=True)

    except Exception as e:
        click.echo(f"Error: {e}", err=True)
        sys.exit(1)


def get_input_text(text: Optional[str], text_opt: Optional[str], text_file: Optional[str]) -> str:
    """Get input text from various sources"""

    # Priority: text positional > --text > --text-file > stdin
    if text:
        return decode_unicode_escapes(text)

    if text_opt:
        return decode_unicode_escapes(text_opt)

    if text_file:
        with open(text_file, "r") as f:
            return f.read()

    # Read from stdin
    return sys.stdin.read()


def decode_unicode_escapes(text: str) -> str:
    """Decode Unicode escape sequences like \\uXXXX or \\u{X...}"""
    result = []
    i = 0

    while i < len(text):
        if i < len(text) - 1 and text[i] == '\\' and text[i+1] == 'u':
            i += 2  # Skip \\u

            if i < len(text) and text[i] == '{':
                # \\u{X...} format
                i += 1  # Skip {
                hex_str = ""
                while i < len(text) and text[i] != '}':
                    hex_str += text[i]
                    i += 1
                if i < len(text):
                    i += 1  # Skip }

                try:
                    code = int(hex_str, 16)
                    result.append(chr(code))
                    continue
                except ValueError:
                    pass

            else:
                # \\uXXXX format (exactly 4 hex digits)
                hex_str = text[i:i+4]
                if len(hex_str) == 4:
                    try:
                        code = int(hex_str, 16)
                        result.append(chr(code))
                        i += 4
                        continue
                    except ValueError:
                        pass

        result.append(text[i])
        i += 1

    return ''.join(result)


def parse_color(color_str: str) -> tuple:
    """Parse color in RRGGBB or RRGGBBAA format"""
    hex_str = color_str.lstrip('#')

    if len(hex_str) == 6:
        # RRGGBB format
        r = int(hex_str[0:2], 16)
        g = int(hex_str[2:4], 16)
        b = int(hex_str[4:6], 16)
        return (r, g, b, 255)
    elif len(hex_str) == 8:
        # RRGGBBAA format
        r = int(hex_str[0:2], 16)
        g = int(hex_str[2:4], 16)
        b = int(hex_str[4:6], 16)
        a = int(hex_str[6:8], 16)
        return (r, g, b, a)
    else:
        raise ValueError(f"Invalid color format: {color_str}. Must be RRGGBB or RRGGBBAA")


def parse_features(features_str: str) -> list:
    """Parse OpenType feature settings"""
    result = []
    for part in features_str.replace(',', ' ').split():
        part = part.strip()
        if not part:
            continue

        if part.startswith('+'):
            result.append((part[1:], 1))
        elif part.startswith('-'):
            result.append((part[1:], 0))
        elif '=' in part:
            tag, val = part.split('=', 1)
            result.append((tag, int(val)))
        else:
            result.append((part, 1))

    return result


def main():
    """Main CLI entry point"""
    try:
        cli()
    except KeyboardInterrupt:
        click.echo("\nInterrupted", err=True)
        sys.exit(130)
    except Exception as e:
        click.echo(f"Error: {e}", err=True)
        sys.exit(1)


if __name__ == "__main__":
    main()
