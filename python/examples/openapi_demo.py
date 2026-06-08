"""
OpenAPI documentation demo for data-bridge API.

This example shows how to:
1. Create an API with automatic OpenAPI generation
2. Access Swagger UI at /docs
3. Access ReDoc at /redoc
4. Get OpenAPI JSON at /openapi.json
"""

from dataclasses import dataclass
from typing import Annotated, List, Optional

from cclab.quasar import App, Path, Query, Body


# Define data models
@dataclass
class User:
    """User model."""
    id: str
    name: str
    email: str
    age: Optional[int] = None


@dataclass
class CreateUser:
    """Create user request."""
    name: str
    email: str
    age: Optional[int] = None


@dataclass
class UpdateUser:
    """Update user request."""
    name: Optional[str] = None
    email: Optional[str] = None
    age: Optional[int] = None


# Create app with documentation enabled
app = App(
    title="User Management API",
    version="1.0.0",
    description="A simple API for managing users with full OpenAPI documentation",
    docs_url="/docs",
    redoc_url="/redoc",
    openapi_url="/openapi.json",
)


# Define routes
@app.get("/users", tags=["users"], summary="List all users")
async def list_users(
    skip: Annotated[int, Query(default=0, description="Number of users to skip")] = 0,
    limit: Annotated[int, Query(default=10, description="Maximum number of users to return")] = 10,
) -> List[User]:
    """
    Retrieve a list of users with pagination.

    - **skip**: Number of records to skip (for pagination)
    - **limit**: Maximum number of records to return
    """
    # In a real app, this would query a database
    return [
        User(id="1", name="Alice", email="alice@example.com", age=30),
        User(id="2", name="Bob", email="bob@example.com", age=25),
    ]


@app.post("/users", tags=["users"], summary="Create a new user", status_code=201)
async def create_user(
    user: Annotated[CreateUser, Body(description="User data")]
) -> User:
    """
    Create a new user with the provided information.

    - **name**: Full name of the user (required)
    - **email**: Email address (required)
    - **age**: Age in years (optional)
    """
    # In a real app, this would insert into a database
    return User(
        id="123",
        name=user.name,
        email=user.email,
        age=user.age,
    )


@app.get("/users/{user_id}", tags=["users"], summary="Get user by ID")
async def get_user(
    user_id: Annotated[str, Path(description="The ID of the user to retrieve")]
) -> User:
    """
    Retrieve a specific user by their ID.

    Returns the user object if found, or raises a 404 error.
    """
    # In a real app, this would query a database
    return User(
        id=user_id,
        name="Alice",
        email="alice@example.com",
        age=30,
    )


@app.put("/users/{user_id}", tags=["users"], summary="Update user")
async def update_user(
    user_id: Annotated[str, Path(description="The ID of the user to update")],
    user: Annotated[UpdateUser, Body(description="Updated user data")]
) -> User:
    """
    Update an existing user's information.

    Only the provided fields will be updated.
    """
    # In a real app, this would update the database
    return User(
        id=user_id,
        name=user.name or "Alice",
        email=user.email or "alice@example.com",
        age=user.age,
    )


@app.delete("/users/{user_id}", tags=["users"], summary="Delete user", status_code=204)
async def delete_user(
    user_id: Annotated[str, Path(description="The ID of the user to delete")]
) -> None:
    """
    Delete a user by their ID.

    Returns 204 No Content on success.
    """
    # In a real app, this would delete from the database
    pass


# Health check endpoint
@app.get("/health", tags=["system"], summary="Health check")
async def health_check() -> dict:
    """Check if the API is running."""
    return {"status": "healthy", "version": "1.0.0"}


# Setup documentation endpoints
app.setup_docs()


if __name__ == "__main__":
    # Print the OpenAPI spec
    import json

    print("=" * 80)
    print("OpenAPI Specification")
    print("=" * 80)
    print(json.dumps(app.openapi(), indent=2))
    print()

    print("=" * 80)
    print("Available Endpoints")
    print("=" * 80)
    print("Swagger UI: http://localhost:8000/docs")
    print("ReDoc:      http://localhost:8000/redoc")
    print("OpenAPI:    http://localhost:8000/openapi.json")
    print()

    print("Routes:")
    for route in app.routes:
        print(f"  {route.method:6} {route.path:30} {route.summary or ''}")
