"""ORM-level validation module for PostgreSQL tables.

This module provides SQLAlchemy-style validation capabilities:
- @validates() decorator for field-level validation
- @validates_many() decorator for multi-field validation
- TypeDecorator for custom type coercion
- Coercion functions for common types
- Built-in validators for common patterns
- Auto-coercion mixin for automatic type conversion

Example:
    >>> from cclab.pg import Table, Column
    >>> from cclab.pg.validation import validates, validate_email
    >>>
    >>> class User(Table):
    ...     email: str
    ...     age: int
    ...
    ...     @validates('email')
    ...     def validate_email_field(self, key, value):
    ...         if not validate_email(value):
    ...             raise ValidationError('email', 'Invalid email format')
    ...         return value.lower()
    ...
    ...     @validates('age')
    ...     def validate_age(self, key, value):
    ...         if value < 0:
    ...             raise ValidationError('age', 'Age must be non-negative')
    ...         return value
    ...
    >>> user = User(email="ALICE@EXAMPLE.COM", age=30)
    >>> user.email  # "alice@example.com" (lowercased by validator)
"""

from __future__ import annotations

import re
from datetime import datetime, date
from decimal import Decimal
from typing import Any, Callable, Dict, List, Optional, Pattern, Set, Type, TypeVar, Union, get_type_hints
from functools import wraps

__all__ = [
    # Decorators
    "validates",
    "validates_many",
    # Type Decorator
    "TypeDecorator",
    # Coercion functions
    "coerce_int",
    "coerce_float",
    "coerce_str",
    "coerce_bool",
    "coerce_datetime",
    "coerce_date",
    "coerce_decimal",
    # Exception
    "ValidationError",
    # Registry
    "ValidatorRegistry",
    # Mixin
    "AutoCoerceMixin",
    # Built-in validators
    "validate_not_empty",
    "validate_email",
    "validate_url",
    "validate_min_length",
    "validate_max_length",
    "validate_regex",
    "validate_range",
    "validate_min_value",
    "validate_max_value",
    "validate_in_list",
    "validate_positive",
    "validate_non_negative",
]


# Type variable for validators
T = TypeVar('T')
ValidatorFunc = Callable[[Any, str, Any], Any]
MultiValidatorFunc = Callable[[Any, Dict[str, Any]], Dict[str, Any]]


# ============================================================================
# Custom Exception
# ============================================================================

class ValidationError(ValueError):
    """
    Raised when field validation fails.

    Attributes:
        field: Name of the field that failed validation
        message: Error message describing the validation failure

    Example:
        >>> raise ValidationError('email', 'Invalid email format')
    """

    def __init__(self, field: str, message: str) -> None:
        """
        Initialize validation error.

        Args:
            field: Field name that failed validation
            message: Error message
        """
        self.field = field
        self.message = message
        super().__init__(f"Validation failed for field '{field}': {message}")


# ============================================================================
# Validator Registry
# ============================================================================

