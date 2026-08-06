from __future__ import annotations

from dataclasses import dataclass
from enum import Enum

BASELINE_METRIC = "app"
FALLBACK_TOKEN = "phase"
REPLACEMENT_CHAR = "_"
EXTRA_TOKEN_CHARS = "_-."
METRIC_SEPARATOR = ", "
DURATION_PARAM = ";dur="


class Disclosure(str, Enum):
    TOTAL_ONLY = "total-only"
    FULL = "full"


DEFAULT_DISCLOSURE = Disclosure.TOTAL_ONLY


@dataclass(frozen=True)
class Phase:
    name: str
    duration_ns: int


def reveals_phases(disclosure: Disclosure) -> bool:
    return disclosure is Disclosure.FULL


def drains_phases(disclosure: Disclosure) -> bool:
    return disclosure is Disclosure.FULL
