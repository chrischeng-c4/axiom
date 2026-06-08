"""
Constraint validators for data-bridge type validation.

These constraint classes integrate with typing.Annotated to provide
field-level validation beyond basic type checking.

Usage:
    from typing import Annotated
    from cclab.mongo import Document, MinLen, MaxLen, Min, Max, Email, Url

    class User(Document):
        name: Annotated[str, MinLen(3), MaxLen(50)]
        age: Annotated[int, Min(0), Max(150)]
        email: Annotated[str, Email()]
        website: Annotated[str, Url()]
"""

from typing import Union


class Constraint:
    """Base class for all constraints."""

    __constraint_type__: str = "base"

    def to_dict(self) -> dict:
        """Convert constraint to dictionary for Rust validation."""
        raise NotImplementedError


class MinLen(Constraint):
    """Minimum string length constraint.

    Usage:
        name: Annotated[str, MinLen(3)]
    """

    __constraint_type__ = "min_length"

    def __init__(self, length: int):
        if length < 0:
            raise ValueError("min_length must be non-negative")
        self.min_length = length

    def to_dict(self) -> dict:
        return {"min_length": self.min_length}

    def __repr__(self) -> str:
        return f"MinLen({self.min_length})"


class MaxLen(Constraint):
    """Maximum string length constraint.

    Usage:
        name: Annotated[str, MaxLen(50)]
    """

    __constraint_type__ = "max_length"

    def __init__(self, length: int):
        if length < 0:
            raise ValueError("max_length must be non-negative")
        self.max_length = length

    def to_dict(self) -> dict:
        return {"max_length": self.max_length}

    def __repr__(self) -> str:
        return f"MaxLen({self.max_length})"


class Min(Constraint):
    """Minimum numeric value constraint.

    Usage:
        age: Annotated[int, Min(0)]
        price: Annotated[float, Min(0.01)]
    """

    __constraint_type__ = "min"

    def __init__(self, value: Union[int, float]):
        self.min = value

    def to_dict(self) -> dict:
        return {"min": self.min}

    def __repr__(self) -> str:
        return f"Min({self.min})"


class Max(Constraint):
    """Maximum numeric value constraint.

    Usage:
        age: Annotated[int, Max(150)]
        discount: Annotated[float, Max(1.0)]
    """

    __constraint_type__ = "max"

    def __init__(self, value: Union[int, float]):
        self.max = value

    def to_dict(self) -> dict:
        return {"max": self.max}

    def __repr__(self) -> str:
        return f"Max({self.max})"


class Email(Constraint):
    """Email format constraint.

    Validates that a string field contains a valid email address.

    Usage:
        email: Annotated[str, Email()]
    """

    __constraint_type__ = "format"

    def __init__(self):
        self.format = "email"

    def to_dict(self) -> dict:
        return {"format": "email"}

    def __repr__(self) -> str:
        return "Email()"


class Url(Constraint):
    """URL format constraint.

    Validates that a string field contains a valid URL (http/https).

    Usage:
        website: Annotated[str, Url()]
    """

    __constraint_type__ = "format"

    def __init__(self):
        self.format = "url"

    def to_dict(self) -> dict:
        return {"format": "url"}

    def __repr__(self) -> str:
        return "Url()"


# Type alias for any constraint
AnyConstraint = Union[MinLen, MaxLen, Min, Max, Email, Url]
