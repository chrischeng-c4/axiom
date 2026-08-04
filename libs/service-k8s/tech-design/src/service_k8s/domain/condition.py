from __future__ import annotations

from dataclasses import dataclass
from enum import Enum


class ConditionStatus(Enum):
    TRUE = "True"
    FALSE = "False"
    UNKNOWN = "Unknown"

    @property
    def token(self) -> str:
        return self.value

    @staticmethod
    def from_bool(value: bool) -> ConditionStatus:
        if value:
            return ConditionStatus.TRUE
        return ConditionStatus.FALSE


@dataclass(frozen=True)
class ConditionFact:
    type_: str
    status: ConditionStatus
    reason: str
    message: str = ""


@dataclass(frozen=True)
class Condition:
    type_: str
    status: str
    reason: str
    message: str
    last_transition_time: str
    observed_generation: int | None = None

    def to_json(self) -> dict[str, object]:
        res: dict[str, object] = {
            "type": self.type_,
            "status": self.status,
            "reason": self.reason,
            "message": self.message,
            "lastTransitionTime": self.last_transition_time,
        }
        if self.observed_generation is not None:
            res["observedGeneration"] = self.observed_generation
        return res


def project(
    prior: tuple[Condition, ...],
    facts: tuple[ConditionFact, ...],
    observed_generation: int,
    now: str,
) -> tuple[Condition, ...]:
    result: list[Condition] = []
    for fact in facts:
        status_token = fact.status.token
        carried = next(
            (c for c in prior if c.type_ == fact.type_ and c.status == status_token),
            None,
        )
        last_transition_time = (
            carried.last_transition_time if carried is not None else now
        )
        result.append(
            Condition(
                type_=fact.type_,
                status=status_token,
                reason=fact.reason,
                message=fact.message,
                last_transition_time=last_transition_time,
                observed_generation=observed_generation,
            )
        )
    return tuple(result)
