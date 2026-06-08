"""Unit tests for form data and file upload support."""

import pytest
from cclab.quasar import Form, File, UploadFile, FormMarker, FileMarker, App


class TestFormMarker:
    """Tests for Form marker."""

    def test_form_marker_required(self):
        """Test Form marker with required field."""
        marker = Form(...)
        assert isinstance(marker, FormMarker)
        assert marker.default is ...

    def test_form_marker_with_default(self):
        """Test Form marker with default value."""
        marker = Form("default_value")
        assert isinstance(marker, FormMarker)
        assert marker.default == "default_value"

    def test_form_marker_none_default(self):
        """Test Form marker with None default."""
        marker = Form(None)
        assert isinstance(marker, FormMarker)
        assert marker.default is None


class TestFileMarker:
    """Tests for File marker."""

    def test_file_marker_required(self):
        """Test File marker with required field."""
        marker = File(...)
        assert isinstance(marker, FileMarker)
        assert marker.default is ...

    def test_file_marker_with_default(self):
        """Test File marker with default value."""
        marker = File(None)
        assert isinstance(marker, FileMarker)
        assert marker.default is None


class TestUploadFile:
    """Tests for UploadFile class."""

    @pytest.fixture
    def sample_file(self):
        """Create a sample UploadFile."""
        return UploadFile(
            filename="test.txt",
            content_type="text/plain",
            data=b"Hello, World!",
            field_name="file"
        )

    @pytest.mark.asyncio
    async def test_read(self, sample_file):
        """Test reading file contents."""
        content = await sample_file.read()
        assert content == b"Hello, World!"

    @pytest.mark.asyncio
    async def test_write(self, sample_file):
        """Test writing to file."""
        new_data = b"New content"
        bytes_written = await sample_file.write(new_data)
        assert bytes_written == len(new_data)
        assert sample_file.data == new_data

    @pytest.mark.asyncio
    async def test_seek(self, sample_file):
        """Test seek operation."""
        result = await sample_file.seek(10)
        assert result == 0  # In-memory files don't support seeking

    @pytest.mark.asyncio
    async def test_close(self, sample_file):
        """Test close operation."""
        await sample_file.close()  # Should not raise

    def test_size_property(self, sample_file):
        """Test size property."""
        assert sample_file.size == len(b"Hello, World!")

    def test_metadata_properties(self, sample_file):
        """Test file metadata properties."""
        assert sample_file.filename == "test.txt"
        assert sample_file.content_type == "text/plain"
        assert sample_file.field_name == "file"


class TestFormParameterResolution:
    """Tests for form parameter resolution in App."""

    @pytest.fixture
    def app(self):
        """Create test app."""
        return App(title="Test App")

    def test_resolve_form_field_string(self, app):
        """Test resolving a string form field."""
        def handler(name: str = Form(...)):
            return name

        kwargs = app._resolve_form_parameters(
            handler,
            {"name": "John Doe"},
            {}
        )
        assert kwargs == {"name": "John Doe"}

    def test_resolve_form_field_int(self, app):
        """Test resolving an integer form field."""
        def handler(age: int = Form(...)):
            return age

        kwargs = app._resolve_form_parameters(
            handler,
            {"age": "25"},
            {}
        )
        assert kwargs == {"age": 25}

    def test_resolve_form_field_float(self, app):
        """Test resolving a float form field."""
        def handler(price: float = Form(...)):
            return price

        kwargs = app._resolve_form_parameters(
            handler,
            {"price": "19.99"},
            {}
        )
        assert kwargs == {"price": 19.99}

    def test_resolve_form_field_with_default(self, app):
        """Test resolving form field with default value."""
        def handler(bio: str = Form("")):
            return bio

        # Field not provided, should use default
        kwargs = app._resolve_form_parameters(handler, {}, {})
        assert kwargs == {"bio": ""}

        # Field provided, should use provided value
        kwargs = app._resolve_form_parameters(
            handler,
            {"bio": "Software Engineer"},
            {}
        )
        assert kwargs == {"bio": "Software Engineer"}

    def test_resolve_file_upload(self, app):
        """Test resolving file upload."""
        def handler(file: UploadFile = File(...)):
            return file

        upload = UploadFile(
            filename="test.txt",
            content_type="text/plain",
            data=b"test data",
            field_name="file"
        )

        kwargs = app._resolve_form_parameters(
            handler,
            {},
            {"file": upload}
        )
        assert kwargs == {"file": upload}

    def test_resolve_file_with_default(self, app):
        """Test resolving file upload with default."""
        def handler(file: UploadFile = File(None)):
            return file

        # File not provided, should use default
        kwargs = app._resolve_form_parameters(handler, {}, {})
        assert kwargs == {"file": None}

    def test_resolve_mixed_form_and_file(self, app):
        """Test resolving mixed form fields and file uploads."""
        def handler(
            name: str = Form(...),
            email: str = Form(...),
            avatar: UploadFile = File(...),
            bio: str = Form("")
        ):
            return {"name": name, "email": email, "avatar": avatar, "bio": bio}

        upload = UploadFile(
            filename="avatar.jpg",
            content_type="image/jpeg",
            data=b"image data",
            field_name="avatar"
        )

        kwargs = app._resolve_form_parameters(
            handler,
            {"name": "John", "email": "john@example.com"},
            {"avatar": upload}
        )

        assert kwargs == {
            "name": "John",
            "email": "john@example.com",
            "avatar": upload,
            "bio": ""
        }

    def test_resolve_optional_form_field(self, app):
        """Test resolving optional form field."""
        from typing import Optional

        def handler(age: Optional[int] = Form(None)):
            return age

        # Field not provided
        kwargs = app._resolve_form_parameters(handler, {}, {})
        assert kwargs == {"age": None}

        # Field provided
        kwargs = app._resolve_form_parameters(
            handler,
            {"age": "30"},
            {}
        )
        assert kwargs == {"age": 30}

    def test_ignore_non_form_parameters(self, app):
        """Test that non-form parameters are ignored."""
        def handler(
            user_id: str,  # No marker
            name: str = Form(...)
        ):
            return {"user_id": user_id, "name": name}

        kwargs = app._resolve_form_parameters(
            handler,
            {"name": "John"},
            {}
        )

        # Only form-marked parameter should be resolved
        assert kwargs == {"name": "John"}
        assert "user_id" not in kwargs


class TestFormParseData:
    """Tests for form data parsing."""

    @pytest.fixture
    def app(self):
        """Create test app."""
        return App(title="Test App")

    @pytest.mark.asyncio
    async def test_parse_form_data_stub(self, app):
        """Test that _parse_form_data returns None (stub)."""
        result = await app._parse_form_data(None)
        assert result is None
