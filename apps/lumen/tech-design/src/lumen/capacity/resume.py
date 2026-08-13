"""Resume decision model for phase interruption recovery."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Final, Union

from lumen.capacity.spec import ActionSpec
from lumen.capacity.verdict import CapacityReason, CapacityRejection

__aw_artifact_id__: Final[str] = "artifact:lumen/capacity-resume"


@dataclass(frozen=True)
class NextMutation:
    identifier: str
    kind: str


@dataclass(frozen=True)
class ResumeVerdict:
    next_mutation: NextMutation
    persisted_action: ActionSpec


def _get(obj: Any, key: str, default: Any = None) -> Any:
    if isinstance(obj, dict):
        return obj.get(key, default)
    return getattr(obj, key, default)


def decide_resume(
    interrupted_state: Any,
    persisted_action: Any,
    requested_actions: Any = None,
) -> Union[ResumeVerdict, CapacityRejection]:
    """Decide which mutation to resume following an interruption."""
    active_mutation = _get(interrupted_state, "active_mutation")
    if active_mutation is not None and str(active_mutation).strip() != "":
        return CapacityRejection(
            reason=CapacityReason.ANOTHER_MUTATION_ACTIVE,
            field_path="interrupted_state.active_mutation",
            message=f"another mutation {active_mutation!r} is currently active",
        )

    if persisted_action is None:
        return CapacityRejection(
            reason=CapacityReason.INVALID_INPUT,
            field_path="persisted_action",
            message="persisted_action cannot be None",
        )

    identifier = _get(persisted_action, "identifier")
    kind = _get(persisted_action, "kind", "unknown")

    if not identifier:
        return CapacityRejection(
            reason=CapacityReason.INVALID_INPUT,
            field_path="persisted_action.identifier",
            message="persisted_action must have a non-empty identifier",
        )

    next_mut = NextMutation(identifier=str(identifier), kind=str(kind))
    persisted_spec = ActionSpec(identifier=str(identifier), kind=str(kind))
    return ResumeVerdict(next_mutation=next_mut, persisted_action=persisted_spec)
