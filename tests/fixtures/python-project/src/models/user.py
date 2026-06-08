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
