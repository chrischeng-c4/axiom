#!/bin/bash
# Prepare test folder for Python environment and import resolution testing
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TEST_DIR="$PROJECT_ROOT/tests/fixtures/python-project"

echo "🧹 Cleaning old test environment..."
rm -rf "$TEST_DIR"

echo "📁 Creating test project structure..."
mkdir -p "$TEST_DIR"/{src/models,tests,.venv/lib/python3.11/site-packages/requests}

# Create pyproject.toml with Argus configuration
cat > "$TEST_DIR/pyproject.toml" <<'EOF'
[project]
name = "test-project"
version = "0.1.0"
description = "Test project for Argus type inference"
requires-python = ">=3.11"

[tool.argus]
python_version = "3.11"

[tool.argus.python]
search_paths = ["./src", "./tests"]
venv_path = ".venv"
ignore_site_packages = false
EOF

# Create main.py - imports from local modules and third-party
cat > "$TEST_DIR/src/main.py" <<'EOF'
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
EOF

# Create utils.py - utility functions
cat > "$TEST_DIR/src/utils.py" <<'EOF'
"""Utility functions for the application."""
from datetime import datetime
from typing import Optional


def format_name(name: str) -> str:
    """Format a name to title case.

    Args:
        name: The name to format

    Returns:
        The formatted name
    """
    return name.title()


def calculate_age(birth_year: int) -> int:
    """Calculate age from birth year.

    Args:
        birth_year: The year of birth

    Returns:
        The calculated age
    """
    current_year = datetime.now().year
    return current_year - birth_year


def parse_email(email: str) -> tuple[str, str]:
    """Parse an email into username and domain.

    Args:
        email: The email address

    Returns:
        Tuple of (username, domain)
    """
    if "@" not in email:
        raise ValueError("Invalid email format")

    username, domain = email.split("@", 1)
    return username, domain
EOF

# Create models/__init__.py
cat > "$TEST_DIR/src/models/__init__.py" <<'EOF'
"""Models package."""
from .user import User, UserRepository

__all__ = ["User", "UserRepository"]
EOF

# Create models/user.py - data models
cat > "$TEST_DIR/src/models/user.py" <<'EOF'
"""User model and repository."""
from dataclasses import dataclass
from typing import List, Optional


@dataclass
class User:
    """User data model."""
    id: int
    name: str
    email: str
    age: int

    def is_adult(self) -> bool:
        """Check if user is an adult."""
        return self.age >= 18

    def get_display_name(self) -> str:
        """Get formatted display name."""
        return f"{self.name} (ID: {self.id})"


class UserRepository:
    """Repository for user data."""

    def __init__(self) -> None:
        self._users: List[User] = []

    def save(self, user: User) -> None:
        """Save a user to the repository."""
        self._users.append(user)

    def find_by_id(self, user_id: int) -> Optional[User]:
        """Find a user by ID."""
        for user in self._users:
            if user.id == user_id:
                return user
        return None

    def find_by_email(self, email: str) -> Optional[User]:
        """Find a user by email."""
        for user in self._users:
            if user.email == email:
                return user
        return None

    def get_all(self) -> List[User]:
        """Get all users."""
        return self._users.copy()
EOF

# Create test file
cat > "$TEST_DIR/tests/test_imports.py" <<'EOF'
"""Test imports and type checking."""
import sys
from pathlib import Path

# Add src to path for testing
sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

from models.user import User, UserRepository
from utils import format_name, calculate_age


def test_user_creation():
    """Test creating a user."""
    user = User(id=1, name="Bob", email="bob@test.com", age=25)
    assert user.name == "Bob"
    assert user.is_adult() is True


def test_repository():
    """Test user repository."""
    repo = UserRepository()
    user = User(id=1, name="Alice", email="alice@test.com", age=30)

    repo.save(user)
    found = repo.find_by_id(1)
    assert found is not None
    assert found.email == "alice@test.com"


def test_utils():
    """Test utility functions."""
    assert format_name("alice") == "Alice"
    age = calculate_age(1990)
    assert age > 0
EOF

# Create simulated third-party package (requests)
cat > "$TEST_DIR/.venv/lib/python3.11/site-packages/requests/__init__.py" <<'EOF'
"""Simulated requests library."""
from typing import Optional, Dict, Any


class Response:
    """Simulated HTTP response."""

    def __init__(self, status_code: int = 200, data: Optional[Dict[str, Any]] = None):
        self.status_code = status_code
        self.data = data or {}

    def json(self) -> Dict[str, Any]:
        """Get JSON response."""
        return self.data


def get(url: str, **kwargs: Any) -> Response:
    """Simulated GET request."""
    return Response(status_code=200, data={"message": "OK"})


def post(url: str, data: Optional[Dict[str, Any]] = None, **kwargs: Any) -> Response:
    """Simulated POST request."""
    return Response(status_code=201, data=data or {})
EOF

# Create stub file for requests (to test .pyi priority)
cat > "$TEST_DIR/.venv/lib/python3.11/site-packages/requests/__init__.pyi" <<'EOF'
"""Type stubs for requests library."""
from typing import Any, Dict, Optional

class Response:
    status_code: int
    data: Dict[str, Any]
    def json(self) -> Dict[str, Any]: ...

def get(url: str, **kwargs: Any) -> Response: ...
def post(url: str, data: Optional[Dict[str, Any]] = None, **kwargs: Any) -> Response: ...
EOF

# Create pyvenv.cfg to mark as virtual environment
cat > "$TEST_DIR/.venv/pyvenv.cfg" <<'EOF'
home = /opt/homebrew/opt/python@3.11/bin
include-system-site-packages = false
version = 3.11.7
executable = /opt/homebrew/opt/python@3.11/bin/python3.11
command = /opt/homebrew/opt/python@3.11/bin/python3.11 -m venv /Users/test/.venv
EOF

echo "✅ Test environment created at: $TEST_DIR"
echo ""
echo "📊 Structure:"
tree -L 3 "$TEST_DIR" 2>/dev/null || find "$TEST_DIR" -type f | head -20

echo ""
echo "🧪 Test environment includes:"
echo "  - pyproject.toml with [tool.argus.python] config"
echo "  - Local modules: main.py, utils.py, models/user.py"
echo "  - Virtual environment: .venv with pyvenv.cfg"
echo "  - Simulated third-party package: requests (with .pyi stubs)"
echo "  - Test file: tests/test_imports.py"
echo ""
echo "🚀 Next steps:"
echo "  1. Run: cd $TEST_DIR"
echo "  2. Test Argus environment detection"
echo "  3. Test import resolution"
echo "  4. Test MCP tools"
