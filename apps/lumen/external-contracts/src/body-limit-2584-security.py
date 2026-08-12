"""EC security case for #2584 -- fail-closed request-body-limit admission.

Every expected literal is EC-owned and transcribed from #2584 R1/R4 and AC3:
the optional field accepts only positive ``u64`` integers.  Invalid candidates
must yield a named rejection at ``body_limit_bytes`` with no effective fallback;
the neighbouring lower-bound value remains admitted.  This pure-model case
does not assert Kubernetes admission or rollout behavior.
"""

from __future__ import annotations

from lumen.body_limit.admission import decide_body_limit_spec
from lumen.body_limit.spec import BodyLimitSpec
from lumen.body_limit.verdict import AdmittedBodyLimit, Rejection

MINIMUM_CHECKS = 9

BODY_LIMIT_2584_SECURITY_MATRIX = (
    ("boolean_body_limit_is_rejected_as_non_integer", "body_limit_not_integer"),
    ("boolean_body_limit_refusal_names_the_field", "body_limit_bytes"),
    ("negative_body_limit_is_rejected_as_out_of_range", "body_limit_out_of_range"),
    ("negative_body_limit_refusal_names_the_field", "body_limit_bytes"),
    ("zero_body_limit_is_rejected_as_out_of_range", "body_limit_out_of_range"),
    ("zero_body_limit_never_receives_a_default_fallback", None),
    ("u64_overflow_body_limit_is_rejected_as_out_of_range", "body_limit_out_of_range"),
    ("u64_overflow_refusal_names_the_field", "body_limit_bytes"),
    ("neighbouring_positive_lower_bound_remains_admitted", 1),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def verify_body_limit_2584_security() -> dict:
    checks = []

    # 1-2. R1/R4/AC3 -- Python bool is an int subclass, so it must be supplied
    # explicitly and refused as a non-integer configuration value.
    boolean = decide_body_limit_spec(BodyLimitSpec(body_limit_bytes=True))
    obs1 = _outcome(boolean)
    exp1 = BODY_LIMIT_2584_SECURITY_MATRIX[0][1]
    checks.append({"name": BODY_LIMIT_2584_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    obs2 = boolean.field_path if isinstance(boolean, Rejection) else "admitted"
    exp2 = BODY_LIMIT_2584_SECURITY_MATRIX[1][1]
    checks.append({"name": BODY_LIMIT_2584_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3-4. R1/R4/AC3 -- a negative Python candidate is outside the unsigned
    # range and identifies the same explicit CR field.
    negative = decide_body_limit_spec(BodyLimitSpec(body_limit_bytes=-1))
    obs3 = _outcome(negative)
    exp3 = BODY_LIMIT_2584_SECURITY_MATRIX[2][1]
    checks.append({"name": BODY_LIMIT_2584_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    obs4 = negative.field_path if isinstance(negative, Rejection) else "admitted"
    exp4 = BODY_LIMIT_2584_SECURITY_MATRIX[3][1]
    checks.append({"name": BODY_LIMIT_2584_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5-6. R1/R4/AC3 -- zero is outside the documented positive range and may
    # not be silently replaced by the compiled default.
    zero = decide_body_limit_spec(BodyLimitSpec(body_limit_bytes=0))
    obs5 = _outcome(zero)
    exp5 = BODY_LIMIT_2584_SECURITY_MATRIX[4][1]
    checks.append({"name": BODY_LIMIT_2584_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    obs6 = zero.effective_limit_bytes if isinstance(zero, AdmittedBodyLimit) else None
    exp6 = BODY_LIMIT_2584_SECURITY_MATRIX[5][1]
    checks.append({"name": BODY_LIMIT_2584_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7-8. R1/R4/AC3 -- a value just above u64 is a distinct invalid boundary;
    # it must retain both the range reason and actionable field path.
    overflow = decide_body_limit_spec(BodyLimitSpec(body_limit_bytes=18446744073709551616))
    obs7 = _outcome(overflow)
    exp7 = BODY_LIMIT_2584_SECURITY_MATRIX[6][1]
    checks.append({"name": BODY_LIMIT_2584_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    obs8 = overflow.field_path if isinstance(overflow, Rejection) else "admitted"
    exp8 = BODY_LIMIT_2584_SECURITY_MATRIX[7][1]
    checks.append({"name": BODY_LIMIT_2584_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R1 -- the direct neighbour proves the range check does not reject the
    # entire field merely to satisfy its hostile-input rows.
    lower = decide_body_limit_spec(BodyLimitSpec(body_limit_bytes=1))
    obs9 = lower.effective_limit_bytes if isinstance(lower, AdmittedBodyLimit) else None
    exp9 = BODY_LIMIT_2584_SECURITY_MATRIX[8][1]
    checks.append({"name": BODY_LIMIT_2584_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    return {
        "case_id": "body-limit-2584-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
