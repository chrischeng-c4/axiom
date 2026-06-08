"""
BaseSettings — environment-based configuration with Rust backend.

Loads values from environment variables, .env files, and secrets directories.
When the Rust backend (cclab.cclab.schema) is available, env parsing and type
coercion are performed in Rust for maximum performance. Falls back to pure
Python otherwise.

Priority order (highest to lowest):
    1. Explicit keyword arguments
    2. Environment variables
    3. .env file values
    4. Secrets files
    5. Default values

Example:
    class AppSettings(BaseSettings):
        debug: bool = False
        port: int = 8000
        host: str = "0.0.0.0"

        class Config:
            env_prefix = "APP_"
            env_file = ".env"

    settings = AppSettings()  # Reads APP_DEBUG, APP_PORT, APP_HOST
"""

import json
import os
from typing import (
    Any,
    ClassVar,
    Dict,
    List,
    Optional,
    Type,
    Union,
    get_args,
    get_origin,
    get_type_hints,
)

# Try to import Rust backend
try:
    import cclab.cclab as _native
    _load_settings = _native.schema.load_settings
    _HAS_RUST = True
except (ImportError, AttributeError):
    _HAS_RUST = False
    _load_settings = None


def _python_type_to_str(type_hint: Any) -> str:
    """Convert a Python type hint to a field type string for the Rust backend."""
    # Handle Optional[T]
    origin = get_origin(type_hint)
    if origin is Union:
        args = get_args(type_hint)
        non_none = [a for a in args if a is not type(None)]
        if len(non_none) == 1:
            return _python_type_to_str(non_none[0])

    # Handle List[T]
    if origin is list or origin is List:
        return "list"

    # Handle Dict[K, V]
    if origin is dict:
        return "dict"

    # Basic types
    if type_hint is str:
        return "str"
    if type_hint is int:
        return "int"
    if type_hint is float:
        return "float"
    if type_hint is bool:
        return "bool"
    if type_hint is list:
        return "list"
    if type_hint is dict:
        return "dict"

    # Check if it's a BaseSettings subclass (nested)
    if isinstance(type_hint, type) and issubclass(type_hint, BaseSettings):
        return "nested"

    return "str"


def _is_settings_subclass(type_hint: Any) -> bool:
    """Check if a type hint is a BaseSettings subclass."""
    origin = get_origin(type_hint)
    if origin is Union:
        args = get_args(type_hint)
        non_none = [a for a in args if a is not type(None)]
        if len(non_none) == 1:
            return _is_settings_subclass(non_none[0])
    return isinstance(type_hint, type) and issubclass(type_hint, BaseSettings)


def _coerce_value_python(value: str, target_type: Any) -> Any:
    """Pure Python fallback for type coercion."""
    origin = get_origin(target_type)
    if origin is Union:
        args = get_args(target_type)
        non_none = [a for a in args if a is not type(None)]
        if len(non_none) == 1:
            target_type = non_none[0]

    if target_type is bool:
        return value.lower() in ("true", "1", "yes", "on", "t", "y")
    if target_type is int:
        return int(value)
    if target_type is float:
        return float(value)
    if target_type is list or get_origin(target_type) is list:
        try:
            return json.loads(value)
        except json.JSONDecodeError:
            return [v.strip() for v in value.split(",") if v.strip()]
    if target_type is dict or get_origin(target_type) is dict:
        return json.loads(value)
    return value