class ValidatorRegistry:
    """
    Global registry for validators.

    Stores validator functions for each class and field combination.
    Supports field-level and multi-field validators.

    Example:
        >>> registry = ValidatorRegistry()
        >>> registry.register(User, 'email', validate_email_func)
        >>> validators = registry.get_validators(User, 'email')
    """

    def __init__(self) -> None:
        """Initialize validator registry."""
        # Field-level validators: {class: {field: [validators]}}
        self._field_validators: Dict[Type, Dict[str, List[ValidatorFunc]]] = {}
        # Multi-field validators: {class: [validators]}
        self._multi_validators: Dict[Type, List[MultiValidatorFunc]] = {}

    def register(self, cls: Type, field: str, validator: ValidatorFunc) -> None:
        """
        Register a field-level validator.

        Args:
            cls: Table class
            field: Field name
            validator: Validator function
        """
        if cls not in self._field_validators:
            self._field_validators[cls] = {}
        if field not in self._field_validators[cls]:
            self._field_validators[cls][field] = []
        self._field_validators[cls][field].append(validator)

    def register_multi(self, cls: Type, validator: MultiValidatorFunc) -> None:
        """
        Register a multi-field validator.

        Args:
            cls: Table class
            validator: Multi-field validator function
        """
        if cls not in self._multi_validators:
            self._multi_validators[cls] = []
        self._multi_validators[cls].append(validator)

    def get_validators(self, cls: Type, field: str) -> List[ValidatorFunc]:
        """
        Get all validators for a field.

        Args:
            cls: Table class
            field: Field name

        Returns:
            List of validator functions (empty if none registered)
        """
        # Check class hierarchy for validators
        validators = []
        for klass in cls.__mro__:
            if klass in self._field_validators and field in self._field_validators[klass]:
                validators.extend(self._field_validators[klass][field])
        return validators

    def get_multi_validators(self, cls: Type) -> List[MultiValidatorFunc]:
        """
        Get all multi-field validators for a class.

        Args:
            cls: Table class

        Returns:
            List of multi-field validator functions
        """
        # Check class hierarchy for validators
        validators = []
        for klass in cls.__mro__:
            if klass in self._multi_validators:
                validators.extend(self._multi_validators[klass])
        return validators

    def validate_field(self, instance: Any, field: str, value: Any) -> Any:
        """
        Run all validators for a field.

        Args:
            instance: Table instance
            field: Field name
            value: Value to validate

        Returns:
            Validated (possibly modified) value

        Raises:
            ValidationError: If validation fails
        """
        validators = self.get_validators(type(instance), field)
        for validator in validators:
            value = validator(instance, field, value)
        return value

    def validate_many(self, instance: Any, values: Dict[str, Any]) -> Dict[str, Any]:
        """
        Run all multi-field validators.

        Args:
            instance: Table instance
            values: Dictionary of field values

        Returns:
            Validated (possibly modified) values

        Raises:
            ValidationError: If validation fails
        """
        validators = self.get_multi_validators(type(instance))
        for validator in validators:
            values = validator(instance, values)
        return values


# Global validator registry
_validator_registry = ValidatorRegistry()


# ============================================================================
# Decorator Functions
# ============================================================================

def validates(*fields: str) -> Callable[[ValidatorFunc], ValidatorFunc]:
    """
    Decorator to register a field validator.

    The decorated method is called when the specified field(s) are set.
    It receives the field name and value, and should return the (possibly modified) value.

    Args:
        *fields: Field name(s) to validate

    Returns:
        Decorator function

    Example:
        >>> class User(Table):
        ...     email: str
        ...
        ...     @validates('email')
        ...     def validate_email(self, key, value):
        ...         if '@' not in value:
        ...             raise ValidationError('email', 'Invalid email')
        ...         return value.lower()
    """
    def decorator(func: ValidatorFunc) -> ValidatorFunc:
        # Store field names on the function for later registration
        if not hasattr(func, '_validates_fields'):
            func._validates_fields = []  # type: ignore
        func._validates_fields.extend(fields)  # type: ignore

        @wraps(func)
        def wrapper(self: Any, key: str, value: Any) -> Any:
            return func(self, key, value)

        # Copy the metadata
        wrapper._validates_fields = func._validates_fields  # type: ignore
        return wrapper

    return decorator


