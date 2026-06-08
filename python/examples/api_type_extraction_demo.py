"""
Demo: Enhanced Type Extraction System

This example demonstrates the enhanced type extraction capabilities
supporting complex types, dataclasses, Pydantic models, and nested objects.
"""

from typing import Annotated, List, Dict, Optional, Union, Literal
from dataclasses import dataclass
from datetime import datetime
from uuid import UUID
from enum import Enum

from cclab.quasar.type_extraction import (
    extract_type_schema,
    extract_handler_meta,
    schema_to_rust_type_descriptor,
)
from cclab.quasar.types import Path, Query, Body, Header


# ============================================================================
# Example 1: Basic Types
# ============================================================================

print("=" * 70)
print("Example 1: Basic Types")
print("=" * 70)

for type_hint in [str, int, float, bool, UUID, datetime]:
    schema = extract_type_schema(type_hint)
    print(f"{type_hint.__name__:15} -> {schema}")


# ============================================================================
# Example 2: Complex Types
# ============================================================================

print("\n" + "=" * 70)
print("Example 2: Complex Types")
print("=" * 70)

complex_types = [
    ("List[str]", List[str]),
    ("Dict[str, int]", Dict[str, int]),
    ("Optional[str]", Optional[str]),
    ("Union[str, int]", Union[str, int]),
    ("List[Dict[str, List[int]]]", List[Dict[str, List[int]]]),
]

for name, type_hint in complex_types:
    schema = extract_type_schema(type_hint)
    print(f"\n{name}:")
    print(f"  {schema}")


# ============================================================================
# Example 3: Dataclasses
# ============================================================================

print("\n" + "=" * 70)
print("Example 3: Dataclasses")
print("=" * 70)

@dataclass
class Address:
    street: str
    city: str
    country: str
    postal_code: Optional[str] = None

@dataclass
class User:
    id: UUID
    username: str
    email: str
    age: int
    address: Address
    tags: List[str]
    metadata: Dict[str, str]
    created_at: datetime

schema = extract_type_schema(User)
print(f"\nUser dataclass schema:")
print(f"  Type: {schema['type']}")
print(f"  Class: {schema['class_name']}")
print(f"  Fields: {len(schema['fields'])}")
for field in schema['fields']:
    required = "required" if field.get("required", True) else "optional"
    print(f"    - {field['name']}: {field['type']} ({required})")


# ============================================================================
# Example 4: Enums and Literals
# ============================================================================

print("\n" + "=" * 70)
print("Example 4: Enums and Literals")
print("=" * 70)

class UserRole(Enum):
    ADMIN = "admin"
    USER = "user"
    GUEST = "guest"

role_schema = extract_type_schema(UserRole)
print(f"\nEnum schema:")
print(f"  Type: {role_schema['type']}")
print(f"  Values: {role_schema['values']}")
print(f"  Names: {role_schema['names']}")

status_schema = extract_type_schema(Literal["active", "inactive", "pending"])
print(f"\nLiteral schema:")
print(f"  Type: {status_schema['type']}")
print(f"  Values: {status_schema['values']}")


# ============================================================================
# Example 5: Handler Metadata Extraction
# ============================================================================

print("\n" + "=" * 70)
print("Example 5: Handler Metadata Extraction")
print("=" * 70)

@dataclass
class CreateUserRequest:
    username: str
    email: str
    password: str
    age: Optional[int] = None

@dataclass
class UserResponse:
    id: UUID
    username: str
    email: str
    created_at: datetime

async def create_user(
    user_id: Annotated[str, Path(description="User ID")],
    limit: Annotated[int, Query(default=10, ge=1, le=100, description="Page limit")],
    offset: Annotated[int, Query(default=0, ge=0, description="Page offset")],
    x_request_id: Annotated[str, Header()],
    user: Annotated[CreateUserRequest, Body(description="User creation data")],
) -> UserResponse:
    """Create a new user."""
    pass

meta = extract_handler_meta(create_user, "POST", "/users/{user_id}")

print("\nHandler: create_user")
print(f"\nPath parameters ({len(meta['validator']['path_params'])}):")
for param in meta['validator']['path_params']:
    print(f"  - {param['name']}: {param['type']}")

print(f"\nQuery parameters ({len(meta['validator']['query_params'])}):")
for param in meta['validator']['query_params']:
    default = param.get('default', 'REQUIRED')
    constraints = []
    if 'minimum' in param['type']:
        constraints.append(f"min={param['type']['minimum']}")
    if 'maximum' in param['type']:
        constraints.append(f"max={param['type']['maximum']}")
    constraint_str = f" ({', '.join(constraints)})" if constraints else ""
    print(f"  - {param['name']}: {param['type']['type']}{constraint_str} = {default}")

print(f"\nHeader parameters ({len(meta['validator']['header_params'])}):")
for param in meta['validator']['header_params']:
    print(f"  - {param['name']}: {param['type']}")

if meta['validator']['body']:
    body = meta['validator']['body']
    print(f"\nBody parameter:")
    print(f"  - {body['name']}: {body['type']['class_name']}")
    print(f"    Fields: {len(body['type']['fields'])}")
    for field in body['type']['fields']:
        print(f"      - {field['name']}: {field['type']}")

print(f"\nResponse schema:")
print(f"  Type: {meta['response_schema']['type']}")
print(f"  Class: {meta['response_schema']['class_name']}")


# ============================================================================
# Example 6: Rust TypeDescriptor Conversion
# ============================================================================

print("\n" + "=" * 70)
print("Example 6: Rust TypeDescriptor Conversion")
print("=" * 70)

test_schemas = [
    {
        "type": "string",
        "min_length": 3,
        "max_length": 50,
        "pattern": "^[a-zA-Z0-9_]+$"
    },
    {
        "type": "int",
        "minimum": 0,
        "maximum": 100
    },
    {
        "type": "list",
        "items": {"type": "string"}
    },
    {
        "type": "optional",
        "inner": {"type": "int"}
    },
    {
        "type": "object",
        "fields": [
            {"name": "id", "type": {"type": "int"}, "required": True},
            {"name": "name", "type": {"type": "string"}, "required": True},
        ]
    }
]

for i, schema in enumerate(test_schemas, 1):
    rust_descriptor = schema_to_rust_type_descriptor(schema)
    print(f"\nSchema {i}:")
    print(f"  Python: {schema}")
    print(f"  Rust:   {rust_descriptor}")


# ============================================================================
# Summary
# ============================================================================

print("\n" + "=" * 70)
print("Summary")
print("=" * 70)
print("""
The enhanced type extraction system supports:

1. Basic Types
   - str, int, float, bool, bytes
   - UUID, datetime, date, time, timedelta, Decimal

2. Complex Types
   - List[T], Dict[K, V], Tuple[T, ...], Set[T]
   - Optional[T], Union[A, B, C]
   - Nested combinations

3. Structured Types
   - Dataclasses with full field extraction
   - Pydantic models (optional)
   - Enum types
   - Literal types

4. Handler Metadata
   - Path, Query, Header, Body parameters
   - Constraint extraction (min, max, length, pattern)
   - Dependency injection support
   - Response schema extraction

5. Rust Integration
   - TypeDescriptor conversion for Rust validation
   - Maintains type safety across language boundaries
   - Supports complex nested structures

Use Cases:
- API endpoint validation
- OpenAPI schema generation
- Request/response type checking
- Automatic documentation
- Rust-powered validation engine
""")

print("\n✅ Demo completed successfully!")
