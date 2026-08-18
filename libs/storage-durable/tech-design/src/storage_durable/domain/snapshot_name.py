from __future__ import annotations

from dataclasses import dataclass
from typing import Iterable

@dataclass(frozen=True)
class SnapshotName:
    prefix: str
    seq: int
    extension: str

def render_name(prefix: str, seq: int, extension: str) -> str:
    return f"{prefix}-{seq}.{extension}"

def parse_name(name: str, prefix: str, extension: str) -> int | None:
    required_suffix = "." + extension
    if not name.endswith(required_suffix):
        return None
    stem = name[: len(name) - len(required_suffix)]
    required_prefix = prefix + "-"
    if not stem.startswith(required_prefix):
        return None
    digits = stem[len(required_prefix) :]
    if digits == "":
        return None
    if not all(ch in "0123456789" for ch in digits):
        return None
    if len(digits) > 1 and digits[0] == "0":
        return None
    return int(digits)

def order_by_sequence(entries: Iterable[tuple[int, str]]) -> tuple[tuple[int, str], ...]:
    return tuple(sorted(entries, key=lambda item: item[0]))
