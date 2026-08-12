"""EC security case for #2324 -- fail-closed ownership and terminal claims.

The EC-owned literals below transcribe R2/R3/R4 and AC2/AC3.  They require an
immutable, linked, single-owner inventory; a total failure partition; and an
open refusal for every shared/non-domain failure or unbounded app-domain skip.
Each refusal checks its reason vocabulary and the named offending field, with a
neighbouring admissible input retained in the behavior case.
"""

from __future__ import annotations

from types import MappingProxyType

from lumen.capability_ownership import (
    classify_failure_slices,
    decide_terminal_result,
    ownership_inventory,
    validate_ownership_inventory,
)

MINIMUM_CHECKS = 15

CAPABILITY_2324_SECURITY_MATRIX = (
    ("inventory_mapping_is_immutable", "TypeError"),
    ("missing_owner_is_refused", "missing_canonical_owner"),
    ("missing_owner_names_the_owner_field", "http.owner"),
    ("multiple_owners_are_refused", "multiple_canonical_owners"),
    ("multiple_owners_name_the_owner_field", "http.owner"),
    ("missing_integration_seam_is_refused", "missing_integration_seam"),
    ("missing_integration_seam_names_the_seam_field", "http.integration_seam"),
    ("shared_and_domain_owner_is_refused", "conflicting_feature_ownership"),
    ("shared_and_domain_owner_names_the_owner_field", "http.owner"),
    ("unknown_failure_owner_is_refused", "unknown_failure_owner"),
    ("unknown_failure_owner_names_the_owner_field", "failures[0].owner"),
    ("shared_failure_cannot_yield_tracked_skip", "shared_non_domain_failure"),
    ("shared_failure_names_the_failure_owner_field", "failure_owners"),
    ("unbounded_domain_skip_is_refused", "missing_bounded_issue"),
    ("unbounded_domain_skip_names_the_issue_field", "issue"),
)


def _outcome(verdict) -> str:
    reason = getattr(verdict, "reason", None)
    return reason.value if reason is not None else verdict


def _field_path(verdict) -> str:
    return getattr(verdict, "field_path", "")


def _inventory_with_http(**replacement) -> MappingProxyType:
    """Construct explicit hostile validator input, never a model default."""
    inventory = dict(ownership_inventory())
    inventory["http"] = {
        "owner": "service-http",
        "capability_id": "api-cli-agent-integration",
        "integration_seam": "service_http",
        **replacement,
    }
    return MappingProxyType(inventory)


def verify_capability_2324_security() -> dict:
    checks = []

    inventory = ownership_inventory()
    try:
        inventory["forbidden"] = {"owner": "Lumen-domain"}
    except Exception as exc:  # The observable contract is the immutable mapping error.
        obs1 = type(exc).__name__
    else:
        obs1 = "mutable"
    exp1 = CAPABILITY_2324_SECURITY_MATRIX[0][1]
    checks.append({"name": CAPABILITY_2324_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    missing_owner = validate_ownership_inventory(_inventory_with_http(owner=""))

    # 2. AC2 -- an empty owner is not a harmless omitted default.
    obs2 = _outcome(missing_owner)
    exp2 = CAPABILITY_2324_SECURITY_MATRIX[1][1]
    checks.append({"name": CAPABILITY_2324_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. AC2 -- the refusal tells the producer exactly which declaration failed.
    obs3 = _field_path(missing_owner)
    exp3 = CAPABILITY_2324_SECURITY_MATRIX[2][1]
    checks.append({"name": CAPABILITY_2324_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    multiple_owners = validate_ownership_inventory(
        _inventory_with_http(owner=("service-http", "service-auth"))
    )

    # 4. AC2 -- a concern cannot evade canonical ownership by naming two libraries.
    obs4 = _outcome(multiple_owners)
    exp4 = CAPABILITY_2324_SECURITY_MATRIX[3][1]
    checks.append({"name": CAPABILITY_2324_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. AC2 -- that ambiguity is pinned to the owner declaration.
    obs5 = _field_path(multiple_owners)
    exp5 = CAPABILITY_2324_SECURITY_MATRIX[4][1]
    checks.append({"name": CAPABILITY_2324_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    missing_seam = validate_ownership_inventory(_inventory_with_http(integration_seam=""))

    # 6. R2/AC2 -- ownership without an integration seam is not source/evidence linkage.
    obs6 = _outcome(missing_seam)
    exp6 = CAPABILITY_2324_SECURITY_MATRIX[5][1]
    checks.append({"name": CAPABILITY_2324_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R2/AC2 -- the missing linkage is field-addressable.
    obs7 = _field_path(missing_seam)
    exp7 = CAPABILITY_2324_SECURITY_MATRIX[6][1]
    checks.append({"name": CAPABILITY_2324_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    conflicting_owner = validate_ownership_inventory(
        _inventory_with_http(owner=("service-http", "Lumen-domain"))
    )

    # 8. R3/AC2 -- shared work cannot be relabelled domain work to unlock a skip.
    obs8 = _outcome(conflicting_owner)
    exp8 = CAPABILITY_2324_SECURITY_MATRIX[7][1]
    checks.append({"name": CAPABILITY_2324_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R3/AC2 -- the conflicting classification names its owner declaration.
    obs9 = _field_path(conflicting_owner)
    exp9 = CAPABILITY_2324_SECURITY_MATRIX[8][1]
    checks.append({"name": CAPABILITY_2324_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    unknown_owner = classify_failure_slices(
        ({"concern": "http", "owner": "unowned-library"},)
    )

    # 10. R3/R4 -- a partition must fail closed rather than silently choosing a side.
    obs10 = _outcome(unknown_owner)
    exp10 = CAPABILITY_2324_SECURITY_MATRIX[9][1]
    checks.append({"name": CAPABILITY_2324_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. R3/R4 -- and identify the owner value that was not classifiable.
    obs11 = _field_path(unknown_owner)
    exp11 = CAPABILITY_2324_SECURITY_MATRIX[10][1]
    checks.append({"name": CAPABILITY_2324_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    shared_failure = decide_terminal_result(("service-http",), 2324)

    # 12. R3/AC3 -- even a real issue cannot turn a shared failure into a skip.
    obs12 = _outcome(shared_failure)
    exp12 = CAPABILITY_2324_SECURITY_MATRIX[11][1]
    checks.append({"name": CAPABILITY_2324_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13. R3/AC3 -- the terminal refusal points at the forbidden owner input.
    obs13 = _field_path(shared_failure)
    exp13 = CAPABILITY_2324_SECURITY_MATRIX[12][1]
    checks.append({"name": CAPABILITY_2324_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    missing_issue = decide_terminal_result(("Lumen-domain",), None)

    # 14. AC3 -- app-only is not enough; it needs a validated bounded issue.
    obs14 = _outcome(missing_issue)
    exp14 = CAPABILITY_2324_SECURITY_MATRIX[13][1]
    checks.append({"name": CAPABILITY_2324_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    # 15. AC3 -- the issue linkage is a concrete refusal field, never a bare warning.
    obs15 = _field_path(missing_issue)
    exp15 = CAPABILITY_2324_SECURITY_MATRIX[14][1]
    checks.append({"name": CAPABILITY_2324_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    return {
        "case_id": "capability-2324-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
