from __future__ import annotations

import hashlib


def hex_sha256(data: bytes) -> str:
    """Lowercase hex sha256. No colons, no uppercase, always 64 chars."""
    return hashlib.sha256(data).hexdigest()
