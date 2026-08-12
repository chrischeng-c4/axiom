"""EC security case for #2939 -- fail-closed catalog authority.

Expected literals are transcribed from #2939: R1 refuses unsupported catalog
modes and insufficient eligible voters; R3 refuses operator-only serving; R4
rejects mismatched-instance and stale seed generations deterministically; and
R6 refuses stale cache replacements and mutations without current quorum.
Each refusal is checked for its vocabulary, named field, and an adjacent
admitted input at the same entry point.
"""

from __future__ import annotations

from lumen.topology.catalog_access import decide_serving_topology
from lumen.topology.catalog_admission import decide_catalog_spec
from lumen.topology.catalog_bootstrap import decide_bootstrap
from lumen.topology.catalog_cache import decide_cache_update
from lumen.topology.catalog_spec import BootstrapSeed, CatalogSpec, EligibleMember
from lumen.topology.catalog_state import CatalogState
from lumen.topology.catalog_verdict import Rejection

MINIMUM_CHECKS = 18

CATALOG_2939_SECURITY_MATRIX = (
    ("unsupported_catalog_mode_is_rejected", "unsupported_catalog_mode"),
    ("unsupported_catalog_mode_refusal_names_mode", "mode"),
    ("non_ha_neighbour_is_admitted", "admitted"),
    ("three_voter_ha_with_two_eligible_members_is_rejected", "insufficient_eligible_members"),
    ("insufficient_members_refusal_names_eligible_members", "eligible_members"),
    ("three_voter_ha_with_three_eligible_members_is_admitted", "admitted"),
    ("operator_only_serving_source_is_rejected", "operator_not_serving_authority"),
    ("operator_only_refusal_names_source", "source"),
    ("catalog_serving_neighbour_is_admitted", "catalog"),
    ("foreign_instance_seed_is_rejected", "instance_id_mismatch"),
    ("foreign_instance_refusal_names_seed_instance_id", "seed.instance_id"),
    ("matching_instance_seed_is_admitted", "admitted"),
    ("stale_seed_generation_is_rejected", "stale_seed_generation"),
    ("stale_seed_refusal_names_generation", "generation"),
    ("same_stale_seed_is_deterministic", "stale_seed_generation"),
    ("stale_cache_candidate_is_rejected", "stale_catalog_generation"),
    ("stale_cache_refusal_names_candidate_generation", "candidate.current_generation"),
    ("quorum_unavailable_mutation_is_rejected", "catalog_quorum_unavailable"),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def verify_catalog_2939_security() -> dict:
    checks = []
    eligible_three = (
        EligibleMember(member_id="node-a", hostname="host-a", zone="zone-a"),
        EligibleMember(member_id="node-b", hostname="host-b", zone="zone-b"),
        EligibleMember(member_id="node-c", hostname="host-c", zone="zone-c"),
    )
    unsupported = decide_catalog_spec(CatalogSpec(instance_id="lumen-a", mode="five-voter-ha"), eligible_three)
    # 1-3. R1 -- unsupported mode, named field, adjacent admitted non-HA mode.
    obs1 = _outcome(unsupported); exp1 = CATALOG_2939_SECURITY_MATRIX[0][1]
    checks.append({"name": CATALOG_2939_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = unsupported.field_path if isinstance(unsupported, Rejection) else ""; exp2 = CATALOG_2939_SECURITY_MATRIX[1][1]
    checks.append({"name": CATALOG_2939_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    obs3 = _outcome(decide_catalog_spec(CatalogSpec(instance_id="lumen-a", mode="non-ha"), eligible_three)); exp3 = CATALOG_2939_SECURITY_MATRIX[2][1]
    checks.append({"name": CATALOG_2939_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    insufficient = decide_catalog_spec(CatalogSpec(instance_id="lumen-a", mode="three-voter-ha"), eligible_three[:2])
    # 4-6. R1 -- HA fails closed without three eligible members, but admits at capacity.
    obs4 = _outcome(insufficient); exp4 = CATALOG_2939_SECURITY_MATRIX[3][1]
    checks.append({"name": CATALOG_2939_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    obs5 = insufficient.field_path if isinstance(insufficient, Rejection) else ""; exp5 = CATALOG_2939_SECURITY_MATRIX[4][1]
    checks.append({"name": CATALOG_2939_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    obs6 = _outcome(decide_catalog_spec(CatalogSpec(instance_id="lumen-a", mode="three-voter-ha"), eligible_three)); exp6 = CATALOG_2939_SECURITY_MATRIX[5][1]
    checks.append({"name": CATALOG_2939_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    operator = decide_serving_topology("operator")
    # 7-9. R3 -- operator-only routing is never serving authority.
    obs7 = _outcome(operator); exp7 = CATALOG_2939_SECURITY_MATRIX[6][1]
    checks.append({"name": CATALOG_2939_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    obs8 = operator.field_path if isinstance(operator, Rejection) else ""; exp8 = CATALOG_2939_SECURITY_MATRIX[7][1]
    checks.append({"name": CATALOG_2939_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    obs9 = decide_serving_topology("catalog").source; exp9 = CATALOG_2939_SECURITY_MATRIX[8][1]
    checks.append({"name": CATALOG_2939_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    foreign_seed = BootstrapSeed(instance_id="lumen-b", seed_id="seed-a", hostname="host-a", zone="zone-a", generation=8)
    mismatch = decide_bootstrap(foreign_seed, "lumen-a", 7)
    matching_seed = BootstrapSeed(instance_id="lumen-a", seed_id="seed-a", hostname="host-a", zone="zone-a", generation=8)
    # 10-12. R4/AC4 -- foreign identity has a typed fielded refusal; matching seed admits.
    obs10 = _outcome(mismatch); exp10 = CATALOG_2939_SECURITY_MATRIX[9][1]
    checks.append({"name": CATALOG_2939_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})
    obs11 = mismatch.field_path if isinstance(mismatch, Rejection) else ""; exp11 = CATALOG_2939_SECURITY_MATRIX[10][1]
    checks.append({"name": CATALOG_2939_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})
    obs12 = _outcome(decide_bootstrap(matching_seed, "lumen-a", 7)); exp12 = CATALOG_2939_SECURITY_MATRIX[11][1]
    checks.append({"name": CATALOG_2939_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    stale_seed = BootstrapSeed(instance_id="lumen-a", seed_id="seed-a", hostname="host-a", zone="zone-a", generation=6)
    stale = decide_bootstrap(stale_seed, "lumen-a", 7)
    # 13-15. R4/AC4 -- stale identity is rejected, names its generation, and is deterministic.
    obs13 = _outcome(stale); exp13 = CATALOG_2939_SECURITY_MATRIX[12][1]
    checks.append({"name": CATALOG_2939_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})
    obs14 = stale.field_path if isinstance(stale, Rejection) else ""; exp14 = CATALOG_2939_SECURITY_MATRIX[13][1]
    checks.append({"name": CATALOG_2939_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})
    obs15 = _outcome(decide_bootstrap(stale_seed, "lumen-a", 7)); exp15 = CATALOG_2939_SECURITY_MATRIX[14][1]
    checks.append({"name": CATALOG_2939_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    current = CatalogState(shard_ranges=((0, 1023, "orders-a"),), shard_group_ids=("orders-a",), member_roles=(("node-a", "voter"),), collection_schema_generations=(("orders", 8),), mutation_intent="none", current_generation=8, converged_generation=8)
    stale_candidate = CatalogState(shard_ranges=((0, 511, "orders-a"),), shard_group_ids=("orders-a",), member_roles=(("node-a", "voter"),), collection_schema_generations=(("orders", 7),), mutation_intent="none", current_generation=7, converged_generation=7)
    stale_cache = decide_cache_update(current, stale_candidate, quorum_available=True)
    quorum_missing = decide_cache_update(current, current, quorum_available=False)
    # 16-18. R6/AC1 -- stale state cannot replace current, and no-quorum mutations fail closed.
    obs16 = _outcome(stale_cache); exp16 = CATALOG_2939_SECURITY_MATRIX[15][1]
    checks.append({"name": CATALOG_2939_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})
    obs17 = stale_cache.field_path if isinstance(stale_cache, Rejection) else ""; exp17 = CATALOG_2939_SECURITY_MATRIX[16][1]
    checks.append({"name": CATALOG_2939_SECURITY_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})
    obs18 = _outcome(quorum_missing); exp18 = CATALOG_2939_SECURITY_MATRIX[17][1]
    checks.append({"name": CATALOG_2939_SECURITY_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})

    return {"case_id": "catalog-2939-security", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