def validates_many(*fields: str) -> Callable[[MultiValidatorFunc], MultiValidatorFunc]:
    """
    Decorator to register a multi-field validator.

    The decorated method is called when any of the specified fields are set.
    It receives a dictionary of values and should return the (possibly modified) dictionary.

    Args:
        *fields: Field names to validate together

    Returns:
        Decorator function

    Example:
        >>> class User(Table):
        ...     password: str
        ...     password_confirm: str
        ...
        ...     @validates_many('password', 'password_confirm')
        ...     def validate_passwords(self, values):
        ...         if values.get('password') != values.get('password_confirm'):
        ...             raise ValidationError('password', "Passwords don't match")
        ...         return values
    """
    def decorator(func: MultiValidatorFunc) -> MultiValidatorFunc:
        # Store field names on the function for later registration
        if not hasattr(func, '_validates_many_fields'):
            func._validates_many_fields = []  # type: ignore
        func._validates_many_fields.extend(fields)  # type: ignore

        @wraps(func)
        def wrapper(self: Any, values: Dict[str, Any]) -> Dict[str, Any]:
            return func(self, values)

        # Copy the metadata
        wrapper._validates_many_fields = func._validates_many_fields  # type: ignore
        return wrapper

    return decorator


# ============================================================================
# Type Decorator Base Class
# ============================================================================

class TypeDecorator:
    """
    Base class for custom type decorators with coercion.

    Subclass this to create custom types that automatically convert
    between Python and database representations.

    Attributes:
        impl: The underlying Python type (e.g., str, int)

    Example:
        >>> class LowercaseString(TypeDecorator):
        ...     impl = str
        ...
        ...     def process_bind_param(self, value, dialect):
        ...         return value.lower() if value else value
        ...
        ...     def process_result_value(self, value, dialect):
        ...         return value.lower() if value else value
        ...
        >>> class User(Table):
        ...     email: LowercaseString
    """

    impl: Type = str

    def process_bind_param(self, value: Any, dialect: Optional[str] = None) -> Any:
        """
        Convert Python value to database value.

        Args:
            value: Python value
            dialect: Database dialect (e.g., 'postgresql')

        Returns:
            Database-compatible value
        """
        return value

    def process_result_value(self, value: Any, dialect: Optional[str] = None) -> Any:
        """
        Convert database value to Python value.

        Args:
            value: Database value
            dialect: Database dialect (e.g., 'postgresql')

        Returns:
            Python value
        """
        return value

    def coerce(self, value: Any) -> Any:
        """
        Coerce value to the implementation type.

        Args:
            value: Value to coerce

        Returns:
            Coerced value
        """
        if value is None:
            return None
        return self.impl(value)


# ============================================================================
# Coercion Functions
# ============================================================================

def coerce_int(value: Any) -> Optional[int]:
    """
    Coerce value to int.

    Args:
        value: Value to coerce

    Returns:
        Integer value or None

    Raises:
        ValueError: If value cannot be coerced to int

    Example:
        >>> coerce_int("42")
        42
        >>> coerce_int("3.14")
        3
        >>> coerce_int(None)
        None
    """
    if value is None:
        return None
    if isinstance(value, int):
        return value
    if isinstance(value, (float, str)):
        return int(value)
    raise ValueError(f"Cannot coerce {type(value).__name__} to int")


def coerce_float(value: Any) -> Optional[float]:
    """
    Coerce value to float.

    Args:
        value: Value to coerce

    Returns:
        Float value or None

    Raises:
        ValueError: If value cannot be coerced to float

    Example:
        >>> coerce_float("3.14")
        3.14
        >>> coerce_float(42)
        42.0
    """
    if value is None:
        return None
    if isinstance(value, float):
        return value
    if isinstance(value, (int, str)):
        return float(value)
    raise ValueError(f"Cannot coerce {type(value).__name__} to float")


def coerce_str(value: Any) -> Optional[str]:
    """
    Coerce value to str.

    Args:
        value: Value to coerce

    Returns:
        String value or None

    Example:
        >>> coerce_str(42)
        "42"
        >>> coerce_str(None)
        None
    """
    if value is None:
        return None
    return str(value)


