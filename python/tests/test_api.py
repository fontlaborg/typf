# this_file: python/tests/test_api.py

"""Unit tests for typf Python API."""

import pytest
import tempfile
import os
from pathlib import Path
from unittest.mock import patch, MagicMock
import sys

PROJECT_ROOT = Path(__file__).resolve().parents[2]
PYTHON_PACKAGE = PROJECT_ROOT / "python"
if str(PYTHON_PACKAGE) not in sys.path:
    sys.path.insert(0, str(PYTHON_PACKAGE))

# Mock the native module for testing
class NativeFontMock:
    def __init__(self, family, size, weight, style, variations=None, features=None):
        self.family = family
        self.size = size
        self.weight = weight
        self.style = style
        self.variations = variations or {}
        self.features = features or {}

    @classmethod
    def from_path(cls, path, size, weight, style, variations=None, features=None):
        return cls(path, size, weight, style, variations, features)

    @classmethod
    def from_bytes(cls, name, _data, size, weight, style, variations=None, features=None):
        return cls(name, size, weight, style, variations, features)


mock_native = MagicMock()
mock_native.get_version = lambda: "0.1.0-test"
mock_native.TextRenderer = MagicMock
mock_native.Font = NativeFontMock
mock_native.ShapingResult = MagicMock
mock_native.Glyph = MagicMock

# Patch the import before importing typf
import sys
sys.modules['typf.typf'] = mock_native

import typf


class TestFont:
    """Test Font class."""

    def test_font_creation(self):
        """Test basic font creation."""
        font = typf.Font("Arial", 24)
        assert font.family == "Arial"
        assert font.size == 24
        assert font.weight == 400
        assert font.style == "normal"

    def test_font_with_options(self):
        """Test font with all options."""
        font = typf.Font(
            "Inter",
            size=18,
            weight=700,
            style="italic",
            variations={"wght": 600},
            features={"kern": True}
        )
        assert font.weight == 700
        assert font.style == "italic"
        assert font.variations == {"wght": 600}
        assert font.features == {"kern": True}

    def test_font_from_path(self):
        """Test font from file path."""
        font = typf.Font.from_path(Path("/path/to/font.ttf"), 16)
        assert "font.ttf" in font.family

    def test_font_from_bytes(self):
        """Test font from raw bytes."""
        payload = b"fake-font-data"
        font = typf.Font.from_bytes("Custom", payload, 20)
        assert font.family == "Custom"
        clone = font.with_size(22)
        assert clone.family == "Custom"

    def test_font_with_size(self):
        """Test creating font variant with different size."""
        font1 = typf.Font.from_path("/tmp/foo.ttf", 24)
        font2 = font1.with_size(36)
        assert font2.size == 36
        assert font2.family == font1.family
        assert font2.weight == font1.weight

    def test_font_with_weight(self):
        """Test creating font variant with different weight."""
        font1 = typf.Font("Arial", 24)
        font2 = font1.with_weight(700)
        assert font2.weight == 700
        assert font2.family == font1.family
        assert font2.size == font1.size

    def test_font_repr(self):
        """Test font string representation."""
        font = typf.Font("Arial", 24, weight=700)
        repr_str = repr(font)
        assert "Arial" in repr_str
        assert "24" in repr_str
        assert "700" in repr_str


