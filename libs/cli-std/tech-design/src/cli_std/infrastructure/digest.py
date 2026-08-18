from __future__ import annotations

import hashlib

from cli_std.domain.errors import DigestMismatch


def sha256_hex(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest().lower()


def verify_sha256(
    payload: bytes, expected: str
) -> None | DigestMismatch:
    actual = sha256_hex(payload)
    exp_clean = expected.strip()
    if actual == exp_clean.lower():
        return None
    return DigestMismatch(expected=exp_clean, actual=actual)
