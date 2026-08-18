from __future__ import annotations

from dataclasses import dataclass
from enum import Enum

DEFAULT_REFILL_SECS = 60
DEFAULT_MAX_KEYS = 1024
NANOS_PER_SECOND = 1_000_000_000


class Outcome(str, Enum):
    ALLOW = "allow"
    DENY = "deny"
    BYPASS = "bypass"


@dataclass(frozen=True)
class AdmissionPolicy:
    capacity: int
    refill_window_ns: int
    max_keys: int


@dataclass(frozen=True)
class Decision:
    outcome: Outcome
    retry_after_ns: int | None


@dataclass(frozen=True)
class Event:
    route_class: str
    outcome: Outcome
    retry_after_ms: int | None


def policy_problem(policy: AdmissionPolicy) -> str | None:
    if policy.capacity <= 0:
        return "capacity must be positive"
    if policy.refill_window_ns <= 0:
        return "refill window must be positive"
    if policy.max_keys <= 0:
        return "max keys must be positive"
    return None


def is_valid_policy(policy: AdmissionPolicy) -> bool:
    return policy_problem(policy) is None


def max_credits(policy: AdmissionPolicy) -> int:
    return policy.refill_window_ns * policy.capacity


def request_cost(policy: AdmissionPolicy) -> int:
    return policy.refill_window_ns


def default_refill_window_ns() -> int:
    return DEFAULT_REFILL_SECS * NANOS_PER_SECOND


def observed_fields(event: Event) -> dict[str, object]:
    body: dict[str, object] = {
        "class": event.route_class,
        "outcome": event.outcome.value,
    }
    if event.retry_after_ms is not None:
        body["retryAfterMs"] = event.retry_after_ms
    return body
