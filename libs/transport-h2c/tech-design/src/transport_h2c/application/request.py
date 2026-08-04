from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass

from transport_h2c.domain.errors import (
    H2cError,
    NoConnection,
    Shutdown,
    Timeout,
    is_connection_lost,
)
from transport_h2c.infrastructure.config import ManagerConfig


@dataclass(frozen=True)
class Admitted:
    pass


@dataclass(frozen=True)
class Refused:
    error: H2cError


AdmissionOutcome = Admitted | Refused


@dataclass(frozen=True)
class Delivered:
    attempts: int


@dataclass(frozen=True)
class Failed:
    error: H2cError
    attempts: int


RequestOutcome = Delivered | Failed

MAX_ATTEMPTS = 2


def admit(
    config: ManagerConfig,
    *,
    shut_down: bool,
    admission_closed: bool,
    waited_seconds: float,
) -> AdmissionOutcome:
    if shut_down:
        return Refused(Shutdown())
    if waited_seconds > config.pool_timeout_seconds:
        return Refused(Timeout(config.pool_timeout_seconds))
    if admission_closed:
        return Refused(Shutdown())
    return Admitted()


def should_retry(attempt: int, error: H2cError) -> bool:
    return attempt == 0 and is_connection_lost(error)


def resolve_request(
    authority: str,
    outcomes: Sequence[H2cError | None],
) -> RequestOutcome:
    last_error: H2cError | None = None
    consumed = 0
    for attempt in range(MAX_ATTEMPTS):
        if attempt >= len(outcomes):
            break
        consumed += 1
        outcome = outcomes[attempt]
        if outcome is None:
            return Delivered(attempts=consumed)
        last_error = outcome
        if should_retry(attempt, outcome):
            continue
        break

    if last_error is None:
        return Failed(NoConnection(authority), consumed)
    return Failed(last_error, consumed)
