"""EC security case for #2533 -- fail-closed topology observability.

The EC owns the literal boundary values below.  It refuses negative phase age,
does not overclaim a stall at the policy boundary, requires each phase to have
its own policy entry, and prevents complete-backup status from impersonating a
topology mutation.  Imports are intentionally fail-closed until the pure design
lands.
"""

from __future__ import annotations

from dataclasses import FrozenInstanceError

from lumen.topology.backup_status import BackupState, decide_backup_observation
from lumen.topology.observability import (
    MutationKind,
    MutationState,
    Phase,
    ProgressCounters,
    StallPolicy,
    decide_mutation_observation,
    decide_stall_signal,
    phase_age_seconds,
)

MINIMUM_CHECKS = 10

OBSERVABILITY_2533_SECURITY_MATRIX = (
    ("mutation_observation_is_immutable", "FrozenInstanceError"),
    ("phase_age_never_becomes_negative", 0),
    ("threshold_equal_phase_is_not_overclaimed_as_stalled", "not_stalled"),
    ("missing_phase_policy_fails_closed", "policy_missing_phase_threshold"),
    ("missing_phase_policy_names_the_phase_threshold", "phase_thresholds.prepare_split"),
    ("explicit_prepare_split_policy_neighbour_is_admitted", "stalled"),
    ("backup_observation_never_has_a_mutation_kind", "absent"),
    ("backup_observation_never_has_a_mutation_phase", "absent"),
    ("backup_failure_reason_is_retained_not_recast_as_phase", "upload_failed"),
    ("backup_prior_success_manifest_is_retained_on_failure", "manifest://backup/40"),
)


def verify_observability_2533_security() -> dict:
    checks = []
    base = decide_mutation_observation(
        MutationState(MutationKind.SHARD_SPLIT, Phase.PREPARE_SPLIT, 41, 1_000, ProgressCounters({"prepared": 0}), 1_000, "lumen-search", "orders-3"),
        now_epoch_seconds=1_300,
    )

    # 1. R1 -- published observations cannot be rewritten after publication.
    try:
        base.generation = 99
        obs1 = "mutable"
    except FrozenInstanceError as error:
        obs1 = type(error).__name__
    exp1 = OBSERVABILITY_2533_SECURITY_MATRIX[0][1]
    checks.append({"name": OBSERVABILITY_2533_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R4 -- a persisted timestamp later than now clamps rather than yielding
    #    an impossible negative duration.
    obs2 = phase_age_seconds(1_001, 1_000)
    exp2 = OBSERVABILITY_2533_SECURITY_MATRIX[1][1]
    checks.append({"name": OBSERVABILITY_2533_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    threshold_policy = StallPolicy(phase_threshold_seconds={Phase.PREPARE_SPLIT: 300})
    # 3. AC2 -- exactly at the threshold is not "exceeds" and must not alert.
    obs3 = decide_stall_signal(base, threshold_policy).status
    exp3 = OBSERVABILITY_2533_SECURITY_MATRIX[2][1]
    checks.append({"name": OBSERVABILITY_2533_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    missing_policy = StallPolicy(phase_threshold_seconds={Phase.CATCHING_UP: 300})
    missing = decide_stall_signal(base, missing_policy)
    # 4. AC3 -- a phase without its own threshold is not silently evaluated
    #    through a final-fence default.
    obs4 = missing.status
    exp4 = OBSERVABILITY_2533_SECURITY_MATRIX[3][1]
    checks.append({"name": OBSERVABILITY_2533_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. AC3 -- the fail-closed result identifies the missing phase threshold.
    obs5 = missing.field_path
    exp5 = OBSERVABILITY_2533_SECURITY_MATRIX[4][1]
    checks.append({"name": OBSERVABILITY_2533_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    over_threshold = decide_mutation_observation(
        MutationState(MutationKind.SHARD_SPLIT, Phase.PREPARE_SPLIT, 41, 1_000, ProgressCounters({"prepared": 0}), 1_000, "lumen-search", "orders-3"),
        now_epoch_seconds=1_301,
    )
    explicit = decide_stall_signal(over_threshold, threshold_policy)
    # 6. AC3 -- the neighbouring, explicitly configured non-final phase alerts.
    obs6 = explicit.status
    exp6 = OBSERVABILITY_2533_SECURITY_MATRIX[5][1]
    checks.append({"name": OBSERVABILITY_2533_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    backup = decide_backup_observation(
        BackupState(41, {"orders-3": "partial"}, "manifest://backup/40", "upload_failed")
    )
    # 7. R5 -- backup status never masquerades as a topology mutation kind.
    obs7 = "present" if hasattr(backup, "mutation_kind") else "absent"
    exp7 = OBSERVABILITY_2533_SECURITY_MATRIX[6][1]
    checks.append({"name": OBSERVABILITY_2533_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R5 -- backup status never masquerades as a topology mutation phase.
    obs8 = "present" if hasattr(backup, "phase") else "absent"
    exp8 = OBSERVABILITY_2533_SECURITY_MATRIX[7][1]
    checks.append({"name": OBSERVABILITY_2533_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. AC4 -- the failure reason remains a backup fact, not a phase.
    obs9 = backup.failure_reason
    exp9 = OBSERVABILITY_2533_SECURITY_MATRIX[8][1]
    checks.append({"name": OBSERVABILITY_2533_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. AC4 -- the prior successful manifest remains visible after failure.
    obs10 = backup.last_successful_manifest
    exp10 = OBSERVABILITY_2533_SECURITY_MATRIX[9][1]
    checks.append({"name": OBSERVABILITY_2533_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    return {"case_id": "observability-2533-security", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(check["passed"] for check in checks) and len(checks) == MINIMUM_CHECKS}
