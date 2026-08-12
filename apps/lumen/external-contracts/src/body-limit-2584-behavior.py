"""EC behavior case for #2584 -- configurable Lumen request-body limits.

Every expected literal is EC-owned and transcribed from #2584 R1/R2/R4 and
AC1: ``bodyLimitBytes`` is optional, accepts the sibling contract's positive
``u64`` range (1 through 18446744073709551615), and omission preserves the
compiled 8 MiB (8388608-byte) cap.  This case drives only the pure admission
model; CRD generation, ConfigMap rendering, and HTTP 413 delivery are runtime
stage concerns.
"""

from __future__ import annotations

from lumen.body_limit.admission import decide_body_limit_spec
from lumen.body_limit.spec import BodyLimitSpec
from lumen.body_limit.verdict import AdmittedBodyLimit, Rejection

MINIMUM_CHECKS = 7

BODY_LIMIT_2584_BEHAVIOR_MATRIX = (
    ("omitted_limit_preserves_the_compiled_8_mib_default", 8388608),
    ("omitted_limit_retains_no_configured_override", None),
    ("documented_positive_lower_bound_is_admitted", 1),
    ("documented_u64_upper_bound_is_admitted", 18446744073709551615),
    ("accepted_override_becomes_the_effective_limit", 16777216),
    ("accepted_override_is_retained_as_the_configured_limit", 16777216),
    ("accepted_override_returns_an_admitted_verdict", "admitted"),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def verify_body_limit_2584_behavior() -> dict:
    checks = []

    omitted = decide_body_limit_spec(BodyLimitSpec(body_limit_bytes=None))

    # 1-2. R2/AC1 -- omitted configuration is deliberately not a new default:
    # it produces today's compiled 8 MiB limit and records no override.
    obs1 = omitted.effective_limit_bytes if isinstance(omitted, AdmittedBodyLimit) else None
    exp1 = BODY_LIMIT_2584_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": BODY_LIMIT_2584_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    obs2 = omitted.configured_limit_bytes if isinstance(omitted, AdmittedBodyLimit) else "rejected"
    exp2 = BODY_LIMIT_2584_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": BODY_LIMIT_2584_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R1 -- the explicitly supplied lower boundary is admissible; this is
    # not a default-only row that could pass while ignoring the input field.
    lower = decide_body_limit_spec(BodyLimitSpec(body_limit_bytes=1))
    obs3 = lower.effective_limit_bytes if isinstance(lower, AdmittedBodyLimit) else None
    exp3 = BODY_LIMIT_2584_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": BODY_LIMIT_2584_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R1 -- the documented unsigned upper boundary is still a legitimate
    # user value, rather than an implementation-selected narrower cap.
    upper = decide_body_limit_spec(BodyLimitSpec(body_limit_bytes=18446744073709551615))
    obs4 = upper.effective_limit_bytes if isinstance(upper, AdmittedBodyLimit) else None
    exp4 = BODY_LIMIT_2584_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": BODY_LIMIT_2584_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    override = decide_body_limit_spec(BodyLimitSpec(body_limit_bytes=16777216))

    # 5-7. R1/R2 -- an explicit, valid override becomes effective, retains the
    # user selection separately from the default path, and is actually admitted.
    obs5 = override.effective_limit_bytes if isinstance(override, AdmittedBodyLimit) else None
    exp5 = BODY_LIMIT_2584_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": BODY_LIMIT_2584_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    obs6 = override.configured_limit_bytes if isinstance(override, AdmittedBodyLimit) else None
    exp6 = BODY_LIMIT_2584_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": BODY_LIMIT_2584_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    obs7 = _outcome(override)
    exp7 = BODY_LIMIT_2584_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": BODY_LIMIT_2584_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    return {
        "case_id": "body-limit-2584-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
