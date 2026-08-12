"""EC security case for #2938 -- fail-closed durability admission.

Every expected value below is an EC-owned literal transcribed from #2938:
R1/R3 refuse an explicitly embedded backend at the
``decide_new_shard_durability`` admission entry point and R3 retains the local,
test, and detected-legacy exceptions; R4 reports ``MigrationRequired``, names
the legacy state that blocks startup, and supplies no bootstrap plan; R7 maps a
typed persistence failure to ``durability-unavailable`` and ``unready``.
"""

from __future__ import annotations

from lumen.topology.durability_admission import (
    decide_new_shard_durability,
    decide_profile_durability,
    decide_startup_state,
    map_persistence_failure,
)
from lumen.topology.durability_spec import (
    DurabilityProfile,
    LegacyState,
    PersistenceFailure,
    ProfileClass,
    RaftState,
    ShardDurabilityIntent,
)
from lumen.topology.durability_verdict import Rejection

MINIMUM_CHECKS = 15

DURABILITY_CONTRACT_SECURITY_MATRIX = (
    ("production_embedded_is_rejected_for_raft_requirement", "production_requires_raft"),
    ("production_embedded_refusal_names_backend", "backend"),
    ("production_raft_neighbour_is_admitted", "admitted"),
    ("local_embedded_profile_is_admitted", "admitted"),
    ("test_embedded_profile_is_admitted", "admitted"),
    ("detected_legacy_embedded_profile_is_admitted", "admitted"),
    ("legacy_without_raft_requires_migration", "MigrationRequired"),
    ("migration_refusal_names_legacy_state", "legacy_state"),
    ("migration_refusal_has_no_empty_raft_bootstrap_plan", None),
    ("persistence_failure_is_never_success", "durability-unavailable"),
    ("persistence_failure_is_unready", "unready"),
    ("unready_failure_names_the_persistence_failure", "persistence_failure"),
    ("new_production_shard_embedded_is_rejected_for_raft_requirement", "production_requires_raft"),
    ("new_production_shard_embedded_refusal_names_backend", "backend"),
    ("new_production_shard_raft_neighbour_is_admitted", "admitted"),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def verify_durability_contract_security() -> dict:
    checks = []

    production_embedded = decide_profile_durability(
        DurabilityProfile(profile=ProfileClass.PRODUCTION, backend="embedded"),
        LegacyState.absent(),
    )

    # 1. R3 -- a new production profile cannot opt back into embedded state.
    obs1 = _outcome(production_embedded)
    exp1 = DURABILITY_CONTRACT_SECURITY_MATRIX[0][1]
    checks.append({"name": DURABILITY_CONTRACT_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R3 -- the refusal identifies the dangerous selection, not a generic error.
    obs2 = production_embedded.field_path if isinstance(production_embedded, Rejection) else ""
    exp2 = DURABILITY_CONTRACT_SECURITY_MATRIX[1][1]
    checks.append({"name": DURABILITY_CONTRACT_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R3 -- the nearest production alternative, Raft, remains admitted.
    production_raft = decide_profile_durability(
        DurabilityProfile(profile=ProfileClass.PRODUCTION, backend="raft"),
        LegacyState.absent(),
    )
    obs3 = _outcome(production_raft)
    exp3 = DURABILITY_CONTRACT_SECURITY_MATRIX[2][1]
    checks.append({"name": DURABILITY_CONTRACT_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R3 -- embedded remains selectable in the deliberately local profile.
    local_embedded = decide_profile_durability(
        DurabilityProfile(profile=ProfileClass.LOCAL, backend="embedded"),
        LegacyState.absent(),
    )
    obs4 = _outcome(local_embedded)
    exp4 = DURABILITY_CONTRACT_SECURITY_MATRIX[3][1]
    checks.append({"name": DURABILITY_CONTRACT_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R3 -- test is another explicit non-production exception.
    test_embedded = decide_profile_durability(
        DurabilityProfile(profile=ProfileClass.TEST, backend="embedded"),
        LegacyState.absent(),
    )
    obs5 = _outcome(test_embedded)
    exp5 = DURABILITY_CONTRACT_SECURITY_MATRIX[4][1]
    checks.append({"name": DURABILITY_CONTRACT_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R3 -- detected legacy is the third, explicit embedded exception.
    legacy_embedded = decide_profile_durability(
        DurabilityProfile(profile=ProfileClass.PRODUCTION, backend="embedded"),
        LegacyState.non_empty(),
    )
    obs6 = _outcome(legacy_embedded)
    exp6 = DURABILITY_CONTRACT_SECURITY_MATRIX[5][1]
    checks.append({"name": DURABILITY_CONTRACT_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    startup = decide_startup_state(LegacyState.non_empty(), RaftState.absent())

    # 7. R4 -- non-empty legacy state without Raft state is migration-required.
    obs7 = _outcome(startup)
    exp7 = DURABILITY_CONTRACT_SECURITY_MATRIX[6][1]
    checks.append({"name": DURABILITY_CONTRACT_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R4 -- the refusal identifies which authoritative state blocks startup.
    obs8 = startup.field_path if isinstance(startup, Rejection) else ""
    exp8 = DURABILITY_CONTRACT_SECURITY_MATRIX[7][1]
    checks.append({"name": DURABILITY_CONTRACT_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R4 -- no empty-Raft bootstrap proposal accompanies that refusal.
    obs9 = startup.bootstrap_plan if isinstance(startup, Rejection) else "unexpected_admission"
    exp9 = DURABILITY_CONTRACT_SECURITY_MATRIX[8][1]
    checks.append({"name": DURABILITY_CONTRACT_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    failure = map_persistence_failure(PersistenceFailure(kind="no_space", detail="ENOSPC"))

    # 10. R7 -- a persistence failure cannot become a successful write response.
    obs10 = failure.write_result
    exp10 = DURABILITY_CONTRACT_SECURITY_MATRIX[9][1]
    checks.append({"name": DURABILITY_CONTRACT_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. R7 -- its readiness vocabulary is explicitly unready.
    obs11 = failure.readiness
    exp11 = DURABILITY_CONTRACT_SECURITY_MATRIX[10][1]
    checks.append({"name": DURABILITY_CONTRACT_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12. R7 -- the status retains the typed failure dimension for operators.
    obs12 = failure.field_path
    exp12 = DURABILITY_CONTRACT_SECURITY_MATRIX[11][1]
    checks.append({"name": DURABILITY_CONTRACT_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    production_embedded_shard = decide_new_shard_durability(
        ShardDurabilityIntent(
            profile=ProfileClass.PRODUCTION,
            shard_name="orders",
            shard_ordinal=0,
            voters=1,
            backend="embedded",
        )
    )

    # 13. R1/R3 -- new production shards cannot explicitly opt into embedded state.
    obs13 = _outcome(production_embedded_shard)
    exp13 = DURABILITY_CONTRACT_SECURITY_MATRIX[12][1]
    checks.append({"name": DURABILITY_CONTRACT_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    # 14. R1/R3 -- the admission refusal names the dangerous backend field.
    obs14 = production_embedded_shard.field_path if isinstance(production_embedded_shard, Rejection) else ""
    exp14 = DURABILITY_CONTRACT_SECURITY_MATRIX[13][1]
    checks.append({"name": DURABILITY_CONTRACT_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    production_raft_shard = decide_new_shard_durability(
        ShardDurabilityIntent(
            profile=ProfileClass.PRODUCTION,
            shard_name="orders",
            shard_ordinal=0,
            voters=1,
            backend="raft",
        )
    )

    # 15. R1/R3 -- an explicitly Raft-backed production shard remains admitted.
    obs15 = _outcome(production_raft_shard)
    exp15 = DURABILITY_CONTRACT_SECURITY_MATRIX[14][1]
    checks.append({"name": DURABILITY_CONTRACT_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    return {
        "case_id": "durability-contract-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
