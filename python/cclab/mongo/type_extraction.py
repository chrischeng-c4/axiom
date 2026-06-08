"""
Type extraction module for data-bridge Rust-side validation.

Converts Python type hints to BSON type descriptors that can be passed to Rust
for efficient type validation.
"""

import inspect
import typing
from typing import Any, Dict, List, Optional, Type, Tuple, get_type_hints, get_origin, get_args, Annotated
from datetime import datetime, date
from decimal import Decimal
from functools import lru_cache

# Import constraint classes for constraint extraction
try:
    from .constraints import Constraint
    CONSTRAINTS_AVAILABLE = True
except ImportError:
    Constraint = None
    CONSTRAINTS_AVAILABLE = False

# Import EmbeddedDocument for embedded document support
# Note: Pure Python, no Pydantic dependency
try:
    from .embedded import EmbeddedDocument
    EMBEDDED_DOCUMENT_AVAILABLE = True
except ImportError:
    EmbeddedDocument = None
    EMBEDDED_DOCUMENT_AVAILABLE = False


def is_annotated_type(field_type: Type) -> bool:
    """Check if a type is an Annotated type.

    Args:
        field_type: Python type annotation

    Returns:
        True if the type is Annotated[T, ...], False otherwise
    """
    return get_origin(field_type) is Annotated


def unwrap_annotated_type(field_type: Type) -> Tuple[Type, Tuple[Any, ...]]:
    """Unwrap an Annotated type to get base type and metadata.

    Args:
        field_type: Python type annotation

    Returns:
        Tuple of (base_type, metadata_tuple)
        For non-Annotated types, returns (field_type, ())
    """
    if is_annotated_type(field_type):
        args = get_args(field_type)
        base_type = args[0]
        metadata = args[1:] if len(args) > 1 else ()
        return base_type, metadata
    return field_type, ()


def extract_constraints(field_type: Type) -> Dict[str, Any]:
    """Extract constraint metadata from an Annotated type.

    Looks for Constraint instances in Annotated metadata and converts
    them to a dictionary format for Rust validation.

    Args:
        field_type: Python type annotation (may be Annotated[T, ...])

    Returns:
        Dict with constraint keys: min_length, max_length, min, max, format
        Empty dict if no constraints found
    """
    if not CONSTRAINTS_AVAILABLE or Constraint is None:
        return {}

    base_type, metadata = unwrap_annotated_type(field_type)
    constraints = {}

    for item in metadata:
        if isinstance(item, Constraint):
            # Merge constraint dict into constraints
            constraint_dict = item.to_dict()
            constraints.update(constraint_dict)

    return constraints


def _build_type_descriptor(type_name: str, constraints: Dict[str, Any], **kwargs) -> Dict[str, Any]:
    """Build type descriptor with optional constraints.

    Args:
        type_name: The BSON type name
        constraints: Constraint dictionary (may be empty)
        **kwargs: Additional fields like 'items', 'schema', 'inner'

    Returns:
        Type descriptor dict with constraints if non-empty
    """
    result = {'type': type_name}
    result.update(kwargs)
    if constraints:
        result['constraints'] = constraints
    return result


def is_embedded_document_type(field_type: Type) -> bool:
    """Check if a type is an EmbeddedDocument subclass.

    Handles Optional[EmbeddedDocument] by unwrapping Union types.
    """
    if not EMBEDDED_DOCUMENT_AVAILABLE or EmbeddedDocument is None:
        return False

    # Handle Optional[EmbeddedDocument] which is Union[EmbeddedDocument, None]
    origin = get_origin(field_type)
    if origin is typing.Union:
        args = get_args(field_type)
        # Optional[T] is Union[T, None]
        if len(args) == 2 and type(None) in args:
            inner_type = args[0] if args[1] is type(None) else args[1]
            return is_embedded_document_type(inner_type)

    try:
        return (inspect.isclass(field_type) and issubclass(field_type, EmbeddedDocument)
                and field_type is not EmbeddedDocument)
    except TypeError:
        return False


def get_embedded_document_inner_type(field_type: Type) -> Optional[Type]:
    """Get the inner EmbeddedDocument type from Optional[EmbeddedDocument]."""
    origin = get_origin(field_type)
    if origin is typing.Union:
        args = get_args(field_type)
        if len(args) == 2 and type(None) in args:
            inner_type = args[0] if args[1] is type(None) else args[1]
            if is_embedded_document_type(field_type):  # Check if it's an EmbeddedDocument
                return inner_type
    return None


