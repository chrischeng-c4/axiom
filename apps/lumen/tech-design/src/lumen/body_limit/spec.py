"""Lumen request body limit specification model."""
from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Final

__aw_artifact_id__: Final[str] = "artifact:lumen/body-limit/spec"


@dataclass(frozen=True)
class BodyLimitSpec:
    """CRD specification input for Lumen request body limits."""

    body_limit_bytes: Any = None
