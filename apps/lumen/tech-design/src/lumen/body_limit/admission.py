"""Admission decider for Lumen request body limits."""
from __future__ import annotations

from typing import Final

from lumen.body_limit.spec import BodyLimitSpec
from lumen.body_limit.verdict import (
    AdmittedBodyLimit,
    Rejection,
    RejectionReason,
    Verdict,
)

__aw_artifact_id__: Final[str] = "artifact:lumen/body-limit/admission"

DEFAULT_BODY_LIMIT_BYTES: Final[int] = 8388608
MIN_BODY_LIMIT_BYTES: Final[int] = 1
MAX_BODY_LIMIT_BYTES: Final[int] = 18446744073709551615


def decide_body_limit_spec(spec: BodyLimitSpec) -> Verdict:
    """Decide admission for a given BodyLimitSpec.

    Returns an AdmittedBodyLimit if omitted (using compiled default 8388608)
    or a valid u64 integer (1 to 18446744073709551615).
    Returns a Rejection for non-integer or out-of-range inputs.
    """
    val = spec.body_limit_bytes

    if val is None:
        return AdmittedBodyLimit(
            configured_limit_bytes=None,
            effective_limit_bytes=DEFAULT_BODY_LIMIT_BYTES,
        )

    if isinstance(val, bool) or not isinstance(val, int):
        return Rejection(
            reason=RejectionReason.NOT_INTEGER,
            field_path="body_limit_bytes",
        )

    if val < MIN_BODY_LIMIT_BYTES or val > MAX_BODY_LIMIT_BYTES:
        return Rejection(
            reason=RejectionReason.OUT_OF_RANGE,
            field_path="body_limit_bytes",
        )

    return AdmittedBodyLimit(
        configured_limit_bytes=val,
        effective_limit_bytes=val,
    )
