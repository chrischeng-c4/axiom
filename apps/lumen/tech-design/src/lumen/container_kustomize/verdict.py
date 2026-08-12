"""Verdict definitions for container/Kustomize verification decisions."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Final, Optional

from lumen.container_kustomize.spec import Action

__aw_artifact_id__: Final[str] = "artifact:lumen/container-kustomize-verdict"


class Reason(Enum):
    BOUNDED_ISSUE_REQUIRED = "bounded_issue_required"
    SHARED_RERUN_REQUIRED = "shared_rerun_required"


@dataclass(frozen=True)
class Rejection:
    reason: Reason
    field_path: str


@dataclass(frozen=True)
class FailureOutcome:
    action: Action
    issue_number: Optional[int] = None


@dataclass(frozen=True)
class MixedFailureOutcome:
    shared: FailureOutcome
    domain: FailureOutcome
