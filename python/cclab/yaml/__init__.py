"""YAML parsing with Rust backend (serde_yaml)."""

try:
    from cclab._yaml import load, loads, dump, dumps
except ImportError:
    pass

__all__ = [
    "load",
    "loads",
    "dump",
    "dumps",
]