class TestBitmap:
    """Test Bitmap class."""

    def test_bitmap_creation(self):
        """Test bitmap creation with raw data."""
        data = b'\x00' * (100 * 100 * 4)
        bitmap = typf.Bitmap(data, 100, 100)
        assert bitmap.width == 100
        assert bitmap.height == 100
        assert bitmap.format == "rgba"
        assert len(bitmap.data) == 100 * 100 * 4

    def test_bitmap_formats(self):
        """Test different bitmap formats."""
        data_rgba = b'\x00' * (10 * 10 * 4)
        bitmap_rgba = typf.Bitmap(data_rgba, 10, 10, "rgba")
        assert bitmap_rgba.format == "rgba"

        data_rgb = b'\x00' * (10 * 10 * 3)
        bitmap_rgb = typf.Bitmap(data_rgb, 10, 10, "rgb")
        assert bitmap_rgb.format == "rgb"

    @patch('typf.HAS_NUMPY', True)
    @patch('typf.np')
    def test_bitmap_to_numpy(self, mock_np):
        """Test converting bitmap to numpy array."""
        mock_np.frombuffer.return_value.reshape.return_value = MagicMock()

        data = b'\x00' * (10 * 10 * 4)
        bitmap = typf.Bitmap(data, 10, 10)
        arr = bitmap.to_numpy()

        mock_np.frombuffer.assert_called_once()
        assert mock_np.frombuffer.return_value.reshape.called

    @patch('typf.HAS_PIL', True)
    @patch('typf.PILImage')
    def test_bitmap_to_pil(self, mock_pil):
        """Test converting bitmap to PIL image."""
        mock_image = MagicMock()
        mock_pil.frombytes.return_value = mock_image

        data = b'\x00' * (10 * 10 * 4)
        bitmap = typf.Bitmap(data, 10, 10)
        img = bitmap.to_pil()

        mock_pil.frombytes.assert_called_once_with("RGBA", (10, 10), data)
        assert img == mock_image

    @patch('typf.HAS_PIL', False)
    def test_bitmap_save_without_pil(self):
        """Test saving bitmap without PIL (raw data)."""
        data = b'\x00\x01\x02\x03' * 100
        bitmap = typf.Bitmap(data, 20, 20)

        with tempfile.NamedTemporaryFile(delete=False) as f:
            bitmap.save(f.name)
            f.close()

            with open(f.name, 'rb') as f2:
                saved_data = f2.read()

            os.unlink(f.name)
            assert saved_data == data


