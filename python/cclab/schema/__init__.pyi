"""Type stubs for cclab.schema"""

from typing import Any, ClassVar, Dict, Optional

class BaseSettings:
    """Settings class that loads values from environment variables.

    Similar to Pydantic's BaseSettings, automatically loads configuration
    from environment variables, .env files, and secrets directories.
    """

    __fields_info__: ClassVar[Dict[str, Any]]
    __field_types__: ClassVar[Dict[str, Any]]

    class Config:
        env_prefix: str = ""
        env_file: Optional[str] = ".env"
        env_file_encoding: str = "utf-8"
        case_sensitive: bool = False
        env_nested_delimiter: Optional[str] = None
        secrets_dir: Optional[str] = None

    def __init__(self, **data: Any) -> None: ...
    def model_dump(self) -> Dict[str, Any]: ...
    def model_dump_json(self, *, indent: Optional[int] = None) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: Any) -> bool: ...

__all__ = ["BaseSettings"]
