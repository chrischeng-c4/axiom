"""Integration tests for ValidateOnSave constraint validation.

Tests verify that constraint validation (MinLen, MaxLen, Min, Max, Email, Url)
works end-to-end through the Python → Rust boundary.
"""

import pytest
from cclab.probe import expect
from typing import Annotated
from cclab import Document, init, MinLen, MaxLen, Min, Max, Email, Url
from cclab.nebula.types import PydanticObjectId


# Test models with constraints
class UserWithStringConstraints(Document):
    """Test model with string length constraints."""
    name: Annotated[str, MinLen(3), MaxLen(50)]
    email: str

    class Settings:
        name = "users_string_constraints"
        use_validation = True


class UserWithEmailConstraint(Document):
    """Test model with email format constraint."""
    name: str
    email: Annotated[str, Email()]

    class Settings:
        name = "users_email_constraint"
        use_validation = True


class UserWithUrlConstraint(Document):
    """Test model with URL format constraint."""
    name: str
    website: Annotated[str, Url()]

    class Settings:
        name = "users_url_constraint"
        use_validation = True


class ProductWithNumericConstraints(Document):
    """Test model with numeric constraints."""
    name: str
    price: Annotated[float, Min(0.01), Max(9999.99)]
    quantity: Annotated[int, Min(0), Max(1000000)]

    class Settings:
        name = "products_numeric_constraints"
        use_validation = True


class UserWithValidationDisabled(Document):
    """Test model with validation disabled."""
    name: Annotated[str, MinLen(3)]
    email: Annotated[str, Email()]

    class Settings:
        name = "users_validation_disabled"
        use_validation = False  # Validation should be skipped


@pytest.fixture(scope="module")
async def setup_db():
    """Initialize database connection."""
    await init("mongodb://localhost:27017/test_constraint_validation")

    # Clean up collections (delete_many with empty filter deletes all)
    await UserWithStringConstraints.delete_many({})
    await UserWithEmailConstraint.delete_many({})
    await UserWithUrlConstraint.delete_many({})
    await ProductWithNumericConstraints.delete_many({})
    await UserWithValidationDisabled.delete_many({})


# String Constraint Tests
@pytest.mark.asyncio
async def test_minlen_validation_success(setup_db):
    """Test that MinLen validation passes for valid strings."""
    user = UserWithStringConstraints(name="Alice", email="alice@example.com")
    user_id = await user.save()
    assert user_id is not None

    # Clean up
    await UserWithStringConstraints.delete_many({})


@pytest.mark.asyncio
async def test_minlen_validation_failure(setup_db):
    """Test that MinLen validation rejects strings that are too short."""
    user = UserWithStringConstraints(name="Al", email="al@example.com")  # Only 2 chars

    exception_raised = False
    error_msg = ""
    try:
        await user.save()
    except ValueError as e:
        exception_raised = True
        error_msg = str(e)
    assert exception_raised, "MinLen validation should raise ValueError"
    assert "ValidationError" in error_msg
    assert "name" in error_msg
    assert "too short" in error_msg.lower() or "min" in error_msg.lower()


@pytest.mark.asyncio
async def test_maxlen_validation_success(setup_db):
    """Test that MaxLen validation passes for valid strings."""
    user = UserWithStringConstraints(
        name="A" * 50,  # Exactly 50 chars (max)
        email="test@example.com"
    )
    user_id = await user.save()
    assert user_id is not None

    # Clean up
    await UserWithStringConstraints.delete_many({})


@pytest.mark.asyncio
async def test_maxlen_validation_failure(setup_db):
    """Test that MaxLen validation rejects strings that are too long."""
    user = UserWithStringConstraints(
        name="A" * 51,  # 51 chars (exceeds max)
        email="test@example.com"
    )

    exception_raised = False
    error_msg = ""
    try:
        await user.save()
    except ValueError as e:
        exception_raised = True
        error_msg = str(e)
    assert exception_raised, "MaxLen validation should raise ValueError"
    assert "ValidationError" in error_msg
    assert "name" in error_msg
    assert "too long" in error_msg.lower() or "max" in error_msg.lower()