def coerce_bool(value: Any) -> Optional[bool]:
    """
    Coerce value to bool.

    Handles common string representations like 'true', 'false', 'yes', 'no', '1', '0'.

    Args:
        value: Value to coerce

    Returns:
        Boolean value or None

    Example:
        >>> coerce_bool("true")
        True
        >>> coerce_bool("0")
        False
        >>> coerce_bool(1)
        True
    """
    if value is None:
        return None
    if isinstance(value, bool):
        return value
    if isinstance(value, int):
        return bool(value)
    if isinstance(value, str):
        lower = value.lower()
        if lower in ('true', 'yes', '1', 't', 'y'):
            return True
        if lower in ('false', 'no', '0', 'f', 'n'):
            return False
        raise ValueError(f"Cannot coerce string '{value}' to bool")
    raise ValueError(f"Cannot coerce {type(value).__name__} to bool")


def coerce_datetime(value: Any) -> Optional[datetime]:
    """
    Coerce value to datetime.

    Args:
        value: Value to coerce (datetime, date, timestamp, or ISO string)

    Returns:
        Datetime value or None

    Raises:
        ValueError: If value cannot be coerced to datetime

    Example:
        >>> coerce_datetime("2024-01-15T10:30:00")
        datetime(2024, 1, 15, 10, 30, 0)
        >>> coerce_datetime(1705315800)  # Unix timestamp
        datetime(...)
    """
    if value is None:
        return None
    if isinstance(value, datetime):
        return value
    if isinstance(value, date):
        return datetime.combine(value, datetime.min.time())
    if isinstance(value, (int, float)):
        # Unix timestamp
        return datetime.fromtimestamp(value)
    if isinstance(value, str):
        # Try ISO format
        try:
            return datetime.fromisoformat(value.replace('Z', '+00:00'))
        except ValueError:
            pass
        raise ValueError(f"Cannot parse datetime from string: {value}")
    raise ValueError(f"Cannot coerce {type(value).__name__} to datetime")


def coerce_date(value: Any) -> Optional[date]:
    """
    Coerce value to date.

    Args:
        value: Value to coerce (date, datetime, or ISO string)

    Returns:
        Date value or None

    Raises:
        ValueError: If value cannot be coerced to date

    Example:
        >>> coerce_date("2024-01-15")
        date(2024, 1, 15)
        >>> coerce_date(datetime(2024, 1, 15, 10, 30))
        date(2024, 1, 15)
    """
    if value is None:
        return None
    if isinstance(value, date) and not isinstance(value, datetime):
        return value
    if isinstance(value, datetime):
        return value.date()
    if isinstance(value, str):
        try:
            return date.fromisoformat(value)
        except ValueError:
            pass
        raise ValueError(f"Cannot parse date from string: {value}")
    raise ValueError(f"Cannot coerce {type(value).__name__} to date")


def coerce_decimal(value: Any) -> Optional[Decimal]:
    """
    Coerce value to Decimal.

    Args:
        value: Value to coerce

    Returns:
        Decimal value or None

    Raises:
        ValueError: If value cannot be coerced to Decimal

    Example:
        >>> coerce_decimal("3.14159")
        Decimal('3.14159')
        >>> coerce_decimal(3.14)
        Decimal('3.14')
    """
    if value is None:
        return None
    if isinstance(value, Decimal):
        return value
    if isinstance(value, (int, float, str)):
        return Decimal(str(value))
    raise ValueError(f"Cannot coerce {type(value).__name__} to Decimal")


# ============================================================================
# Auto-Coercion Mixin
# ============================================================================