def _load_dotenv_python(file_path: str) -> Dict[str, str]:
    """Pure Python .env file parser (fallback)."""
    result: Dict[str, str] = {}
    try:
        with open(file_path, "r", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if not line or line.startswith("#"):
                    continue
                if "=" in line:
                    key, _, value = line.partition("=")
                    key = key.strip()
                    value = value.strip()
                    if value and value[0] in ('"', "'") and value[-1] == value[0]:
                        value = value[1:-1]
                    result[key] = value
    except (IOError, OSError):
        pass
    return result


class BaseSettings:
    """Settings class that loads values from environment variables.

    Similar to Pydantic's BaseSettings, automatically loads configuration
    from environment variables, .env files, and secrets directories.

    Config Options:
        env_prefix: Prefix for environment variable names (default: "").
        env_file: Path to .env file (default: ".env", None to disable).
        env_file_encoding: Encoding for .env file (default: "utf-8").
        case_sensitive: Whether env var names are case-sensitive (default: False).
        env_nested_delimiter: Delimiter for nested fields (default: None).
            When set (e.g. "__"), DB__HOST=localhost maps to db.host.
        secrets_dir: Path to secrets directory (default: None).
            Each file's name = key, content = value.

    Example:
        class DatabaseSettings(BaseSettings):
            host: str = "localhost"
            port: int = 5432

            class Config:
                env_prefix = "DB_"

        class AppSettings(BaseSettings):
            debug: bool = False
            db: DatabaseSettings = DatabaseSettings()

            class Config:
                env_file = ".env"
                env_nested_delimiter = "__"
    """

    # Class-level metadata (populated by __init_subclass__)
    __fields_info__: ClassVar[Dict[str, Any]] = {}
    __field_types__: ClassVar[Dict[str, Any]] = {}

    class Config:
        """Default configuration for BaseSettings."""
        env_prefix: str = ""
        env_file: Optional[str] = ".env"
        env_file_encoding: str = "utf-8"
        case_sensitive: bool = False
        env_nested_delimiter: Optional[str] = None
        secrets_dir: Optional[str] = None

    def __init_subclass__(cls, **kwargs: Any) -> None:
        """Extract field metadata at class definition time."""
        super().__init_subclass__(**kwargs)
        cls._extract_fields()

    @classmethod
    def _extract_fields(cls) -> None:
        """Extract field names, types, and defaults from type hints."""
        try:
            hints = get_type_hints(cls, include_extras=True)
        except Exception:
            hints = getattr(cls, "__annotations__", {})

        fields_info: Dict[str, Any] = {}
        field_types: Dict[str, Any] = {}

        for name, type_hint in hints.items():
            if name.startswith("_"):
                continue

            # Get default value
            default = getattr(cls, name, ...)

            field_types[name] = type_hint
            fields_info[name] = {
                "type_hint": type_hint,
                "type_str": _python_type_to_str(type_hint),
                "default": default,
                "is_nested": _is_settings_subclass(type_hint),
            }

        cls.__fields_info__ = fields_info
        cls.__field_types__ = field_types

    def __init__(self, **data: Any) -> None:
        """Initialize settings from environment variables.

        Loads values from (in priority order):
        1. Explicit keyword arguments (highest priority)
        2. Environment variables
        3. .env file
        4. Secrets directory
        5. Default values (lowest priority)
        """
        config = getattr(self.__class__, "Config", BaseSettings.Config)
        env_prefix = getattr(config, "env_prefix", "")
        env_file = getattr(config, "env_file", ".env")
        case_sensitive = getattr(config, "case_sensitive", False)
        env_nested_delimiter = getattr(config, "env_nested_delimiter", None)
        secrets_dir = getattr(config, "secrets_dir", None)

        if _HAS_RUST:
            resolved = self._init_rust(
                data, env_prefix, env_file, case_sensitive,
                env_nested_delimiter, secrets_dir,
            )
        else:
            resolved = self._init_python(
                data, env_prefix, env_file, case_sensitive,
                env_nested_delimiter, secrets_dir,
            )

        # Set attributes
        for name, info in self.__fields_info__.items():
            if name in resolved:
                value = resolved[name]
                # If value is a dict and field is nested, instantiate it
                if info["is_nested"] and isinstance(value, dict):
                    nested_type = info["type_hint"]
                    origin = get_origin(nested_type)
                    if origin is Union:
                        args = get_args(nested_type)
                        non_none = [a for a in args if a is not type(None)]
                        if non_none:
                            nested_type = non_none[0]
                    value = nested_type(**value)
                setattr(self, name, value)
            elif info["default"] is not ...:
                setattr(self, name, info["default"])
            else:
                raise ValueError(f"Missing required settings field: {name}")

    def _init_rust(
        self,
        data: Dict[str, Any],
        env_prefix: str,
        env_file: Optional[str],
        case_sensitive: bool,
        env_nested_delimiter: Optional[str],
        secrets_dir: Optional[str],
    ) -> Dict[str, Any]:
        """Use Rust backend for settings loading."""
        # Build fields dict for Rust
        fields_dict: Dict[str, Any] = {}
        for name, info in self.__fields_info__.items():
            field_entry: Dict[str, Any] = {
                "type": info["type_str"],
                "is_nested": info["is_nested"],
            }
            if info["default"] is not ...:
                default = info["default"]
                if isinstance(default, BaseSettings):
                    default = default.model_dump()
                field_entry["default"] = default

            # For nested fields, include sub-field definitions
            if info["is_nested"]:
                nested_type = info["type_hint"]
                origin = get_origin(nested_type)
                if origin is Union:
                    args = get_args(nested_type)
                    non_none = [a for a in args if a is not type(None)]
                    if non_none:
                        nested_type = non_none[0]
                if hasattr(nested_type, "__fields_info__"):
                    nested_fields: Dict[str, Any] = {}
                    for nname, ninfo in nested_type.__fields_info__.items():
                        nentry: Dict[str, Any] = {"type": ninfo["type_str"]}
                        if ninfo["default"] is not ...:
                            nentry["default"] = ninfo["default"]
                        nested_fields[nname] = nentry
                    field_entry["nested_fields"] = nested_fields

            fields_dict[name] = field_entry

        config_dict = {
            "env_prefix": env_prefix,
            "env_file": env_file,
            "case_sensitive": case_sensitive,
            "env_nested_delimiter": env_nested_delimiter,
            "secrets_dir": secrets_dir,
        }

        # Convert overrides to string dict
        overrides: Dict[str, Any] = {}
        for k, v in data.items():
            if isinstance(v, BaseSettings):
                overrides[k] = json.dumps(v.model_dump())
            else:
                overrides[k] = v

        result = _load_settings(fields_dict, config_dict, overrides)
        return result

    def _init_python(
        self,
        data: Dict[str, Any],
        env_prefix: str,
        env_file: Optional[str],
        case_sensitive: bool,
        env_nested_delimiter: Optional[str],
        secrets_dir: Optional[str],
    ) -> Dict[str, Any]:
        """Pure Python fallback for settings loading."""
        # Collect env values
        env_values: Dict[str, str] = {}

        # Load secrets
        if secrets_dir and os.path.isdir(secrets_dir):
            for filename in os.listdir(secrets_dir):
                filepath = os.path.join(secrets_dir, filename)
                if os.path.isfile(filepath) and not filename.startswith("."):
                    try:
                        with open(filepath, "r") as f:
                            env_values[filename] = f.read().strip()
                    except (IOError, OSError):
                        pass

        # Load .env file
        if env_file and os.path.exists(env_file):
            env_values.update(_load_dotenv_python(env_file))

        # Load OS env vars
        env_values.update(os.environ)

        # Resolve fields
        result: Dict[str, Any] = {}
        for name, info in self.__fields_info__.items():
            if name in data:
                result[name] = data[name]
                continue

            if info["is_nested"] and env_nested_delimiter:
                nested_result = self._resolve_nested_python(
                    name, info, env_values, env_prefix,
                    case_sensitive, env_nested_delimiter,
                )
                if nested_result is not None:
                    result[name] = nested_result
                    continue

            env_name = env_prefix + name
            if not case_sensitive:
                env_name = env_name.upper()

            env_value = None
            if case_sensitive:
                env_value = env_values.get(env_name)
            else:
                for key, value in env_values.items():
                    if key.upper() == env_name:
                        env_value = value
                        break

            if env_value is not None:
                result[name] = _coerce_value_python(env_value, info["type_hint"])

        return result

    def _resolve_nested_python(
        self,
        name: str,
        info: Dict[str, Any],
        env_values: Dict[str, str],
        env_prefix: str,
        case_sensitive: bool,
        delimiter: str,
    ) -> Optional[Dict[str, Any]]:
        """Resolve nested settings fields using delimiter."""
        nested_type = info["type_hint"]
        origin = get_origin(nested_type)
        if origin is Union:
            args = get_args(nested_type)
            non_none = [a for a in args if a is not type(None)]
            if non_none:
                nested_type = non_none[0]

        if not hasattr(nested_type, "__fields_info__"):
            return None

        nested_prefix = f"{env_prefix}{name}{delimiter}"
        nested_result: Dict[str, Any] = {}

        for nname, ninfo in nested_type.__fields_info__.items():
            env_name = nested_prefix + nname
            if not case_sensitive:
                env_name = env_name.upper()

            env_value = None
            if case_sensitive:
                env_value = env_values.get(env_name)
            else:
                for key, value in env_values.items():
                    if key.upper() == env_name:
                        env_value = value
                        break

            if env_value is not None:
                nested_result[nname] = _coerce_value_python(
                    env_value, ninfo["type_hint"]
                )

        return nested_result if nested_result else None

    def model_dump(self) -> Dict[str, Any]:
        """Convert settings to a dictionary.

        Returns:
            Dictionary representation of all settings fields.
        """
        result: Dict[str, Any] = {}
        for name in self.__fields_info__:
            if hasattr(self, name):
                value = getattr(self, name)
                if isinstance(value, BaseSettings):
                    value = value.model_dump()
                result[name] = value
        return result

    def model_dump_json(self, *, indent: Optional[int] = None) -> str:
        """Serialize settings to JSON string.

        Args:
            indent: JSON indentation level.

        Returns:
            JSON string representation.
        """
        return json.dumps(self.model_dump(), indent=indent, default=str)

    def __repr__(self) -> str:
        """String representation of settings."""
        fields = ", ".join(
            f"{k}={getattr(self, k, None)!r}"
            for k in self.__fields_info__
            if hasattr(self, k)
        )
        return f"{self.__class__.__name__}({fields})"

    def __eq__(self, other: Any) -> bool:
        """Compare two settings instances for equality."""
        if not isinstance(other, self.__class__):
            return False
        return self.model_dump() == other.model_dump()
