from __future__ import annotations


def byte_len(value: str) -> int:
    return len(value.encode("utf-8"))


def truncate_utf8(value: str, max_bytes: int) -> str:
    raw = value.encode("utf-8")
    if len(raw) <= max_bytes:
        return value
    end = max_bytes
    while end > 0 and (raw[end] & 0b1100_0000) == 0b1000_0000:
        end -= 1
    return raw[:end].decode("utf-8")
