from __future__ import annotations

from build_stamp.domain.fallback import UNKNOWN


def decode_target(value: str | None) -> str:
    """Decode target triple environment variable or UNKNOWN if None."""
    if value is None:
        return UNKNOWN
    return value
