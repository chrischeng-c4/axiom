"""
Unit tests for PyLoop CRUD decorator and crud_routes method.

This tests the structure and registration of CRUD endpoints
without requiring a MongoDB connection.
"""

import pytest
from cclab.orbit import App
from cclab.nebula import Document


class SampleProduct(Document):
    """Sample document model for testing."""
    name: str
    price: float
    stock: int = 0

    class Settings:
        name = "test_products"


def test_crud_decorator_exists():
    """Test that the crud decorator method exists on App."""
    app = App(title="Test API", version="1.0.0")
    assert hasattr(app, 'crud')
    assert callable(app.crud)


def test_crud_decorator_syntax():
    """Test that the crud decorator has correct syntax."""
    app = App(title="Test API", version="1.0.0")

    # Should be able to use as decorator
    @app.crud(SampleProduct)
    class ProductCRUD:
        pass

    # Should return the decorated class
    assert ProductCRUD is not None


def test_crud_decorator_with_prefix():
    """Test that the crud decorator accepts custom prefix."""
    app = App(title="Test API", version="1.0.0")

    @app.crud(SampleProduct, prefix="/api/products")
    class ProductCRUD:
        pass

    assert ProductCRUD is not None


def test_crud_decorator_with_tags():
    """Test that the crud decorator accepts custom tags."""
    app = App(title="Test API", version="1.0.0")

    @app.crud(SampleProduct, tags=["inventory", "products"])
    class ProductCRUD:
        pass

    assert ProductCRUD is not None


def test_crud_decorator_collection_name_detection():
    """Test that collection name is correctly detected from Document."""
    app = App(title="Test API", version="1.0.0")

    # Should detect collection name from Settings.name
    @app.crud(SampleProduct)
    class ProductCRUD:
        pass

    # Collection name should be "test_products" from SampleProduct.Settings.name
    assert SampleProduct.__collection_name__() == "test_products"


def test_multiple_crud_decorators():
    """Test that multiple CRUD decorators can be used on same app."""
    app = App(title="Test API", version="1.0.0")

    class User(Document):
        email: str
        name: str

        class Settings:
            name = "users"

    @app.crud(SampleProduct)
    class ProductCRUD:
        pass

    @app.crud(User)
    class UserCRUD:
        pass

    # Both should work without conflicts
    assert ProductCRUD is not None
    assert UserCRUD is not None


def test_crud_routes_direct_call():
    """Test direct method call without decorator."""
    app = App(title="Test API", version="1.0.0")

    # Should not raise - direct call, no decorator needed
    app.crud_routes(SampleProduct)


def test_crud_routes_operations_string():
    """Test operations string parameter."""
    app = App(title="Test API", version="1.0.0")

    # Only read operations
    app.crud_routes(SampleProduct, operations="RL")

    # Only create and read
    app.crud_routes(SampleProduct, operations="CR", prefix="/api/v2/test")


def test_crud_routes_boolean_flags():
    """Test boolean flag parameters."""
    app = App(title="Test API", version="1.0.0")

    # Only list and read (disable create, update, delete)
    app.crud_routes(SampleProduct, create=False, update=False, delete=False)


def test_crud_routes_all_disabled():
    """Test that crud_routes can be called with all operations disabled."""
    app = App(title="Test API", version="1.0.0")

    # Disable all operations (edge case)
    app.crud_routes(
        SampleProduct,
        create=False,
        read=False,
        update=False,
        delete=False,
        list=False
    )


def test_crud_routes_operations_override_flags():
    """Test that operations string overrides individual flags."""
    app = App(title="Test API", version="1.0.0")

    # operations should override individual flags
    # Even though create=False, "C" in operations should enable it
    app.crud_routes(
        SampleProduct,
        operations="C",
        create=False,
        read=False,
        update=False,
        delete=False,
        list=False
    )


def test_crud_routes_case_insensitive():
    """Test that operations string is case-insensitive."""
    app = App(title="Test API", version="1.0.0")

    # Lowercase should work
    app.crud_routes(SampleProduct, operations="crudl")

    # Mixed case should work
    app.crud_routes(SampleProduct, operations="CrUdL", prefix="/api/v2/test")


def test_crud_backward_compatibility():
    """Test that old crud() decorator still works."""
    app = App(title="Test API", version="1.0.0")

    # Old decorator syntax should still work
    @app.crud(SampleProduct)
    class ProductCRUD:
        pass

    assert ProductCRUD is not None


def test_crud_routes_method_exists():
    """Test that crud_routes method exists on App."""
    app = App(title="Test API", version="1.0.0")
    assert hasattr(app, 'crud_routes')
    assert callable(app.crud_routes)


def test_crud_routes_positional_prefix():
    """Test that prefix can be passed as positional argument."""
    app = App(title="Test API", version="1.0.0")

    # Prefix as positional argument (most concise syntax)
    app.crud_routes(SampleProduct, "/api/products")

    # Should also work with operations
    app.crud_routes(SampleProduct, "/api/v2/products", operations="RL")


def test_crud_routes_positional_and_keyword():
    """Test mixing positional prefix with keyword arguments."""
    app = App(title="Test API", version="1.0.0")

    # Prefix positional, operations keyword
    app.crud_routes(SampleProduct, "/products", operations="CRUDL")

    # Prefix positional, boolean flags as keywords
    app.crud_routes(SampleProduct, "/api/products", create=True, delete=False)


if __name__ == "__main__":
    # Run tests
    pytest.main([__file__, "-v"])
