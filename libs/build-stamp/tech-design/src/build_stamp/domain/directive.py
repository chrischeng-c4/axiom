from __future__ import annotations

from dataclasses import dataclass
from enum import Enum


class DirectiveKind(str, Enum):
    RUSTC_ENV = "rustc-env"
    RERUN_IF_CHANGED = "rerun-if-changed"


class DirectiveRejection(str, Enum):
    CONTROL_CHARACTER = "control_character"
    EMPTY_KEY = "empty_key"


@dataclass(frozen=True)
class Directive:
    kind: DirectiveKind
    key: str  # "" for RERUN_IF_CHANGED, which has no key
    value: str

    def render(self) -> str:
        if self.kind == DirectiveKind.RUSTC_ENV:
            return f"cargo:{self.kind.value}={self.key}={self.value}"
        elif self.kind == DirectiveKind.RERUN_IF_CHANGED:
            return f"cargo:{self.kind.value}={self.value}"
        else:
            raise ValueError(f"Unknown DirectiveKind: {self.kind}")


def sanitize_key(key: str) -> str:
    """Drop every control character (ord < 0x20 or ord == 0x7F) from key."""
    return "".join(ch for ch in key if not (ord(ch) < 0x20 or ord(ch) == 0x7F))


def make_directive(
    kind: DirectiveKind, key: str, value: str
) -> Directive | DirectiveRejection:
    """Safely construct a Directive or return a DirectiveRejection if unsafe.

    A literal 'cargo:' inside a value is harmless once newlines are impossible,
    so no substring blocklist is applied to value.
    """
    for ch in key + value:
        if ord(ch) < 0x20 or ord(ch) == 0x7F:
            return DirectiveRejection.CONTROL_CHARACTER
    if kind == DirectiveKind.RUSTC_ENV and key == "":
        return DirectiveRejection.EMPTY_KEY
    return Directive(kind=kind, key=key, value=value)
