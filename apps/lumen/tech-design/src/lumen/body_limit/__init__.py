"""Lumen request body limit tech-design model."""
from __future__ import annotations

from typing import Final

from lumen.body_limit.admission import (
    DEFAULT_BODY_LIMIT_BYTES,
    MAX_BODY_LIMIT_BYTES,
    MIN_BODY_LIMIT_BYTES,
    decide_body_limit_spec,
)
from lumen.body_limit.spec import BodyLimitSpec
from lumen.body_limit.verdict import (
    AdmittedBodyLimit,
    Rejection,
    RejectionReason,
    Verdict,
)

__aw_artifact_id__: Final[str] = "artifact:lumen/body-limit"

__all__ = [
    "DEFAULT_BODY_LIMIT_BYTES",
    "MAX_BODY_LIMIT_BYTES",
    "MIN_BODY_LIMIT_BYTES",
    "AdmittedBodyLimit",
    "BodyLimitSpec",
    "Rejection",
    "RejectionReason",
    "Verdict",
    "decide_body_limit_spec",
]
