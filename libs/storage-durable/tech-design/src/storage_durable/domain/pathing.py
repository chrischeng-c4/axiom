from __future__ import annotations

from typing import Final

STAGING_SUFFIX: Final[str] = ".tmp"
COMPACTION_SUFFIX: Final[str] = ".compact.tmp"

def parent_directory(path: str) -> str:
    if "/" not in path:
        return ""
    idx = path.rfind("/")
    if idx == 0:
        return "/"
    return path[:idx]

def join_path(directory: str, name: str) -> str:
    if not directory:
        return name
    if directory == "/":
        return "/" + name
    if directory.endswith("/"):
        return directory + name
    return directory + "/" + name

def staging_path(path: str) -> str:
    return path + STAGING_SUFFIX

def compaction_path(path: str) -> str:
    return path + COMPACTION_SUFFIX