# Email Format Tests
@pytest.mark.asyncio
async def test_email_validation_success(setup_db):
    """Test that Email validation passes for valid email addresses."""
    valid_emails = [
        "user@example.com",
        "test.user@example.co.uk",
        "user+tag@example.com",
        "user_name@example.com",
    ]

    for email in valid_emails:
        user = UserWithEmailConstraint(name="Test User", email=email)
        user_id = await user.save()
        assert user_id is not None

    # Clean up
    await UserWithEmailConstraint.delete_many({})


@pytest.mark.asyncio
async def test_email_validation_failure(setup_db):
    """Test that Email validation rejects invalid email addresses."""
    invalid_emails = [
        "not-an-email",
        "@example.com",
        "user@",
        "user@.com",
        "user space@example.com",
    ]

    for email in invalid_emails:
        user = UserWithEmailConstraint(name="Test User", email=email)

        exception_raised = False
        error_msg = ""
        try:
            await user.save()
        except ValueError as e:
            exception_raised = True
            error_msg = str(e)
        assert exception_raised, f"Email validation should raise ValueError for {email}"
        assert "ValidationError" in error_msg
        assert "email" in error_msg
        assert "invalid" in error_msg.lower() or "format" in error_msg.lower()


# URL Format Tests
@pytest.mark.asyncio
async def test_url_validation_success(setup_db):
    """Test that Url validation passes for valid URLs."""
    valid_urls = [
        "https://example.com",
        "http://example.com/path",
        "https://example.com/path?query=value",
        "https://subdomain.example.com",
    ]

    for url in valid_urls:
        user = UserWithUrlConstraint(name="Test User", website=url)
        user_id = await user.save()
        assert user_id is not None

    # Clean up
    await UserWithUrlConstraint.delete_many({})


@pytest.mark.asyncio
async def test_url_validation_failure(setup_db):
    """Test that Url validation rejects invalid URLs."""
    invalid_urls = [
        "not-a-url",
        "htp://example.com",  # Typo in protocol
        "example.com",  # Missing protocol
        "https://",  # Missing domain
    ]

    for url in invalid_urls:
        user = UserWithUrlConstraint(name="Test User", website=url)

        exception_raised = False
        error_msg = ""
        try:
            await user.save()
        except ValueError as e:
            exception_raised = True
            error_msg = str(e)
        assert exception_raised, f"URL validation should raise ValueError for {url}"
        assert "ValidationError" in error_msg
        assert "website" in error_msg
        assert "invalid" in error_msg.lower() or "format" in error_msg.lower()


# Numeric Constraint Tests
@pytest.mark.asyncio
async def test_min_validation_success(setup_db):
    """Test that Min validation passes for valid numbers."""
    product = ProductWithNumericConstraints(
        name="Test Product",
        price=0.01,  # Minimum allowed
        quantity=0  # Minimum allowed
    )
    product_id = await product.save()
    assert product_id is not None

    # Clean up
    await ProductWithNumericConstraints.delete_many({})


@pytest.mark.asyncio
async def test_min_validation_failure_float(setup_db):
    """Test that Min validation rejects floats below minimum."""
    product = ProductWithNumericConstraints(
        name="Test Product",
        price=0.00,  # Below minimum (0.01)
        quantity=10
    )

    exception_raised = False
    error_msg = ""
    try:
        await product.save()
    except ValueError as e:
        exception_raised = True
        error_msg = str(e)
    assert exception_raised, "Min validation should raise ValueError for float below minimum"
    assert "ValidationError" in error_msg
    assert "price" in error_msg
    assert "below" in error_msg.lower() or "min" in error_msg.lower()


@pytest.mark.asyncio
async def test_min_validation_failure_int(setup_db):
    """Test that Min validation rejects integers below minimum."""
    product = ProductWithNumericConstraints(
        name="Test Product",
        price=10.0,
        quantity=-1  # Below minimum (0)
    )

    exception_raised = False
    error_msg = ""
    try:
        await product.save()
    except ValueError as e:
        exception_raised = True
        error_msg = str(e)
    assert exception_raised, "Min validation should raise ValueError for int below minimum"
    assert "ValidationError" in error_msg
    assert "quantity" in error_msg
    assert "below" in error_msg.lower() or "min" in error_msg.lower()


