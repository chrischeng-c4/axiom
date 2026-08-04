from __future__ import annotations


def authority_of(endpoint: str) -> str:
    rest = endpoint.removeprefix("http://")
    return rest.rstrip("/")
