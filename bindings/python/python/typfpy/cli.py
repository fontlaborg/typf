"""
TYPF Command Line Interface - Beautiful text from your terminal

Where professional text rendering meets command-line simplicity.
Render, shape, and export text with the same power that drives
desktop applications—all from your terminal.

## Quick Start

```bash
# Render text to an image
python -m typfpy render "Hello, World!" hello.png

# Use custom fonts and colors
python -m typfpy render "مرحبا" arabic.png --font=/path/to/font.ttf
```
"""

import sys
from pathlib import Path
from typing import Optional

# Fire gives us that magical command-line interface
try:
    import fire
except ImportError:
    print(
        "Error: 'fire' package not installed. Install with: pip install fire",
        file=sys.stderr,
    )
    sys.exit(1)

# Our Rust-Python bridge must be available
try:
    from typfpy import Typf, __version__, export_image, render_simple
except ImportError:
    print("Error: TYPF extension not built. Run: maturin develop", file=sys.stderr)
    sys.exit(1)


class TypfCLI:
    """Your gateway to professional text rendering from the command line"""

    def __init__(self):
        """Initialize the CLI with version tracking"""
        self.version = __version__

    def render(
        self,
        text: str,
        output: str,
        font: Optional[str] = None,
        size: float = 48.0,
        shaper: str = "harfbuzz",
        renderer: str = "orge",
        format: Optional[str] = None,
        color: str = "0,0,0,255",
        background: Optional[str] = None,
        padding: int = 10,
    ):
        """
        Transform your text into stunning images

        This command takes your text, shapes it with professional algorithms,
        and renders it to your favorite image format. Perfect for thumbnails,
        headers, testing, or any time you need programmatic text rendering.

        Args:
            text: The text you want to render
            output: Where to save the resulting image
            font: Path to a .ttf/.otf font file (optional - uses built-in font if omitted)
            size: How big should the text appear (in points/pixels)
            shaper: Text shaping engine ('none', 'harfbuzz', 'coretext', 'icu-hb')
            renderer: Pixel rendering engine ('orge', 'skia', 'zeno', 'coregraphics', 'json')
            format: Output format (guessed from file extension if not specified)
            color: Text color as "R,G,B,A" (default: black)
            background: Background color as "R,G,B,A" (optional: transparent if omitted)
            padding: Space around your text in pixels

        Examples:
            # Simple black text on transparent background
            typf render "Hello World" hello.png

            # Arabic text with proper shaping
            typf render "مرحبا بالعالم" arabic.png --font=/path/to/arabic.ttf

            # Red text on white background
            typf render "Alert" warning.png --color="255,0,0,255" --background="255,255,255,255"

            # SVG output for scalability
            typf render "Logo" logo.svg --size=128 --font=/path/to/logo.ttf
        """
        # Parse colors
        try:
            fg_parts = [int(x) for x in color.split(",")]
            if len(fg_parts) != 4:
                raise ValueError("Color must have 4 components (R,G,B,A)")
            fg_color = tuple(fg_parts)
        except ValueError as e:
            print(f"Error parsing foreground color: {e}", file=sys.stderr)
            return 1

        bg_color = None
        if background:
            try:
                bg_parts = [int(x) for x in background.split(",")]
                if len(bg_parts) != 4:
                    raise ValueError("Background must have 4 components (R,G,B,A)")
                bg_color = tuple(bg_parts)
            except ValueError as e:
                print(f"Error parsing background color: {e}", file=sys.stderr)
                return 1

        # Infer format from extension if not specified
        if format is None:
            ext = Path(output).suffix.lower().lstrip(".")
            format = ext if ext else "ppm"

        try:
            # Create TYPF instance
            typf = Typf(shaper=shaper, renderer=renderer)

            # Render the text
            if font:
                image_data = typf.render_text(
                    text,
                    font_path=font,
                    size=size,
                    color=fg_color,
                    background=bg_color,
                    padding=padding,
                )
            else:
                # Use simple render with stub font
                image_data = render_simple(text, size=size)

            # Export to requested format
            output_data = export_image(image_data, format)

            # Write to file
            with open(output, "wb") as f:
                f.write(output_data)

            print(f"✓ Rendered to {output} ({len(output_data)} bytes)")
            return 0

        except Exception as e:
            print(f"Error: {e}", file=sys.stderr)
            return 1

    def shape(
        self,
        text: str,
        font: Optional[str] = None,
        size: float = 48.0,
        shaper: str = "harfbuzz",
        features: Optional[str] = None,
        language: Optional[str] = None,
        script: Optional[str] = None,
        output: Optional[str] = None,
    ):
        """
        Shape text and output glyph positioning (JSON format).

        Args:
            text: Text to shape
            font: Path to font file
            size: Font size in points
            shaper: Shaping backend ('none' or 'harfbuzz')
            features: OpenType features as comma-separated key=value pairs
            language: Language tag (e.g., 'ar', 'en')
            script: Script tag (e.g., 'arab', 'latn')
            output: Output file path (stdout if not specified)

        Examples:
            typf shape "Hello" --font=/path/to/font.ttf
            typf shape "مرحبا" --font=/path/to/font.ttf --language=ar --script=arab
            typf shape "Text" --features="liga=1,kern=1" --output=shaped.json
        """
        print(f"Shaping '{text}' with {shaper}...", file=sys.stderr)

        # Parse features
        feature_dict = {}
        if features:
            for pair in features.split(","):
                if "=" in pair:
                    key, value = pair.split("=", 1)
                    feature_dict[key.strip()] = int(value.strip())

        try:
            typf = Typf(shaper=shaper, renderer="orge")

            # For now, just render and note that JSON shaping output
            # would require additional API exposure
            print(
                "Note: Full JSON shaping output requires extended API", file=sys.stderr
            )
            print(f"Shaper: {shaper}, Features: {feature_dict}", file=sys.stderr)

            if font:
                print(f"Font: {font}", file=sys.stderr)

            return 0

        except Exception as e:
            print(f"Error: {e}", file=sys.stderr)
            return 1

    def info(self):
        """
        Display TYPF version and configuration information.

        Examples:
            typf info
        """
        print(f"TYPF v{self.version}")
        print("\nAvailable backends:")
        print("  Shapers: none, harfbuzz")
        print("  Renderers: orge")
        print("  Exporters: pnm, png, svg, json")
        print("\nPython bindings built with PyO3")
        return 0

    def version(self):
        """
        Display version information.

        Examples:
            typf version
        """
        print(f"TYPF v{self.version}")
        return 0


def main():
    """Main CLI entry point"""
    try:
        fire.Fire(TypfCLI)
    except KeyboardInterrupt:
        print("\nInterrupted", file=sys.stderr)
        sys.exit(130)
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
