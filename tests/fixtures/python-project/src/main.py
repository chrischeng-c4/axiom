"""Main module that imports from local and third-party packages."""
import sys
from typing import List, Optional

# Local imports
from utils import format_name, calculate_age
from models.user import User, UserRepository

# Third-party imports (simulated)
import requests


def main() -> None:
    """Main entry point."""
    # Create a user
    user = User(
        id=1,
        name="Alice",
        email="alice@example.com",
        age=30
    )

    # Use utility functions
    formatted = format_name(user.name)
    print(f"User: {formatted}, Age: {user.age}")

    # Use repository
    repo = UserRepository()
    repo.save(user)

    # Simulate API call
    response = requests.get("https://api.example.com/users")
    print(f"API Status: {response.status_code}")


if __name__ == "__main__":
    main()
