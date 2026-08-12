"""Pure design model for #2324 -- Lumen capability contracts and shared ownership."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from types import MappingProxyType
from typing import Any, Final

__aw_artifact_id__: Final[str] = "artifact:lumen/capability-contracts-and-shared-ownership"


class Reason(str, Enum):
    ADMITTED = "admitted"
    MISSING_CANONICAL_OWNER = "missing_canonical_owner"
    MULTIPLE_CANONICAL_OWNERS = "multiple_canonical_owners"
    MISSING_INTEGRATION_SEAM = "missing_integration_seam"
    CONFLICTING_FEATURE_OWNERSHIP = "conflicting_feature_ownership"
    UNKNOWN_FAILURE_OWNER = "unknown_failure_owner"
    SHARED_NON_DOMAIN_FAILURE = "shared_non_domain_failure"
    MISSING_BOUNDED_ISSUE = "missing_bounded_issue"


@dataclass(frozen=True)
class GateSequencePlan:
    commands: tuple[str, ...]
    missing_live_command_policy: str


@dataclass(frozen=True)
class OwnershipVerdict:
    reason: Reason
    field_path: str = ""


@dataclass(frozen=True)
class FailureSlices:
    shared_non_domain: tuple[str, ...]
    lumen_domain: tuple[str, ...]
    shared_action: str = "repair_and_rerun"


REQUIRED_CONCERNS: Final[tuple[str, ...]] = (
    "auth",
    "cli",
    "http",
    "index-storage-policy",
    "kubernetes-render",
    "lumen-crd-reshard-policy",
    "observability",
    "peer-identity",
    "raft-host",
    "search-planner",
)

RAW_INVENTORY: Final[dict[str, dict[str, str]]] = {
    "auth": {
        "owner": "service-auth",
        "capability_id": "security-hardening",
        "integration_seam": "service_auth",
    },
    "cli": {
        "owner": "cli-std",
        "capability_id": "api-cli-agent-integration",
        "integration_seam": "cli_std",
    },
    "http": {
        "owner": "service-http",
        "capability_id": "api-cli-agent-integration",
        "integration_seam": "service_http",
    },
    "index-storage-policy": {
        "owner": "Lumen-domain",
        "capability_id": "indexing",
        "integration_seam": "lumen.index_storage_policy",
    },
    "kubernetes-render": {
        "owner": "service-k8s",
        "capability_id": "kubernetes-native-deployment",
        "integration_seam": "service_k8s",
    },
    "lumen-crd-reshard-policy": {
        "owner": "Lumen-domain",
        "capability_id": "kubernetes-native-deployment",
        "integration_seam": "lumen.operator.reshard_policy",
    },
    "observability": {
        "owner": "service-observability",
        "capability_id": "operations-observability",
        "integration_seam": "service_observability",
    },
    "peer-identity": {
        "owner": "peer-tls",
        "capability_id": "security-hardening",
        "integration_seam": "peer_tls",
    },
    "raft-host": {
        "owner": "raft-runtime",
        "capability_id": "scaling-availability",
        "integration_seam": "raft_runtime",
    },
    "search-planner": {
        "owner": "Lumen-domain",
        "capability_id": "querying",
        "integration_seam": "lumen.search_planner",
    },
}

KNOWN_VALID_OWNERS: Final[set[str]] = {
    "service-auth",
    "cli-std",
    "service-http",
    "service-k8s",
    "service-observability",
    "peer-tls",
    "raft-runtime",
    "Lumen-domain",
}


def required_gate_sequence() -> GateSequencePlan:
    """Return the ordered reusable gates and missing live command policy (R1)."""
    return GateSequencePlan(
        commands=(
            "aw capability check --project lumen --verify --write-evidence",
            "aw health --project lumen full --verify-traceability --verify-cb --verify-cold --verify-tests",
        ),
        missing_live_command_policy="require_shared_or_thin_app_wrapper_before_passed",
    )


def ownership_inventory() -> MappingProxyType:
    """Return an immutable mapping of checked concerns to canonical owners/linkages (R2)."""
    return MappingProxyType(RAW_INVENTORY)


def validate_ownership_inventory(inventory: Any) -> OwnershipVerdict:
    """Validate that every concern has exactly one canonical owner and declared seam (AC2)."""
    if not isinstance(inventory, (dict, MappingProxyType)):
        return OwnershipVerdict(
            reason=Reason.MISSING_CANONICAL_OWNER,
            field_path="inventory",
        )

    for concern in REQUIRED_CONCERNS:
        if concern not in inventory:
            return OwnershipVerdict(
                reason=Reason.MISSING_CANONICAL_OWNER,
                field_path=f"{concern}.owner",
            )

    for concern, record in inventory.items():
        if not isinstance(record, (dict, MappingProxyType)):
            return OwnershipVerdict(
                reason=Reason.MISSING_CANONICAL_OWNER,
                field_path=f"{concern}.owner",
            )

        owner = record.get("owner")
        if owner is None or owner == "":
            return OwnershipVerdict(
                reason=Reason.MISSING_CANONICAL_OWNER,
                field_path=f"{concern}.owner",
            )

        if isinstance(owner, (tuple, list, set)):
            if len(owner) == 0:
                return OwnershipVerdict(
                    reason=Reason.MISSING_CANONICAL_OWNER,
                    field_path=f"{concern}.owner",
                )
            owners_set = set(owner)
            if len(owners_set) > 1:
                has_domain = "Lumen-domain" in owners_set
                has_shared = any(o != "Lumen-domain" for o in owners_set)
                if has_domain and has_shared:
                    return OwnershipVerdict(
                        reason=Reason.CONFLICTING_FEATURE_OWNERSHIP,
                        field_path=f"{concern}.owner",
                    )
                return OwnershipVerdict(
                    reason=Reason.MULTIPLE_CANONICAL_OWNERS,
                    field_path=f"{concern}.owner",
                )
            single_owner = list(owners_set)[0]
            if single_owner not in KNOWN_VALID_OWNERS:
                return OwnershipVerdict(
                    reason=Reason.MISSING_CANONICAL_OWNER,
                    field_path=f"{concern}.owner",
                )
        elif isinstance(owner, str):
            if owner not in KNOWN_VALID_OWNERS:
                return OwnershipVerdict(
                    reason=Reason.MISSING_CANONICAL_OWNER,
                    field_path=f"{concern}.owner",
                )
        else:
            return OwnershipVerdict(
                reason=Reason.MISSING_CANONICAL_OWNER,
                field_path=f"{concern}.owner",
            )

        seam = record.get("integration_seam")
        if seam is None or seam == "":
            return OwnershipVerdict(
                reason=Reason.MISSING_INTEGRATION_SEAM,
                field_path=f"{concern}.integration_seam",
            )

    return OwnershipVerdict(reason=Reason.ADMITTED, field_path="")


def classify_failure_slices(failures: Any) -> FailureSlices | OwnershipVerdict:
    """Partition failures into shared_non_domain and lumen_domain slices (R4)."""
    if failures is None:
        return FailureSlices(shared_non_domain=(), lumen_domain=())

    shared_concerns: list[str] = []
    domain_concerns: list[str] = []

    for index, item in enumerate(failures):
        if isinstance(item, dict):
            owner = item.get("owner")
            concern = item.get("concern", item.get("name", str(owner)))
        elif isinstance(item, (tuple, list)) and len(item) == 2:
            concern, owner = item
        elif isinstance(item, str):
            if item in RAW_INVENTORY:
                concern = item
                owner = RAW_INVENTORY[item]["owner"]
            else:
                concern = item
                owner = item
        else:
            return OwnershipVerdict(
                reason=Reason.UNKNOWN_FAILURE_OWNER,
                field_path=f"failures[{index}].owner",
            )

        if owner not in KNOWN_VALID_OWNERS:
            return OwnershipVerdict(
                reason=Reason.UNKNOWN_FAILURE_OWNER,
                field_path=f"failures[{index}].owner",
            )

        if owner == "Lumen-domain":
            domain_concerns.append(str(concern))
        else:
            shared_concerns.append(str(concern))

    return FailureSlices(
        shared_non_domain=tuple(shared_concerns),
        lumen_domain=tuple(domain_concerns),
    )


def decide_terminal_result(failure_owners: Any, issue: Any) -> str | OwnershipVerdict:
    """Decide terminal result: passed, tracked_skip(#issue), or refusal (R3, AC3)."""
    if failure_owners is None:
        owners = ()
    else:
        owners = tuple(failure_owners)

    if not owners:
        return "passed"

    extracted_owners: list[str] = []
    for item in owners:
        if isinstance(item, dict):
            extracted_owners.append(str(item.get("owner", "")))
        else:
            extracted_owners.append(str(item))

    has_shared = any(o != "Lumen-domain" for o in extracted_owners)
    if has_shared:
        return OwnershipVerdict(
            reason=Reason.SHARED_NON_DOMAIN_FAILURE,
            field_path="failure_owners",
        )

    if issue is None or issue == "" or issue == 0:
        return OwnershipVerdict(
            reason=Reason.MISSING_BOUNDED_ISSUE,
            field_path="issue",
        )

    if isinstance(issue, int):
        issue_str = f"#{issue}"
    else:
        s = str(issue).strip()
        issue_str = s if s.startswith("#") else f"#{s}"

    return f"tracked_skip({issue_str})"