class TestTextRenderer:
    """Test TextRenderer class."""

    def test_renderer_creation(self):
        """Test basic renderer creation."""
        renderer = typf.TextRenderer()
        assert renderer.cache_size == 512
        assert renderer.parallel is True
        assert renderer.timeout is None

    def test_renderer_with_backend(self):
        """Test renderer with specific backend."""
        renderer = typf.TextRenderer(backend="harfbuzz")
        assert renderer.backend == "harfbuzz"

    def test_renderer_options(self):
        """Test renderer with all options."""
        renderer = typf.TextRenderer(
            backend="coretext",
            cache_size=1024,
            parallel=False,
            timeout=5.0
        )
        assert renderer.cache_size == 1024
        assert renderer.parallel is False
        assert renderer.timeout == 5.0

    @patch('platform.system')
    def test_detect_backend(self, mock_system):
        """Test backend auto-detection."""
        # macOS
        mock_system.return_value = "Darwin"
        renderer = typf.TextRenderer()
        assert renderer.backend == "coretext"

        # Windows
        mock_system.return_value = "Windows"
        renderer = typf.TextRenderer()
        assert renderer.backend == "directwrite"

        # Linux
        mock_system.return_value = "Linux"
        renderer = typf.TextRenderer()
        assert renderer.backend == "harfbuzz"

    def test_render_basic(self):
        """Test basic text rendering."""
        renderer = typf.TextRenderer()
        mock_result = (b'\x00' * 400, 10, 10)
        renderer._renderer.render.return_value = mock_result

        result = renderer.render("Hello", typf.Font("Arial", 24))
        assert isinstance(result, typf.Bitmap)
        assert result.width == 10
        assert result.height == 10

    def test_render_formats(self):
        """Test different render formats."""
        renderer = typf.TextRenderer()

        # PNG format
        renderer._renderer.render.return_value = b'PNG DATA'
        result = renderer.render("Test", "Arial", format="png")
        assert result == b'PNG DATA'

        # SVG format
        renderer._renderer.render.return_value = '<svg>...</svg>'
        result = renderer.render("Test", "Arial", format=typf.RenderFormat.SVG)
        assert result == '<svg>...</svg>'

    def test_render_with_options(self):
        """Test rendering with various options."""
        renderer = typf.TextRenderer()
        renderer._renderer.render.return_value = (b'\x00' * 400, 10, 10)

        result = renderer.render(
            "Hello",
            typf.Font("Arial", 24),
            color="#FF0000",
            background="#FFFFFF",
            padding=10,
            direction=typf.Direction.LEFT_TO_RIGHT
        )

        # Verify options were passed
        call_args = renderer._renderer.render.call_args
        assert call_args[1]["render_options"]["color"] == "#FF0000"
        assert call_args[1]["render_options"]["background"] == "#FFFFFF"
        assert call_args[1]["render_options"]["padding"] == 10

    def test_shape_text(self):
        """Test text shaping."""
        renderer = typf.TextRenderer()
        mock_result = MagicMock()
        renderer._renderer.shape.return_value = mock_result

        result = renderer.shape("Hello", typf.Font("Arial", 24))
        assert result == mock_result

    def test_render_batch_sequential(self):
        """Test batch rendering in sequential mode."""
        renderer = typf.TextRenderer(parallel=False)
        renderer._renderer.render.return_value = b'PNG'

        items = [
            {"text": "Hello", "font": typf.Font("Arial", 24)},
            {"text": "World", "font": "Times"},
        ]

        results = renderer.render_batch(items, format="png")
        assert len(results) == 2
        assert all(r == b'PNG' for r in results)

    def test_render_batch_parallel(self):
        """Test batch rendering with native backend call."""
        renderer = typf.TextRenderer()
        renderer._renderer.render_batch.return_value = [b'PNG']

        font = typf.Font("Arial", 24)
        items = [{"text": "Hello", "font": font}]

        renderer.render_batch(items, format="png")
        args = renderer._renderer.render_batch.call_args[0]
        native_items = args[0]
        assert native_items[0]["font"] == font._font

    def test_render_to_file(self):
        """Test rendering directly to file."""
        renderer = typf.TextRenderer()
        renderer._renderer.render.return_value = b'PNG DATA'

        with tempfile.NamedTemporaryFile(suffix=".png", delete=False) as f:
            renderer.render_to_file("Test", "Arial", f.name)
            f.close()

            with open(f.name, 'rb') as f2:
                data = f2.read()

            os.unlink(f.name)
            assert data == b'PNG DATA'

    @patch('typf.HAS_NUMPY', True)
    def test_render_to_numpy(self):
        """Test rendering to numpy array."""
        renderer = typf.TextRenderer()
        renderer._renderer.render.return_value = (b'\x00' * 400, 10, 10)

        with patch.object(typf.Bitmap, 'to_numpy') as mock_to_numpy:
            mock_to_numpy.return_value = MagicMock()
            arr = renderer.render_to_numpy("Test", "Arial")
            assert mock_to_numpy.called

    @patch('typf.HAS_PIL', True)
    def test_render_to_pil(self):
        """Test rendering to PIL image."""
        renderer = typf.TextRenderer()
        renderer._renderer.render.return_value = (b'\x00' * 400, 10, 10)

        with patch.object(typf.Bitmap, 'to_pil') as mock_to_pil:
            mock_to_pil.return_value = MagicMock()
            img = renderer.render_to_pil("Test", "Arial")
            assert mock_to_pil.called

    def test_clear_cache(self):
        """Test cache clearing."""
        renderer = typf.TextRenderer()
        renderer.clear_cache()
        renderer._renderer.clear_cache.assert_called_once()


