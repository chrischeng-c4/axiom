"""Verdict models and ActionKind enumeration for capacity decisions."""
from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Final

from lumen.capacity.spec import MachineFamily, PdSsdDisposition, StorageClass

__aw_artifact_id__: Final[str] = "artifact:lumen/capacity/verdict"


class GuidanceKind(str, Enum):
    ADMITTED = "admitted"
    REJECTED = "rejected"


class RejectionReason(str, Enum):
    PRICE_DATA_NOT_ALLOWED = "price_data_not_allowed"
    AUTOMATIC_STORAGE_CLASS_MIGRATION_UNSUPPORTED = (
        "automatic_storage_class_migration_unsupported"
    )
    LEGACY_WHOLE_STATE_JSON = "legacy_whole_state_json"
    MISSING_STORAGE_EVIDENCE_PREREQUISITE = (
        "missing_storage_evidence_prerequisite"
    )
    N2_EVIDENCE_NOT_QUALIFIED = "n2_evidence_not_qualified"


@dataclass(frozen=True)
class AdmittedGuidance:
    machine_family: MachineFamily
    storage_class: StorageClass
    pd_ssd_disposition: PdSsdDisposition = PdSsdDisposition.INITIAL_ONLY_FUTURE
    kind: GuidanceKind = GuidanceKind.ADMITTED


@dataclass(frozen=True)
class Rejection:
    reason: RejectionReason
    field_path: str
    threshold: None = None
    machine_recommendation: None = None
    kind: GuidanceKind = GuidanceKind.REJECTED


class CapacityReason(str, Enum):
    ANOTHER_MUTATION_ACTIVE = "another_mutation_active"
    STABLE_WINDOW_NOT_ELAPSED = "stable-window-not-elapsed"
    INSUFFICIENT_HEADROOM = "insufficient-headroom"
    POOL_MAXIMUM_EXCEEDED = "pool-maximum-exceeded"
    COOLDOWN_ACTIVE = "cooldown-active"
    DATA_MEMBER_NODE_CONFLICT = "data_member_node_conflict"
    CAPACITY_BLOCKED = "CapacityBlocked"
    INVALID_INPUT = "invalid_input"
    COMPETING_MUTATION = "competing_mutation"
    INTERRUPTED_MUTATION_MISMATCH = "interrupted_mutation_mismatch"


class ReclaimAction(str, Enum):
    RECLAIM = "reclaim"
    RETAIN = "retain"


class ReapplyAction(str, Enum):
    NO_OP = "no_op"
    REAPPLY = "reapply"


class TransitionKind(str, Enum):
    ADMITTED = "admitted"
    REJECTED = "rejected"


@dataclass(frozen=True)
class CapacityRejection:
    reason: CapacityReason
    field_path: str
    message: str

    @property
    def kind(self) -> TransitionKind:
        return TransitionKind.REJECTED


class ActionKind(str, Enum):
    HOLD = "HOLD"
    PVC_GROW = "PVC_GROW"
    SPLIT = "SPLIT"
    READ_REPLICA = "READ_REPLICA"
    MACHINE_UPGRADE = "MACHINE_UPGRADE"
    HIGHMEM_UPGRADE = "HIGHMEM_UPGRADE"
    READ_REPLICA_REMOVE = "READ_REPLICA_REMOVE"
    MACHINE_DOWNGRADE = "MACHINE_DOWNGRADE"


@dataclass(frozen=True)
class CapacityAction:
    kind: ActionKind
    target: str | None = None


@dataclass(frozen=True)
class CapacityDecision:
    action: CapacityAction
    reason: str = "ok"
    field_path: str = ""


@dataclass(frozen=True)
class DowngradeVerdict:
    action: CapacityAction
    failing_constraint: str | None = None
    reason: str = "ok"
    field_path: str = ""


@dataclass(frozen=True)
class ProfileSelection:
    profile: str | None
    reason: str
    field_path: str = ""
