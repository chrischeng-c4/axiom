from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class Connect:
    authority: str
    message: str


@dataclass(frozen=True)
class H2Protocol:
    go_away: bool = False
    io: bool = False
    reset: bool = False


@dataclass(frozen=True)
class NoConnection:
    authority: str


@dataclass(frozen=True)
class Timeout:
    after_seconds: float


@dataclass(frozen=True)
class Shutdown:
    pass


@dataclass(frozen=True)
class InvalidRequest:
    message: str


H2cError = Connect | H2Protocol | NoConnection | Timeout | Shutdown | InvalidRequest


def is_connection_lost(error: H2cError) -> bool:
    if isinstance(error, H2Protocol):
        return error.go_away or error.io or error.reset
    if isinstance(error, (Connect, NoConnection)):
        return True
    return False
