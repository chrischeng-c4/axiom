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
