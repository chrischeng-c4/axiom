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
