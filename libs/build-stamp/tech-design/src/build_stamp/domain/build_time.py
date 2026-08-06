from __future__ import annotations

from build_stamp.domain.fallback import UNKNOWN


def format_built_at(epoch_seconds: int) -> str:
    """Format Unix epoch seconds as a string, or UNKNOWN if negative."""
    if epoch_seconds < 0:
        return UNKNOWN
    return str(epoch_seconds)
