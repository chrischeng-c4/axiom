from __future__ import annotations

ASCII_DIGITS = "0123456789"
PLUS_SIGN = "+"


def parse_ascii_unsigned(raw: str | None) -> int | None:
    if raw is None:
        return None
    body = raw
    if body.startswith(PLUS_SIGN):
        body = body[1:]
    if body == "":
        return None
    if any(c not in ASCII_DIGITS for c in body):
        return None
    return int(body)


def parse_positive(raw: str | None) -> int | None:
    value = parse_ascii_unsigned(raw)
    if value is None:
        return None
    return value if value > 0 else None