def python_type_to_bson_type(field_type: Type) -> Dict[str, Any]:
    """Convert Python type to BSON type descriptor.

    Args:
        field_type: Python type annotation

    Returns:
        dict with 'type' key and optional nested type info:
        - {'type': 'string'}
        - {'type': 'string', 'constraints': {'min_length': 3}}
        - {'type': 'int64'}
        - {'type': 'array', 'items': {...}}
        - {'type': 'object', 'schema': {...}}
        - {'type': 'optional', 'inner': {...}}
    """
    from .links import Link, BackLink

    # Extract constraints from Annotated types (if any)
    constraints = extract_constraints(field_type)

    # Unwrap Annotated type to get the base type
    if is_annotated_type(field_type):
        field_type, _ = unwrap_annotated_type(field_type)

    # Get origin and args for generic types
    origin = get_origin(field_type)
    args = get_args(field_type)

    # Handle None type
    if field_type is type(None):
        return {'type': 'null'}

    # Handle Optional[T] which is Union[T, None]
    if origin is typing.Union:
        if len(args) == 2 and type(None) in args:
            # It's Optional[T]
            inner_type = args[0] if args[1] is type(None) else args[1]
            inner_descriptor = python_type_to_bson_type(inner_type)
            return {'type': 'optional', 'inner': inner_descriptor}

    # Handle List[T]
    if origin is list or origin is List:
        if args:
            items_type = python_type_to_bson_type(args[0])
            return {'type': 'array', 'items': items_type}
        else:
            # Untyped list
            return {'type': 'array', 'items': {'type': 'any'}}

    # Handle Dict[str, Any]
    if origin is dict or origin is Dict:
        # For now, treat as object with any schema
        return {'type': 'object', 'schema': {}}

    # Handle Link[T] and BackLink[T] - stored as ObjectId references
    if origin is Link or origin is BackLink:
        return {'type': 'objectid'}

    # Handle EmbeddedDocument (embedded documents)
    if EMBEDDED_DOCUMENT_AVAILABLE and is_embedded_document_type(field_type):
        # Extract nested schema recursively
        schema = extract_embedded_document_schema(field_type)
        return {'type': 'object', 'schema': schema}

    # Handle primitive types (with constraints support)
    if field_type is str:
        return _build_type_descriptor('string', constraints)
    elif field_type is int:
        return _build_type_descriptor('int64', constraints)
    elif field_type is float:
        return _build_type_descriptor('double', constraints)
    elif field_type is bool:
        return _build_type_descriptor('bool', constraints)
    elif field_type is bytes:
        return _build_type_descriptor('binary', constraints)
    elif field_type is datetime or field_type is date:
        return _build_type_descriptor('datetime', constraints)
    elif field_type is Decimal:
        return _build_type_descriptor('decimal128', constraints)

    # Check for ObjectId (handle both bson.ObjectId and cclab.ObjectId)
    type_name = getattr(field_type, '__name__', str(field_type))
    if 'ObjectId' in type_name:
        return _build_type_descriptor('objectid', constraints)

    # Fallback: any type
    return _build_type_descriptor('any', constraints)


def extract_embedded_document_schema(model_class: Type) -> Dict[str, Dict[str, Any]]:
    """Extract field schema from an EmbeddedDocument class.

    Args:
        model_class: EmbeddedDocument subclass

    Returns:
        {field_name: type_descriptor, ...}
    """
    if not EMBEDDED_DOCUMENT_AVAILABLE or not is_embedded_document_type(model_class):
        return {}

    schema = {}

    try:
        hints = get_type_hints(model_class)
    except Exception:
        # Fallback to __annotations__ if get_type_hints fails
        hints = getattr(model_class, '__annotations__', {})

    for field_name, field_type in hints.items():
        # Skip private fields
        if field_name.startswith('_'):
            continue

        # Convert to BSON type descriptor
        schema[field_name] = python_type_to_bson_type(field_type)

    return schema


# Schema cache to avoid re-extracting schemas
_schema_cache: Dict[int, Dict[str, Dict[str, Any]]] = {}


def extract_schema(cls: Type) -> Dict[str, Dict[str, Any]]:
    """Extract full field schema from a Document class.

    Schemas are cached per-class to avoid re-extraction overhead.

    Args:
        cls: Document class (subclass of cclab.Document)

    Returns:
        {field_name: type_descriptor, ...}

    Example:
        class User(Document):
            name: str
            email: str
            age: Optional[int] = None

        extract_schema(User) == {
            'name': {'type': 'string'},
            'email': {'type': 'string'},
            'age': {'type': 'optional', 'inner': {'type': 'int64'}}
        }
    """
    # Check cache first
    class_id = id(cls)
    if class_id in _schema_cache:
        return _schema_cache[class_id]

    schema = {}

    # Use _fields if available (set by DocumentMeta)
    if hasattr(cls, '_fields'):
        for field_name, field_type in cls._fields.items():
            schema[field_name] = python_type_to_bson_type(field_type)
    else:
        # Fallback: use get_type_hints
        try:
            hints = get_type_hints(cls)
        except Exception:
            hints = getattr(cls, '__annotations__', {})

        for field_name, field_type in hints.items():
            # Skip private fields and class variables
            if field_name.startswith('_'):
                continue

            schema[field_name] = python_type_to_bson_type(field_type)

    # Cache the result
    _schema_cache[class_id] = schema

    return schema
