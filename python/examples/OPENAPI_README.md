# OpenAPI Documentation Guide

This guide shows how to use the OpenAPI documentation features in cclab API.

## Quick Start

```python
from cclab.quasar import App

app = App(
    title="My API",
    version="1.0.0",
    description="API description",
    docs_url="/docs",           # Swagger UI
    redoc_url="/redoc",          # ReDoc
    openapi_url="/openapi.json", # OpenAPI spec
)

# Define your routes...
@app.get("/users")
async def list_users():
    return []

# Setup documentation endpoints
app.setup_docs()
```

## Accessing Documentation

Once your app is running, you can access:

- **Swagger UI**: `http://localhost:8000/docs`
  - Interactive API documentation
  - Try out API endpoints directly
  - See request/response schemas

- **ReDoc**: `http://localhost:8000/redoc`
  - Clean, three-panel documentation
  - Search functionality
  - Download OpenAPI spec

- **OpenAPI JSON**: `http://localhost:8000/openapi.json`
  - Raw OpenAPI 3.1 specification
  - Use with code generators
  - Import into Postman, Insomnia, etc.

## Type Annotations

The OpenAPI spec is automatically generated from your type hints:

```python
from typing import Annotated, List, Optional
from dataclasses import dataclass
from cclab.quasar import Path, Query, Body

@dataclass
class User:
    id: str
    name: str
    email: str

@app.get("/users/{user_id}")
async def get_user(
    user_id: Annotated[str, Path(description="User ID")]
) -> User:
    """Get a user by ID."""
    pass

@app.get("/users")
async def list_users(
    skip: Annotated[int, Query(default=0, description="Skip N items")] = 0,
    limit: Annotated[int, Query(default=10, description="Max items")] = 10,
) -> List[User]:
    """List users with pagination."""
    pass

@app.post("/users")
async def create_user(
    user: Annotated[User, Body(description="User to create")]
) -> User:
    """Create a new user."""
    pass
```

## Parameter Types

### Path Parameters

```python
@app.get("/items/{item_id}")
async def get_item(
    item_id: Annotated[str, Path(description="Item ID")]
):
    pass
```

### Query Parameters

```python
@app.get("/items")
async def list_items(
    skip: Annotated[int, Query(default=0, ge=0)] = 0,
    limit: Annotated[int, Query(default=10, le=100)] = 10,
    search: Annotated[Optional[str], Query(description="Search term")] = None,
):
    pass
```

### Request Body

```python
@dataclass
class CreateItem:
    name: str
    price: float

@app.post("/items")
async def create_item(
    item: Annotated[CreateItem, Body(description="Item data")]
):
    pass
```

### Headers

```python
@app.get("/protected")
async def protected_endpoint(
    authorization: Annotated[str, Header(alias="Authorization")]
):
    pass
```

## Route Metadata

Add metadata to improve documentation:

```python
@app.get(
    "/users",
    summary="List all users",
    description="Retrieve a paginated list of users",
    tags=["users"],
    deprecated=False,
)
async def list_users() -> List[User]:
    """
    List all users in the system.

    This endpoint supports pagination via skip/limit parameters.
    """
    pass
```

## Response Schemas

Return types are automatically documented:

```python
@dataclass
class User:
    id: str
    name: str

# Single object
@app.get("/users/{user_id}")
async def get_user(user_id: str) -> User:
    pass

# List of objects
@app.get("/users")
async def list_users() -> List[User]:
    pass

# Optional response
@app.get("/users/{user_id}")
async def get_user(user_id: str) -> Optional[User]:
    pass

# Dict response
@app.get("/status")
async def get_status() -> dict:
    pass
```

## Complex Types

Nested dataclasses are fully supported:

```python
@dataclass
class Address:
    street: str
    city: str
    country: str

@dataclass
class User:
    id: str
    name: str
    address: Address

@app.post("/users")
async def create_user(user: Annotated[User, Body()]) -> User:
    pass
```

## Programmatic Access

Get the OpenAPI spec programmatically:

```python
# Get as dict
spec = app.openapi()

# Get as JSON string
json_spec = app.openapi_json()

# Save to file
import json
with open("openapi.json", "w") as f:
    json.dump(app.openapi(), f, indent=2)
```

## Customization

### Disable Documentation

```python
app = App(
    title="My API",
    version="1.0.0",
    docs_url=None,      # Disable Swagger UI
    redoc_url=None,     # Disable ReDoc
    openapi_url=None,   # Disable OpenAPI endpoint
)
```

### Custom URLs

```python
app = App(
    title="My API",
    version="1.0.0",
    docs_url="/api/docs",
    redoc_url="/api/redoc",
    openapi_url="/api/schema.json",
)
```

### Add Servers

```python
from cclab.quasar.openapi import generate_openapi

spec = generate_openapi(
    title="My API",
    version="1.0.0",
    servers=[
        {"url": "https://api.example.com", "description": "Production"},
        {"url": "https://staging.example.com", "description": "Staging"},
    ],
)
```

### Add Tags

```python
spec = generate_openapi(
    title="My API",
    version="1.0.0",
    tags=[
        {"name": "users", "description": "User management"},
        {"name": "items", "description": "Item management"},
    ],
)
```

## Best Practices

1. **Use descriptive docstrings**: They appear in the documentation
2. **Add parameter descriptions**: Use `Query(description="...")`
3. **Type everything**: Better types = better documentation
4. **Use tags**: Organize endpoints logically
5. **Add examples**: Use `example=` in parameter definitions
6. **Set status codes**: Use `status_code=201` for creation, etc.

## Example

See `examples/openapi_demo.py` for a complete working example.

## Features

- ✅ OpenAPI 3.1 compliance
- ✅ Automatic schema generation from type hints
- ✅ Support for dataclasses and Pydantic models
- ✅ Swagger UI integration
- ✅ ReDoc integration
- ✅ Path, Query, Body, Header parameters
- ✅ Request/response schemas
- ✅ Nested objects
- ✅ Optional and Union types
- ✅ Lists and complex types
- ✅ Parameter constraints (min, max, pattern, etc.)
- ✅ Tags and operation metadata
- ✅ Custom status codes
