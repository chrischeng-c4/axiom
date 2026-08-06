from __future__ import annotations

from dataclasses import dataclass

TRACEPARENT_LENGTH = 55
SUPPORTED_VERSION = "00"
DEFAULT_TRACE_FLAGS = "00"
TRACE_ID_HEX_LENGTH = 32
SPAN_ID_HEX_LENGTH = 16
HYPHEN_POSITIONS = (2, 35, 52)
VERSION_SPAN = (0, 2)
TRACE_ID_SPAN = (3, 35)
PARENT_SPAN_ID_SPAN = (36, 52)
TRACE_FLAGS_SPAN = (53, 55)
LOWER_HEX_ALPHABET = "0123456789abcdef"


@dataclass(frozen=True)
class TraceParent:
    version: str
    trace_id: str
    parent_span_id: str
    trace_flags: str


@dataclass(frozen=True)
class TraceContext:
    trace_id: str
    span_id: str
    parent_span_id: str | None
    trace_flags: str


def is_lower_hex(value: str) -> bool:
    return all(c in LOWER_HEX_ALPHABET for c in value)


def is_all_zero(value: str) -> bool:
    return all(c == "0" for c in value)


def is_local_root(context: TraceContext) -> bool:
    return context.parent_span_id is None
