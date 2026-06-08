"""
Example demonstrating Pydantic-style BaseModel for data-bridge-api.

This example shows how to use BaseModel for request/response validation,
similar to Pydantic but with potential Rust-backed validation.
"""

from typing import Optional, List
from cclab.quasar import BaseModel, Field


# Define models
class Address(BaseModel):
    """User's address."""
    street: str = Field(min_length=1, description="Street address")
    city: str = Field(min_length=1, description="City name")
    zip_code: str = Field(pattern=r'^\d{5}(-\d{4})?$', description="ZIP code")
    country: str = "USA"


class User(BaseModel):
    """User model with validation."""
    name: str = Field(min_length=1, max_length=100, description="User's full name")
    age: int = Field(ge=0, le=150, description="Age in years")
    email: str = Field(pattern=r'^[\w\.-]+@[\w\.-]+\.\w+$', description="Email address")
    address: Optional[Address] = None
    tags: List[str] = Field(default_factory=list, max_items=10)
    active: bool = True


def main():
    print("=" * 60)
    print("data-bridge-api BaseModel Example")
    print("=" * 60)

    # Create a user with all fields
    print("\n1. Creating user with all fields:")
    user1 = User(
        name="John Doe",
        age=30,
        email="john@example.com",
        address=Address(
            street="123 Main St",
            city="New York",
            zip_code="10001"
        ),
        tags=["developer", "python", "rust"]
    )
    print(f"   {user1}")

    # Access fields
    print("\n2. Accessing fields:")
    print(f"   Name: {user1.name}")
    print(f"   Age: {user1.age}")
    print(f"   Email: {user1.email}")
    print(f"   City: {user1.address.city}")
    print(f"   Tags: {user1.tags}")

    # model_dump()
    print("\n3. model_dump() - Convert to dictionary:")
    data = user1.model_dump()
    print(f"   {data}")

    # model_dump(exclude_none=True)
    print("\n4. Create user without address:")
    user2 = User(name="Jane Smith", age=25, email="jane@example.com")
    print(f"   model_dump(): {user2.model_dump()}")
    print(f"   model_dump(exclude_none=True): {user2.model_dump(exclude_none=True)}")

    # model_dump(exclude_unset=True)
    print("\n5. Exclude unset fields:")
    user3 = User(name="Bob", age=40, email="bob@example.com")
    print(f"   model_dump(exclude_unset=True): {user3.model_dump(exclude_unset=True)}")

    # model_validate() - Create from dictionary
    print("\n6. model_validate() - Create from dictionary:")
    user_data = {
        "name": "Alice Johnson",
        "age": 28,
        "email": "alice@example.com",
        "address": {
            "street": "456 Oak Ave",
            "city": "Boston",
            "zip_code": "02101"
        },
        "tags": ["engineer", "golang"]
    }
    user4 = User.model_validate(user_data)
    print(f"   {user4}")

    # model_json_schema() - Get JSON Schema
    print("\n7. model_json_schema() - Get validation schema:")
    schema = User.model_json_schema()
    print(f"   Type: {schema['type']}")
    print(f"   Required fields: {schema['required']}")
    print(f"   Properties: {list(schema['properties'].keys())}")
    print(f"   Name constraints: min_length={schema['properties']['name'].get('minLength')}, "
          f"max_length={schema['properties']['name'].get('maxLength')}")
    print(f"   Age constraints: minimum={schema['properties']['age'].get('minimum')}, "
          f"maximum={schema['properties']['age'].get('maximum')}")

    # Equality comparison
    print("\n8. Equality comparison:")
    user5 = User(name="John Doe", age=30, email="john@example.com")
    user6 = User(name="John Doe", age=30, email="john@example.com")
    user7 = User(name="Jane Doe", age=30, email="jane@example.com")
    print(f"   user5 == user6: {user5 == user6}")
    print(f"   user5 == user7: {user5 == user7}")

    # Error handling - missing required field
    print("\n9. Error handling - missing required field:")
    try:
        invalid_user = User(name="Invalid")  # Missing age and email
    except ValueError as e:
        print(f"   ValueError: {e}")

    # Nested model from dict
    print("\n10. Nested model automatically created from dict:")
    user8 = User(
        name="Charlie",
        age=35,
        email="charlie@example.com",
        address={"street": "789 Pine St", "city": "Seattle", "zip_code": "98101"}
    )
    print(f"   Address type: {type(user8.address)}")
    print(f"   Address: {user8.address}")

    print("\n" + "=" * 60)
    print("All examples completed successfully!")
    print("=" * 60)


if __name__ == "__main__":
    main()
