"""Example usage of PostgreSQL ORM validation module.

This demonstrates:
1. @validates() decorator for field-level validation
2. @validates_many() decorator for multi-field validation
3. TypeDecorator for custom type coercion
4. Built-in validators
5. AutoCoerceMixin for automatic type conversion
"""

from decimal import Decimal
from datetime import datetime, date

from cclab.titan import Table, Column
from cclab.titan.validation import (
    validates, validates_many,
    TypeDecorator,
    ValidationError,
    AutoCoerceMixin,
    validate_email, validate_min_length, validate_range,
    coerce_int, coerce_datetime,
)


# ============================================================================
# Example 1: Basic Field Validation
# ============================================================================

class User(Table):
    """User table with email and age validation."""

    email: str
    age: int
    username: str

    class Settings:
        table_name = "users"

    @validates('email')
    def validate_email_field(self, key, value):
        """Validate email format and convert to lowercase."""
        if not validate_email(value):
            raise ValidationError('email', 'Invalid email format')
        return value.lower()

    @validates('age')
    def validate_age_field(self, key, value):
        """Validate age is between 0 and 150."""
        validate_range(value, 0, 150)
        return value

    @validates('username')
    def validate_username_field(self, key, value):
        """Validate username length."""
        validate_min_length(value, 3)
        return value


# ============================================================================
# Example 2: Multi-Field Validation
# ============================================================================

class PasswordForm(Table):
    """Form with password confirmation validation."""

    email: str
    password: str
    password_confirm: str

    class Settings:
        table_name = "password_forms"

    @validates_many('password', 'password_confirm')
    def validate_passwords_match(self, values):
        """Ensure password and confirmation match."""
        pwd = values.get('password')
        pwd_confirm = values.get('password_confirm')

        if pwd and pwd_confirm and pwd != pwd_confirm:
            raise ValidationError('password', "Passwords don't match")

        # Validate password strength
        if pwd and len(pwd) < 8:
            raise ValidationError('password', 'Password must be at least 8 characters')

        return values


# ============================================================================
# Example 3: Custom Type Decorator
# ============================================================================

class UppercaseString(TypeDecorator):
    """Custom type that stores strings as uppercase."""

    impl = str

    def process_bind_param(self, value, dialect=None):
        """Convert to uppercase when storing in database."""
        return value.upper() if value else value

    def process_result_value(self, value, dialect=None):
        """Keep uppercase when retrieving from database."""
        return value.upper() if value else value


class LowercaseString(TypeDecorator):
    """Custom type that stores strings as lowercase."""

    impl = str

    def process_bind_param(self, value, dialect=None):
        """Convert to lowercase when storing in database."""
        return value.lower() if value else value

    def process_result_value(self, value, dialect=None):
        """Keep lowercase when retrieving from database."""
        return value.lower() if value else value


class Company(Table):
    """Company table with custom type decorators."""

    name: str
    stock_symbol: str  # Should be uppercase
    email: str  # Should be lowercase

    class Settings:
        table_name = "companies"


# ============================================================================
# Example 4: Auto-Coercion Mixin
# ============================================================================

class Product(AutoCoerceMixin, Table):  # AutoCoerceMixin must come first!
    """Product table with automatic type coercion.

    NOTE: AutoCoerceMixin must be listed BEFORE Table in the inheritance list.
    """

    name: str
    price: Decimal
    quantity: int
    available: bool
    created_at: datetime

    class Settings:
        table_name = "products"

    # Optionally specify which fields to auto-coerce
    # If not specified, all fields are auto-coerced
    __coerce_fields__ = {'price', 'quantity', 'available', 'created_at'}


# ============================================================================
# Example 5: Combining Validation and Coercion
# ============================================================================