class AutoCoerceMixin:
    """
    Mixin that automatically coerces field values based on type hints.

    Add this to your Table class to enable automatic type coercion.
    Configure which fields to coerce using __coerce_fields__ class attribute.

    IMPORTANT: This mixin must be listed BEFORE Table in the inheritance list
    so its __setattr__ is called first.

    Attributes:
        __coerce_fields__: Set of field names to auto-coerce (None = all fields)

    Example:
        >>> class User(AutoCoerceMixin, Table):  # Note: AutoCoerceMixin first!
        ...     age: int
        ...     score: float
        ...     active: bool
        ...
        ...     __coerce_fields__ = {'age', 'score'}  # Only coerce these
        ...
        >>> user = User(age="25", score="3.14", active="true")
        >>> user.age  # 25 (int)
        >>> user.score  # 3.14 (float)
    """

    __coerce_fields__: Optional[Set[str]] = None  # None means coerce all fields

    def __setattr__(self, name: str, value: Any) -> None:
        """
        Set attribute with automatic type coercion.

        Args:
            name: Attribute name
            value: Attribute value (will be coerced if needed)
        """
        # Skip special attributes and id
        if name.startswith('_') or name == 'id':
            super().__setattr__(name, value)
            return

        # Check if this field should be coerced
        if self.__coerce_fields__ is not None and name not in self.__coerce_fields__:
            super().__setattr__(name, value)
            return

        # Get type hint for the field
        try:
            hints = get_type_hints(type(self))
            if name in hints:
                target_type = hints[name]

                # Handle Optional types
                if hasattr(target_type, '__origin__') and target_type.__origin__ is Union:
                    args = target_type.__args__
                    if type(None) in args:
                        # Optional type - get the non-None type
                        target_type = next(t for t in args if t is not type(None))

                # Coerce based on type
                if value is not None:
                    if target_type is int:
                        value = coerce_int(value)
                    elif target_type is float:
                        value = coerce_float(value)
                    elif target_type is str:
                        value = coerce_str(value)
                    elif target_type is bool:
                        value = coerce_bool(value)
                    elif target_type is datetime:
                        value = coerce_datetime(value)
                    elif target_type is date:
                        value = coerce_date(value)
                    elif target_type is Decimal:
                        value = coerce_decimal(value)
        except Exception:
            # If coercion fails or type hint not found, just set the value
            pass

        super().__setattr__(name, value)


# ============================================================================
# Built-in Validators
# ============================================================================

def validate_not_empty(value: Any) -> bool:
    """
    Validate that value is not empty.

    Args:
        value: Value to validate

    Returns:
        True if valid

    Raises:
        ValidationError: If value is empty

    Example:
        >>> validate_not_empty("hello")
        True
        >>> validate_not_empty("")  # Raises ValidationError
    """
    if not value:
        raise ValueError("Value cannot be empty")
    return True


def validate_email(value: str) -> bool:
    """
    Validate email format.

    Uses a simple regex pattern for basic email validation.

    Args:
        value: Email address to validate

    Returns:
        True if valid email format

    Example:
        >>> validate_email("alice@example.com")
        True
        >>> validate_email("invalid-email")
        False
    """
    if not value:
        return False

    # Basic email regex pattern
    pattern = r'^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$'
    return bool(re.match(pattern, value))


def validate_url(value: str) -> bool:
    """
    Validate URL format.

    Checks for basic URL structure (scheme://domain).

    Args:
        value: URL to validate

    Returns:
        True if valid URL format

    Example:
        >>> validate_url("https://example.com")
        True
        >>> validate_url("not-a-url")
        False
    """
    if not value:
        return False

    # Basic URL regex pattern
    pattern = r'^https?://[a-zA-Z0-9.-]+(?:\.[a-zA-Z]{2,})?(?:/.*)?$'
    return bool(re.match(pattern, value))


def validate_min_length(value: str, min_len: int) -> bool:
    """
    Validate minimum string length.

    Args:
        value: String to validate
        min_len: Minimum required length

    Returns:
        True if valid

    Raises:
        ValueError: If string is too short

    Example:
        >>> validate_min_length("hello", 3)
        True
        >>> validate_min_length("hi", 5)  # Raises ValueError
    """
    if len(value) < min_len:
        raise ValueError(f"Value must be at least {min_len} characters long")
    return True


def validate_max_length(value: str, max_len: int) -> bool:
    """
    Validate maximum string length.

    Args:
        value: String to validate
        max_len: Maximum allowed length

    Returns:
        True if valid

    Raises:
        ValueError: If string is too long

    Example:
        >>> validate_max_length("hello", 10)
        True
        >>> validate_max_length("hello world", 5)  # Raises ValueError
    """
    if len(value) > max_len:
        raise ValueError(f"Value must be at most {max_len} characters long")
    return True