class TestConvenienceFunctions:
    """Test module-level convenience functions."""

    def test_render_function(self):
        """Test quick render function."""
        with patch('typf.TextRenderer') as mock_renderer_cls:
            mock_renderer = MagicMock()
            mock_renderer_cls.return_value = mock_renderer
            mock_renderer.render.return_value = b'RESULT'

            result = typf.render("Hello", "Arial", 24)
            assert result == b'RESULT'
            mock_renderer.render.assert_called_once()

    def test_render_to_file_function(self):
        """Test quick render to file function."""
        with patch('typf.TextRenderer') as mock_renderer_cls:
            mock_renderer = MagicMock()
            mock_renderer_cls.return_value = mock_renderer

            typf.render_to_file("Hello", "test.png", "Arial", 24)
            mock_renderer.render_to_file.assert_called_once()

    def test_shape_text_function(self):
        """Test quick shape function."""
        with patch('typf.TextRenderer') as mock_renderer_cls:
            mock_renderer = MagicMock()
            mock_renderer_cls.return_value = mock_renderer
            mock_renderer.shape.return_value = MagicMock()

            result = typf.shape_text("Hello", "Arial", 24)
            assert result is not None
            mock_renderer.shape.assert_called_once()

    @patch('platform.system')
    def test_list_backends(self, mock_system):
        """Test listing available backends."""
        mock_system.return_value = "Darwin"
        backends = typf.list_backends()
        assert "coretext" in backends
        assert "harfbuzz" in backends
        assert "pure" in backends

    @patch('platform.system')
    def test_get_default_backend(self, mock_system):
        """Test getting default backend."""
        mock_system.return_value = "Darwin"
        assert typf.get_default_backend() == "coretext"

        mock_system.return_value = "Windows"
        assert typf.get_default_backend() == "directwrite"

        mock_system.return_value = "Linux"
        assert typf.get_default_backend() == "harfbuzz"


class TestBatchProcessor:
    """Test BatchProcessor helper class."""

    def test_batch_processor_creation(self):
        """Test batch processor creation."""
        processor = typf.BatchProcessor()
        assert processor.renderer is not None
        assert processor.renderer.parallel is True

    def test_batch_process_single_font(self):
        """Test batch processing with single font."""
        processor = typf.BatchProcessor()
        processor.renderer._renderer.render_batch.return_value = [b'1', b'2', b'3']

        texts = ["Hello", "World", "Test"]
        font = typf.Font("Arial", 24)

        results = processor.process(texts, font)
        assert len(results) == 3

    def test_batch_process_multiple_fonts(self):
        """Test batch processing with multiple fonts."""
        processor = typf.BatchProcessor()
        processor.renderer._renderer.render_batch.return_value = [b'1', b'2']

        texts = ["Hello", "World"]
        fonts = [typf.Font("Arial", 24), typf.Font("Times", 18)]

        results = processor.process(texts, fonts)
        assert len(results) == 2

    def test_batch_process_invalid_fonts(self):
        """Test batch processing with mismatched fonts."""
        processor = typf.BatchProcessor()

        texts = ["Hello", "World", "Test"]
        fonts = [typf.Font("Arial", 24)]  # Only 1 font for 3 texts

        with pytest.raises(ValueError):
            processor.process(texts, fonts)


class TestVersionCheck:
    """Test version checking functionality."""

    @patch('platform.system')
    def test_check_version(self, mock_system):
        """Test version checking."""
        mock_system.return_value = "Darwin"

        info = typf.check_version()
        assert "version" in info
        assert "backends" in info
        assert "default_backend" in info
        assert "features" in info
        assert info["features"]["unicode_support"] is True
        assert info["features"]["batch_processing"] is True


class TestEnums:
    """Test enum types."""

    def test_render_format_enum(self):
        """Test RenderFormat enum."""
        assert typf.RenderFormat.RAW.value == "raw"
        assert typf.RenderFormat.PNG.value == "png"
        assert typf.RenderFormat.SVG.value == "svg"

    def test_direction_enum(self):
        """Test Direction enum."""
        assert typf.Direction.LEFT_TO_RIGHT.value == "ltr"
        assert typf.Direction.RIGHT_TO_LEFT.value == "rtl"
        assert typf.Direction.AUTO.value == "auto"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
