"""EC security case for #3088 -- fail-closed immutable target staging.

Every expected value below is EC-owned and transcribed from #3088: R4/AC2
requires digest, compatibility, and readiness before authority; R5/AC5 latches
the first digest and blocks a different later observation without a replacement;
and R7/AC4 classifies every target failure as UpgradeBlocked, non-authoritative,
and limited to target-owned cleanup.
"""

from __future__ import annotations

from lumen.upgrade.admission import classify_target_failure, decide_authority, decide_target_digest
from lumen.upgrade.operation import claim_upgrade
from lumen.upgrade.spec import CurrentGeneration, RequestedImage, TargetFailureObservation, TargetObservation
from lumen.upgrade.verdict import Rejection

MINIMUM_CHECKS = 9

UPGRADE_3088_SECURITY_MATRIX = (
    ("each_missing_authority_predicate_has_its_own_refusal_reason", ("digest_not_recorded", "incompatible_target", "target_not_ready")),
    ("each_authority_refusal_names_the_missing_input", ("digest_matches", "compatible", "ready")),
    ("complete_authority_neighbour_remains_admitted", "admitted"),
    ("matching_latched_digest_is_retained", "sha256:target"),
    ("different_later_digest_is_a_blocking_mismatch", "digest_mismatch"),
    ("digest_mismatch_names_the_observed_digest", "observed_digest"),
    ("digest_mismatch_supplies_no_replacement_target_identity", None),
    ("every_target_failure_is_upgrade_blocked", ("UpgradeBlocked", "UpgradeBlocked", "UpgradeBlocked", "UpgradeBlocked", "UpgradeBlocked")),
    ("every_target_failure_stays_non_authoritative_and_cleans_only_target_resources", (("non_authoritative", "target_owned_only"), ("non_authoritative", "target_owned_only"), ("non_authoritative", "target_owned_only"), ("non_authoritative", "target_owned_only"), ("non_authoritative", "target_owned_only"))),
)


def _outcome(value) -> str:
    return value.reason.value if isinstance(value, Rejection) else "admitted"


def verify_upgrade_3088_security() -> dict:
    checks = []

    # 1-3. R4/AC2 -- each admission input is explicit. A design may not treat
    # an omitted fact as acceptable, nor may it drop one check while retaining
    # the other two.
    incomplete = (
        decide_authority(TargetObservation(digest_matches=False, compatible=True, ready=True)),
        decide_authority(TargetObservation(digest_matches=True, compatible=False, ready=True)),
        decide_authority(TargetObservation(digest_matches=True, compatible=True, ready=False)),
    )
    obs1 = tuple(_outcome(item) for item in incomplete)
    exp1 = UPGRADE_3088_SECURITY_MATRIX[0][1]
    checks.append({"name": UPGRADE_3088_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = tuple(item.field_path if isinstance(item, Rejection) else "" for item in incomplete)
    exp2 = UPGRADE_3088_SECURITY_MATRIX[1][1]
    checks.append({"name": UPGRADE_3088_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    admitted = decide_authority(TargetObservation(digest_matches=True, compatible=True, ready=True))
    obs3 = _outcome(admitted)
    exp3 = UPGRADE_3088_SECURITY_MATRIX[2][1]
    checks.append({"name": UPGRADE_3088_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    current = CurrentGeneration("generation-7", "lumen:v1", "sha256:current", "release-7")
    operation = claim_upgrade(42, current, RequestedImage("lumen:v2", "sha256:target"))

    # 4. R5/AC5 -- the already-recorded digest is the sole admissible repeat.
    retained = decide_target_digest(operation, "sha256:target")
    obs4 = retained.target_digest
    exp4 = UPGRADE_3088_SECURITY_MATRIX[3][1]
    checks.append({"name": UPGRADE_3088_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    mismatch = decide_target_digest(operation, "sha256:repointed")

    # 5-7. R5/AC5 -- a later tag resolution cannot create a second target; the
    # typed refusal says why and identifies the unsafe observed input.
    obs5 = _outcome(mismatch)
    exp5 = UPGRADE_3088_SECURITY_MATRIX[4][1]
    checks.append({"name": UPGRADE_3088_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    obs6 = mismatch.field_path if isinstance(mismatch, Rejection) else ""
    exp6 = UPGRADE_3088_SECURITY_MATRIX[5][1]
    checks.append({"name": UPGRADE_3088_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})
    obs7 = mismatch.replacement_target_identity if isinstance(mismatch, Rejection) else "unexpected_admission"
    exp7 = UPGRADE_3088_SECURITY_MATRIX[6][1]
    checks.append({"name": UPGRADE_3088_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8-9. R7/AC4 -- each target-only bad observation becomes the same blocking
    # class, never authorizes it, and scopes cleanup to resources it owns.
    failures = tuple(
        classify_target_failure(TargetFailureObservation(kind=kind))
        for kind in ("incompatible", "inconsistent_digest", "unschedulable", "crash_looping", "unready")
    )
    obs8 = tuple(item.blocker for item in failures)
    exp8 = UPGRADE_3088_SECURITY_MATRIX[7][1]
    checks.append({"name": UPGRADE_3088_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    obs9 = tuple((item.authority_state, item.cleanup_scope) for item in failures)
    exp9 = UPGRADE_3088_SECURITY_MATRIX[8][1]
    checks.append({"name": UPGRADE_3088_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    return {"case_id": "upgrade-3088-security", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
