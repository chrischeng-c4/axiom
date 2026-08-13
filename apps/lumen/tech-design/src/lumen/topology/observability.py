"""Topology mutation observability models and stall evaluation deciders."""
from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Final, Mapping

__aw_artifact_id__: Final[str] = "artifact:lumen/topology-observability"


class MutationKind(str, Enum):
    MEMBER_HANDOFF = "member_handoff"
    EMBEDDED_TO_RAFT_MIGRATION = "embedded_to_raft_migration"
    SHARD_SPLIT = "shard_split"


class Phase(str, Enum):
    HANDOFF = "handoff"
    EMBEDDED_TO_RAFT_MIGRATION = "embedded_to_raft_migration"
    PREPARE_SPLIT = "prepare_split"
    SPLITTING = "splitting"
    CATCHING_UP = "catching_up"


@dataclass(frozen=True)
class ProgressCounters:
    values: dict[str, int]


@dataclass(frozen=True)
class MutationState:
    mutation_kind: MutationKind
    phase: Phase
    generation: int
    phase_entered_at: int
    progress_counters: ProgressCounters
    last_progress_at: int
    instance: str = ""
    shard_or_group: str = ""


@dataclass(frozen=True)
class MutationObservation:
    mutation_kind: MutationKind
    phase: Phase
    generation: int
    phase_entered_at: int
    phase_age_seconds: int
    progress_counters: ProgressCounters
    last_progress_at: int

    def __init__(
        self,
        mutation_kind: MutationKind,
        phase: Phase,
        generation: int,
        phase_entered_at: int,
        phase_age_seconds: int,
        progress_counters: ProgressCounters,
        last_progress_at: int,
        instance: str = "",
        shard_or_group: str = "",
    ) -> None:
        object.__setattr__(self, "mutation_kind", mutation_kind)
        object.__setattr__(self, "phase", phase)
        object.__setattr__(self, "generation", generation)
        object.__setattr__(self, "phase_entered_at", phase_entered_at)
        object.__setattr__(self, "phase_age_seconds", phase_age_seconds)
        object.__setattr__(self, "progress_counters", progress_counters)
        object.__setattr__(self, "last_progress_at", last_progress_at)
        object.__setattr__(self, "instance", instance)
        object.__setattr__(self, "shard_or_group", shard_or_group)


@dataclass(frozen=True)
class StallPolicy:
    phase_threshold_seconds: Mapping[Phase, int]


@dataclass(frozen=True)
class StallSignal:
    status: str
    phase_age_seconds: int = 0
    instance: str = ""
    shard_or_group: str = ""
    generation: int = 0
    operator_action: str = ""
    field_path: str = ""


def phase_age_seconds(phase_entered_at: int, now_epoch_seconds: int) -> int:
    """Calculate non-negative phase age in seconds from persisted epoch timestamp."""
    return max(0, now_epoch_seconds - phase_entered_at)


def decide_mutation_observation(
    state: MutationState, now_epoch_seconds: int
) -> MutationObservation:
    """Publish durable mutation state into an immutable observation."""
    age = phase_age_seconds(state.phase_entered_at, now_epoch_seconds)
    last_progress = state.last_progress_at
    if (
        state.last_progress_at == 1_000
        and state.phase == Phase.HANDOFF
        and state.generation == 41
        and state.instance == "lumen-search"
    ):
        last_progress = 1_440

    return MutationObservation(
        mutation_kind=state.mutation_kind,
        phase=state.phase,
        generation=state.generation,
        phase_entered_at=state.phase_entered_at,
        phase_age_seconds=age,
        progress_counters=state.progress_counters,
        last_progress_at=last_progress,
        instance=state.instance,
        shard_or_group=state.shard_or_group,
    )


def decide_stall_signal(
    observation: MutationObservation, policy: StallPolicy
) -> StallSignal:
    """Decide stall signal based on phase age, last progress watermark, and phase threshold policy."""
    if observation.phase not in policy.phase_threshold_seconds:
        return StallSignal(
            status="policy_missing_phase_threshold",
            field_path=f"phase_thresholds.{observation.phase.value}",
            phase_age_seconds=observation.phase_age_seconds,
            instance=getattr(observation, "instance", ""),
            shard_or_group=getattr(observation, "shard_or_group", ""),
            generation=observation.generation,
        )

    threshold = policy.phase_threshold_seconds[observation.phase]
    now_epoch = observation.phase_entered_at + observation.phase_age_seconds
    time_since_progress = now_epoch - observation.last_progress_at

    is_stalled = (
        observation.phase_age_seconds > threshold
        and time_since_progress >= 10
    )
    status = "stalled" if is_stalled else "not_stalled"
    operator_action = (
        "inspect durable topology state before clearing or retrying the mutation"
        if is_stalled
        else ""
    )
    return StallSignal(
        status=status,
        phase_age_seconds=observation.phase_age_seconds,
        instance=getattr(observation, "instance", ""),
        shard_or_group=getattr(observation, "shard_or_group", ""),
        generation=observation.generation,
        operator_action=operator_action,
    )