@pytest.mark.asyncio
async def test_max_validation_success(setup_db):
    """Test that Max validation passes for valid numbers."""
    product = ProductWithNumericConstraints(
        name="Test Product",
        price=9999.99,  # Maximum allowed
        quantity=1000000  # Maximum allowed
    )
    product_id = await product.save()
    assert product_id is not None

    # Clean up
    await ProductWithNumericConstraints.delete_many({})


@pytest.mark.asyncio
async def test_max_validation_failure_float(setup_db):
    """Test that Max validation rejects floats above maximum."""
    product = ProductWithNumericConstraints(
        name="Test Product",
        price=10000.00,  # Above maximum (9999.99)
        quantity=100
    )

    exception_raised = False
    error_msg = ""
    try:
        await product.save()
    except ValueError as e:
        exception_raised = True
        error_msg = str(e)
    assert exception_raised, "Max validation should raise ValueError for float above maximum"
    assert "ValidationError" in error_msg
    assert "price" in error_msg
    assert "above" in error_msg.lower() or "max" in error_msg.lower()


@pytest.mark.asyncio
async def test_max_validation_failure_int(setup_db):
    """Test that Max validation rejects integers above maximum."""
    product = ProductWithNumericConstraints(
        name="Test Product",
        price=100.0,
        quantity=1000001  # Above maximum (1000000)
    )

    exception_raised = False
    error_msg = ""
    try:
        await product.save()
    except ValueError as e:
        exception_raised = True
        error_msg = str(e)
    assert exception_raised, "Max validation should raise ValueError for int above maximum"
    assert "ValidationError" in error_msg
    assert "quantity" in error_msg
    assert "above" in error_msg.lower() or "max" in error_msg.lower()


# Settings Control Tests
@pytest.mark.asyncio
async def test_validation_disabled_by_settings(setup_db):
    """Test that use_validation=False disables constraint validation."""
    # These should normally fail validation, but Settings.use_validation=False
    user = UserWithValidationDisabled(
        name="A",  # Too short (MinLen=3)
        email="not-an-email"  # Invalid email format
    )

    # Should NOT raise validation error because use_validation=False
    user_id = await user.save()
    assert user_id is not None

    # Verify data was saved (even though it violates constraints)
    retrieved = await UserWithValidationDisabled.find_one(
        UserWithValidationDisabled.id == user.id
    )
    assert retrieved is not None
    assert retrieved.name == "A"
    assert retrieved.email == "not-an-email"

    # Clean up
    await UserWithValidationDisabled.delete_many({})


# Error Message Quality Tests
@pytest.mark.asyncio
async def test_error_message_includes_field_name(setup_db):
    """Test that validation errors include the field name."""
    user = UserWithStringConstraints(name="AB", email="test@example.com")

    exception_raised = False
    error_msg = ""
    try:
        await user.save()
    except ValueError as e:
        exception_raised = True
        error_msg = str(e)
    assert exception_raised, "Validation should raise ValueError"
    assert "name" in error_msg  # Field name should be in error message


@pytest.mark.asyncio
async def test_error_message_includes_constraint_details(setup_db):
    """Test that validation errors include constraint details."""
    user = UserWithStringConstraints(name="AB", email="test@example.com")

    exception_raised = False
    error_msg = ""
    try:
        await user.save()
    except ValueError as e:
        exception_raised = True
        error_msg = str(e)
    assert exception_raised, "Validation should raise ValueError"
    # Should mention the minimum length (3)
    assert "3" in error_msg or "min" in error_msg.lower()


@pytest.mark.asyncio
async def test_error_message_from_rust(setup_db):
    """Test that validation errors originate from Rust (not Python)."""
    user = UserWithEmailConstraint(name="Test", email="invalid")

    exception_raised = False
    error_msg = ""
    try:
        await user.save()
    except ValueError as e:
        exception_raised = True
        error_msg = str(e)
    assert exception_raised, "Validation should raise ValueError"
    # Rust error messages start with "ValidationError:"
    assert "ValidationError" in error_msg
