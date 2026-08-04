from __future__ import annotations

from raft_runtime.domain.errors import AppliedIndexError
from raft_runtime.infrastructure.pod_name import ASCII_DIGITS


def encode_applied_index(index: int) -> bytes:
    return str(index).encode("ascii")


def decode_applied_index(raw: bytes | None) -> int:
    if raw is None:
        return 0
    try:
        text = raw.decode("ascii")
    except UnicodeDecodeError as err:
        raise AppliedIndexError(
            "Payload contains non-ASCII byte sequence"
        ) from err

    stripped = text.strip()
    if not stripped:
        return 0

    if not all(c in ASCII_DIGITS for c in stripped):
        raise AppliedIndexError(f"Invalid applied index payload: {text!r}")

    return int(stripped)
