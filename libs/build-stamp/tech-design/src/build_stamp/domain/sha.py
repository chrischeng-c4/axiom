from __future__ import annotations


def decode_short_sha(success: bool, stdout: bytes) -> str | None:
    """Decode git rev-parse stdout into a short sha string or None if absent/invalid."""
    if not success:
        return None
    text = stdout.decode("utf-8", errors="replace")
    trimmed = text.strip()
    if not trimmed:
        return None
    return trimmed
