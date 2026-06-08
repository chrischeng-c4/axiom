"""
cclab.schema - Settings management module

A high-performance settings management library that replaces pydantic-settings.
Uses Rust for environment variable parsing and type coercion.

Example:
    from cclab.schema import BaseSettings

    class DatabaseSettings(BaseSettings):
        host: str = "localhost"
        port: int = 5432
        name: str = "mydb"
        password: str

        class Config:
            env_prefix = "DB_"

    class AppSettings(BaseSettings):
        debug: bool = False
        db: DatabaseSettings = DatabaseSettings()
        allowed_hosts: list = ["*"]

        class Config:
            env_file = ".env"
            env_nested_delimiter = "__"

    # Reads from: DB_HOST, DB_PORT, DB_NAME, DB_PASSWORD, DEBUG, ALLOWED_HOSTS
    settings = AppSettings()
"""

# Prefer pydantic-settings BaseSettings when available (full env parsing + Field support)
try:
    from pydantic_settings import BaseSettings
except ImportError:
    from .settings import BaseSettings  # cclab's own fallback

# Re-export pydantic types for cclab.schema consumers
try:
    from pydantic import BaseModel, Field, ConfigDict
    from pydantic import field_validator, model_validator, validator
    from pydantic import TypeAdapter, ValidationError
except ImportError:
    pass

__all__ = [
    "BaseSettings",
    "BaseModel",
    "Field",
    "ConfigDict",
    "field_validator",
    "model_validator",
    "validator",
    "TypeAdapter",
    "ValidationError",
]