def validate_regex(value: str, pattern: Union[str, Pattern]) -> bool:
    """
    Validate string matches regex pattern.

    Args:
        value: String to validate
        pattern: Regex pattern (string or compiled Pattern)

    Returns:
        True if matches

    Raises:
        ValueError: If string doesn't match pattern

    Example:
        >>> validate_regex("abc123", r"^[a-z]+[0-9]+$")
        True
        >>> validate_regex("123abc", r"^[a-z]+[0-9]+$")  # Raises ValueError
    """
    if isinstance(pattern, str):
        pattern = re.compile(pattern)

    if not pattern.match(value):
        raise ValueError(f"Value does not match pattern: {pattern.pattern}")
    return True


def validate_range(value: Union[int, float], min_val: Union[int, float], max_val: Union[int, float]) -> bool:
    """
    Validate numeric value is within range.

    Args:
        value: Number to validate
        min_val: Minimum value (inclusive)
        max_val: Maximum value (inclusive)

    Returns:
        True if valid

    Raises:
        ValueError: If value is out of range

    Example:
        >>> validate_range(5, 1, 10)
        True
        >>> validate_range(15, 1, 10)  # Raises ValueError
    """
    if not (min_val <= value <= max_val):
        raise ValueError(f"Value must be between {min_val} and {max_val}")
    return True


def validate_min_value(value: Union[int, float], min_val: Union[int, float]) -> bool:
    """
    Validate numeric value is at least minimum.

    Args:
        value: Number to validate
        min_val: Minimum value (inclusive)

    Returns:
        True if valid

    Raises:
        ValueError: If value is too small

    Example:
        >>> validate_min_value(5, 1)
        True
        >>> validate_min_value(0, 1)  # Raises ValueError
    """
    if value < min_val:
        raise ValueError(f"Value must be at least {min_val}")
    return True


def validate_max_value(value: Union[int, float], max_val: Union[int, float]) -> bool:
    """
    Validate numeric value is at most maximum.

    Args:
        value: Number to validate
        max_val: Maximum value (inclusive)

    Returns:
        True if valid

    Raises:
        ValueError: If value is too large

    Example:
        >>> validate_max_value(5, 10)
        True
        >>> validate_max_value(15, 10)  # Raises ValueError
    """
    if value > max_val:
        raise ValueError(f"Value must be at most {max_val}")
    return True


def validate_in_list(value: Any, choices: List[Any]) -> bool:
    """
    Validate value is in list of choices.

    Args:
        value: Value to validate
        choices: List of allowed values

    Returns:
        True if valid

    Raises:
        ValueError: If value not in choices

    Example:
        >>> validate_in_list("red", ["red", "green", "blue"])
        True
        >>> validate_in_list("yellow", ["red", "green", "blue"])  # Raises ValueError
    """
    if value not in choices:
        raise ValueError(f"Value must be one of: {', '.join(map(str, choices))}")
    return True


def validate_positive(value: Union[int, float]) -> bool:
    """
    Validate numeric value is positive (> 0).

    Args:
        value: Number to validate

    Returns:
        True if valid

    Raises:
        ValueError: If value is not positive

    Example:
        >>> validate_positive(5)
        True
        >>> validate_positive(0)  # Raises ValueError
    """
    if value <= 0:
        raise ValueError("Value must be positive")
    return True


def validate_non_negative(value: Union[int, float]) -> bool:
    """
    Validate numeric value is non-negative (>= 0).

    Args:
        value: Number to validate

    Returns:
        True if valid

    Raises:
        ValueError: If value is negative

    Example:
        >>> validate_non_negative(0)
        True
        >>> validate_non_negative(-1)  # Raises ValueError
    """
    if value < 0:
        raise ValueError("Value must be non-negative")
    return True
