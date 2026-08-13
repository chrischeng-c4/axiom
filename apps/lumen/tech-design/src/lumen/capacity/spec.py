"""Specification dataclasses and type definitions for capacity arbitration."""
from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Final, Optional

__aw_artifact_id__: Final[str] = "artifact:lumen/capacity/spec"


@dataclass(frozen=True)
class CapacitySpec:
    machine_type: str
    owner: str = "automatic"


@dataclass(frozen=True)
class MemberSpec:
    identifier: str
    healthy: bool = True
    role: str = "voter"


@dataclass(frozen=True)
class ActionSpec:
    identifier: str
    kind: str


class ProfileAvailability(str, Enum):
    AVAILABLE = "AVAILABLE"
    DRAINING = "DRAINING"
    FULL = "FULL"
    QUOTA_BLOCKED = "QUOTA_BLOCKED"
    UNSCHEDULABLE = "UNSCHEDULABLE"


@dataclass(frozen=True)
class SyntheticClock:
    now: int = 0


@dataclass(frozen=True)
class CapacitySignals:
    disk_pressure: bool = False
    read_dominated: bool = False
    write_cpu_pressure: bool = False
    compaction_cpu_pressure: bool = False
    recovery_cpu_pressure: bool = False
    memory_pressure: bool = False
    low_utilization: bool = False
    telemetry_complete: bool = True
    telemetry_fresh: bool = True
    signal_generation: int = 1
    sustained_since: int | None = None
    window_started_at: int | None = None
    within_deadband: bool = False
    cpu_p95: float = 0.0
    memory_p95: float = 0.0
    compaction_p95: float = 0.0
    recovery_p95: float = 0.0
    system_reserve_p95: float = 0.0


@dataclass(frozen=True)
class CapacityState:
    capacity_ceiling_reached: bool = False
    io_ceiling_reached: bool = False
    vertical_ceiling_reached: bool = False
    excess_read_replicas: int = 0
    current_generation: int = 1
    mutation_active: bool = False
    last_change_at: int | None = None
    converged_at: int | None = None
    automatic_change_limit_reached: bool = False


@dataclass(frozen=True)
class CapacityPolicy:
    cooldown_seconds: int = 300
    scale_out_sustained_seconds: int = 300
    scale_in_sustained_seconds: int = 1800
    headroom_percent: float = 20.0

    @classmethod
    def default(cls) -> CapacityPolicy:
        return cls()


@dataclass(frozen=True)
class CapacityInput:
    signals: CapacitySignals
    state: CapacityState
    policy: CapacityPolicy


@dataclass(frozen=True)
class ProfileCatalog:
    installed: tuple[str, ...]
    availability: dict[str, ProfileAvailability]


@dataclass(frozen=True)
class TransitionGraph:
    edges: dict[str, tuple[str, ...]]
