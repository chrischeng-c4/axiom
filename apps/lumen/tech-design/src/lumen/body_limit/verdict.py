"""Admitted and rejected body limit verdict models."""
from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Final, Union

__aw_artifact_id__: Final[str] = "artifact:lumen/body-limit/verdict"


class RejectionReason(Enum):
    """Closed reason vocabulary for body limit admission refusals."""

    NOT_INTEGER = "body_limit_not_integer"
    OUT_OF_RANGE = "body_limit_out_of_range"


@dataclass(frozen=True)
class AdmittedBodyLimit:
    """Admitted body limit configuration with effective and configured limits."""

    configured_limit_bytes: int | None
    effective_limit_bytes: int


@dataclass(frozen=True)
class Rejection:
    """Rejection verdict for an invalid body limit spec candidate."""

    reason: RejectionReason
    field_path: str


Verdict = Union[AdmittedBodyLimit, Rejection]
