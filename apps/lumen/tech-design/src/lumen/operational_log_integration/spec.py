"""Domain specification structures for #2377 operational log integration."""
from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Final

__aw_artifact_id__: Final[str] = "artifact:lumen/operational-log-integration/spec"


class FailureOwnership(Enum):
    NONE = "none"
    SHARED_NON_DOMAIN = "shared_non_domain"
    APP_DOMAIN_ONLY = "app_domain_only"
    MIXED = "mixed"


class TerminalResult(Enum):
    PASSED = "passed"
    TRACKED_SKIP = "tracked_skip"


@dataclass(frozen=True)
class GateRecord:
    commit: str = ""
    environment: str = ""
    command: str = ""
    output_summary: str = ""
    evidence_path: str = ""


@dataclass(frozen=True)
class Failure:
    ownership: FailureOwnership
    issue_ref: str = ""
    exact_reproduction: str = ""
    authoritative_existing_wi_supplied: bool = False
    authoritative_existing_wi_accepted: bool = False
