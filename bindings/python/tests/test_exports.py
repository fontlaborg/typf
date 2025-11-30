"""
Python parity tests for PNG/SVG export.

These tests verify that the Python bindings produce valid output
matching the Rust library's behavior.
"""



class TestImports:
    """Test that all expected exports are available."""

    def test_import_typf(self):
        """Can import the main Typf class."""
        from typfpy import Typf

        assert Typf is not None

    def test_import_render_simple(self):
        """Can import render_simple function."""
        from typfpy import render_simple

        assert callable(render_simple)

    def test_import_export_image(self):
        """Can import export_image function."""
        from typfpy import export_image

        assert callable(export_image)

    def test_import_font_info(self):
        """Can import FontInfo class."""
        from typfpy import FontInfo

        assert FontInfo is not None

    def test_version_available(self):
        """Version string is available."""
        from typfpy import __version__

        assert isinstance(__version__, str)
        assert len(__version__) > 0


class TestRenderSimple:
    """Test render_simple function (no font file required)."""

    def test_render_simple_basic(self):
        """Can render simple text without a font file."""
        from typfpy import render_simple

        result = render_simple("Hello")
        assert result is not None
        assert "width" in result
        assert "height" in result
        assert "data" in result
        assert result["width"] > 0
        assert result["height"] > 0

    def test_render_simple_with_size(self):
        """Can render with custom size."""
        from typfpy import render_simple

        small = render_simple("A", size=24)
        large = render_simple("A", size=48)

        # Larger size should produce larger output
        assert large["width"] >= small["width"]
        assert large["height"] >= small["height"]

    def test_render_simple_empty_text(self):
        """Empty text produces valid (possibly empty) output."""
        from typfpy import render_simple

        result = render_simple("")
        assert result is not None


class TestExportImage:
    """Test export_image function for different formats."""

    def test_export_png(self):
        """Can export to PNG format."""
        from typfpy import export_image, render_simple

        image = render_simple("Test", size=24)
        png_data = export_image(image, format="png")

        assert png_data is not None
        assert len(png_data) > 0
        # PNG magic bytes
        assert png_data[:8] == b"\x89PNG\r\n\x1a\n"

    def test_export_svg(self):
        """Can export to SVG format."""
        from typfpy import export_image, render_simple

        image = render_simple("Test", size=24)
        svg_data = export_image(image, format="svg")

        assert svg_data is not None
        assert len(svg_data) > 0
        # SVG should be XML
        svg_str = svg_data.decode("utf-8") if isinstance(svg_data, bytes) else svg_data
        assert "<svg" in svg_str or "<?xml" in svg_str

    def test_export_ppm(self):
        """Can export to PPM format."""
        from typfpy import export_image, render_simple

        image = render_simple("Test", size=24)
        ppm_data = export_image(image, format="ppm")

        assert ppm_data is not None
        assert len(ppm_data) > 0
        # PPM magic bytes
        assert ppm_data[:2] == b"P6" or ppm_data[:2] == b"P3"


class TestTypfClass:
    """Test the main Typf class."""

    def test_create_typf_default(self):
        """Can create Typf with default settings."""
        from typfpy import Typf

        typf = Typf()
        assert typf is not None

    def test_create_typf_with_shaper(self):
        """Can create Typf with specific shaper."""
        from typfpy import Typf

        typf = Typf(shaper="harfbuzz")
        assert typf is not None

    def test_create_typf_with_renderer(self):
        """Can create Typf with specific renderer."""
        from typfpy import Typf

        typf = Typf(renderer="opixa")
        assert typf is not None


class TestTypfLinra:
    """Test TypfLinra (macOS native renderer) if available."""

    def test_linra_availability_check(self):
        """Can check if linra is available."""
        import typfpy

        # __linra_available__ may or may not be exported
        linra_available = getattr(typfpy, "__linra_available__", False)
        assert isinstance(linra_available, bool)

    def test_linra_import_when_available(self):
        """Can import TypfLinra when available."""
        import typfpy

        linra_available = getattr(typfpy, "__linra_available__", False)
        if linra_available:
            from typfpy import TypfLinra

            assert TypfLinra is not None
        else:
            # TypfLinra should be None when not available
            assert typfpy.TypfLinra is None
