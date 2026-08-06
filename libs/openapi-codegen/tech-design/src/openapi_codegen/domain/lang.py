from __future__ import annotations

from enum import Enum


class Lang(Enum):
    TS = "typescript"
    PY = "python"
    RUST = "rust"

    @property
    def id(self) -> str:
        return self.value


def lang_from_id(value: str) -> Lang | None:
    for lang in Lang:
        if lang.value == value:
            return lang
    return None