class Order(AutoCoerceMixin, Table):  # AutoCoerceMixin first!
    """Order table with both validation and coercion."""

    customer_email: str
    total_amount: Decimal
    quantity: int
    order_date: datetime

    class Settings:
        table_name = "orders"

    # Auto-coerce numeric and date fields
    __coerce_fields__ = {'total_amount', 'quantity', 'order_date'}

    @validates('customer_email')
    def validate_customer_email(self, key, value):
        """Validate and normalize email."""
        if not validate_email(value):
            raise ValidationError('customer_email', 'Invalid email format')
        return value.lower()

    @validates('total_amount')
    def validate_total_amount(self, key, value):
        """Ensure total amount is positive."""
        if value <= 0:
            raise ValidationError('total_amount', 'Total amount must be positive')
        return value

    @validates('quantity')
    def validate_quantity(self, key, value):
        """Ensure quantity is positive."""
        validate_range(value, 1, 10000)
        return value


# ============================================================================
# Example 6: Complex Multi-Field Validation
# ============================================================================

class Event(Table):
    """Event table with complex validation."""

    name: str
    start_date: datetime
    end_date: datetime
    max_attendees: int
    current_attendees: int

    class Settings:
        table_name = "events"

    @validates('name')
    def validate_name(self, key, value):
        """Validate event name."""
        validate_min_length(value, 5)
        return value

    @validates_many('start_date', 'end_date')
    def validate_dates(self, values):
        """Ensure end date is after start date."""
        start = values.get('start_date')
        end = values.get('end_date')

        if start and end and end <= start:
            raise ValidationError('end_date', 'End date must be after start date')

        return values

    @validates_many('max_attendees', 'current_attendees')
    def validate_attendees(self, values):
        """Ensure current attendees doesn't exceed max."""
        max_att = values.get('max_attendees')
        current = values.get('current_attendees')

        if max_att and current and current > max_att:
            raise ValidationError('current_attendees',
                                'Current attendees cannot exceed maximum')

        return values


# ============================================================================
# Example Usage
# ============================================================================

async def main():
    """Example usage of validated tables."""
    from cclab.titan import init, close

    # Initialize connection
    await init("postgresql://localhost/testdb")

    try:
        # Example 1: Basic validation
        user = User(
            email="ALICE@EXAMPLE.COM",  # Will be lowercased
            age=30,
            username="alice123"
        )
        await user.save()
        print(f"Saved user: {user.email}")  # "alice@example.com"

        # This will raise ValidationError
        try:
            bad_user = User(email="invalid-email", age=30, username="alice")
            await bad_user.save()
        except ValidationError as e:
            print(f"Validation error: {e}")

        # Example 4: Auto-coercion
        product = Product(
            name="Widget",
            price="19.99",  # String will be coerced to Decimal
            quantity="100",  # String will be coerced to int
            available="yes",  # String will be coerced to bool
            created_at="2024-01-15T10:30:00"  # String will be coerced to datetime
        )
        await product.save()
        print(f"Product price: {product.price} (type: {type(product.price)})")
        # Output: Product price: 19.99 (type: <class 'decimal.Decimal'>)

        # Example 5: Combined validation and coercion
        order = Order(
            customer_email="BOB@EXAMPLE.COM",  # Will be validated and lowercased
            total_amount="99.99",  # String coerced to Decimal
            quantity="5",  # String coerced to int
            order_date="2024-01-15T14:30:00"  # String coerced to datetime
        )
        await order.save()
        print(f"Order total: {order.total_amount}")

        # Example 6: Multi-field validation
        event = Event(
            name="Tech Conference 2024",
            start_date=datetime(2024, 6, 1, 9, 0),
            end_date=datetime(2024, 6, 3, 17, 0),
            max_attendees=100,
            current_attendees=45
        )
        await event.save()

        # This will raise ValidationError (end before start)
        try:
            bad_event = Event(
                name="Bad Event",
                start_date=datetime(2024, 6, 3, 9, 0),
                end_date=datetime(2024, 6, 1, 17, 0),  # Before start!
                max_attendees=100,
                current_attendees=0
            )
            await bad_event.save()
        except ValidationError as e:
            print(f"Date validation error: {e}")

    finally:
        # Close connection
        await close()


if __name__ == "__main__":
    import asyncio
    asyncio.run(main())
