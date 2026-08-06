from __future__ import annotations

from collections.abc import Callable

EnvLookup = Callable[[str], str | None]
GhFallback = Callable[[], str | None]

GH_TOKEN_KEY: str = "GH_TOKEN"
GITHUB_TOKEN_KEY: str = "GITHUB_TOKEN"
COURIER_URL_KEY: str = "AXIOM_COURIER_URL"
COURIER_TOKEN_KEY: str = "AXIOM_COURIER_TOKEN"


def resolve_github_token(env: EnvLookup, gh: GhFallback) -> str | None:
    for key in (GH_TOKEN_KEY, GITHUB_TOKEN_KEY):
        raw = env(key)
        if raw is not None:
            token = raw.strip()
            if token != "":
                return token
    return gh()


def resolve_courier_url(env: EnvLookup) -> str | None:
    raw = env(COURIER_URL_KEY)
    if raw is not None:
        val = raw.strip()
        if val != "":
            return val
    return None


def resolve_courier_token(env: EnvLookup) -> str | None:
    raw = env(COURIER_TOKEN_KEY)
    if raw is not None:
        val = raw.strip()
        if val != "":
            return val
    return None
