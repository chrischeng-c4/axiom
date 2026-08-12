"""EC behavior case for #2939 -- catalog-owned topology authority.

Every expected value is an EC-owned literal from #2939: R1 chooses exactly one
or three catalog voters and spreads three-voter placement across hostnames and
zones; R2 makes the listed topology and schema dimensions immutable catalog
state; R3 permits catalog or retained-cache serving authority; R4 admits an
independent matching seed; R6 advances the last-converged cache monotonically;
and AC2 retains an explicit last-converged value when no newer value arrives.
"""

from __future__ import annotations

from lumen.topology.catalog_access import decide_serving_topology
from lumen.topology.catalog_admission import decide_catalog_spec
from lumen.topology.catalog_bootstrap import decide_bootstrap
from lumen.topology.catalog_cache import decide_cache_update, last_converged
from lumen.topology.catalog_spec import BootstrapSeed, CatalogSpec, EligibleMember
from lumen.topology.catalog_state import CatalogState
from lumen.topology.catalog_verdict import AdmittedCatalogPlan, Rejection

MINIMUM_CHECKS = 18

CATALOG_2939_BEHAVIOR_MATRIX = (
    ("non_ha_catalog_has_one_voter", 1),
    ("non_ha_catalog_selects_the_named_eligible_member", ("node-a",)),
    ("ha_catalog_has_three_voters", 3),
    ("ha_catalog_spreads_voters_across_hostnames", ("host-a", "host-b", "host-c")),
    ("ha_catalog_spreads_voters_across_zones", ("zone-a", "zone-b", "zone-c")),
    ("catalog_state_exposes_versioned_shard_ranges", ((0, 1023, "orders-a"),)),
    ("catalog_state_exposes_shard_group_identifiers", ("orders-a",)),
    ("catalog_state_exposes_member_roles", (("node-a", "voter"),)),
    ("catalog_state_exposes_collection_schema_generations", (("orders", 7),)),
    ("catalog_state_exposes_mutation_intent", "add-orders-index"),
    ("catalog_state_exposes_current_generation", 8),
    ("catalog_state_exposes_converged_generation", 7),
    ("catalog_state_is_immutable", True),
    ("catalog_source_is_admitted_for_serving", "catalog"),
    ("last_converged_cache_source_is_admitted_for_serving", "last-converged-cache"),
    ("matching_seed_is_admitted_with_its_location", "admitted"),
    ("last_converged_cache_is_retained_without_a_newer_value", 7),
    ("cache_advances_to_a_newer_catalog_generation", 8),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def verify_catalog_2939_behavior() -> dict:
    checks = []
    eligible = (
        EligibleMember(member_id="node-a", hostname="host-a", zone="zone-a"),
        EligibleMember(member_id="node-b", hostname="host-b", zone="zone-b"),
        EligibleMember(member_id="node-c", hostname="host-c", zone="zone-c"),
    )

    non_ha = decide_catalog_spec(CatalogSpec(instance_id="lumen-a", mode="non-ha"), eligible)
    # 1. R1 -- non-HA intentionally has one catalog voter.
    obs1 = non_ha.voter_count if isinstance(non_ha, AdmittedCatalogPlan) else _outcome(non_ha)
    exp1 = CATALOG_2939_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": CATALOG_2939_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R1 -- its selected member is deterministic, not an anonymous placement.
    obs2 = non_ha.member_ids if isinstance(non_ha, AdmittedCatalogPlan) else ()
    exp2 = CATALOG_2939_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": CATALOG_2939_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    ha = decide_catalog_spec(CatalogSpec(instance_id="lumen-a", mode="three-voter-ha"), eligible)
    # 3. R1/AC3 -- HA is the explicit three-voter choice.
    obs3 = ha.voter_count if isinstance(ha, AdmittedCatalogPlan) else _outcome(ha)
    exp3 = CATALOG_2939_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": CATALOG_2939_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R1 -- capacity permits three different hostname domains.
    obs4 = ha.hostnames if isinstance(ha, AdmittedCatalogPlan) else ()
    exp4 = CATALOG_2939_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": CATALOG_2939_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R1 -- the same plan also spans three zone domains.
    obs5 = ha.zones if isinstance(ha, AdmittedCatalogPlan) else ()
    exp5 = CATALOG_2939_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": CATALOG_2939_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    state = CatalogState(
        shard_ranges=((0, 1023, "orders-a"),), shard_group_ids=("orders-a",),
        member_roles=(("node-a", "voter"),), collection_schema_generations=(("orders", 7),),
        mutation_intent="add-orders-index", current_generation=8, converged_generation=7,
    )
    # 6. R2 -- ranges are catalog state, not an operator-derived default.
    obs6 = state.shard_ranges
    exp6 = CATALOG_2939_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": CATALOG_2939_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R2 -- shard-group identity is independently retained.
    obs7 = state.shard_group_ids
    exp7 = CATALOG_2939_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": CATALOG_2939_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R2 -- member roles remain visible to topology consumers.
    obs8 = state.member_roles
    exp8 = CATALOG_2939_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": CATALOG_2939_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R2 -- schema generations have a catalog-owned value.
    obs9 = state.collection_schema_generations
    exp9 = CATALOG_2939_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": CATALOG_2939_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. R2 -- pending mutation intent is not discarded from state.
    obs10 = state.mutation_intent
    exp10 = CATALOG_2939_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": CATALOG_2939_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. R2 -- current generation is explicit.
    obs11 = state.current_generation
    exp11 = CATALOG_2939_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": CATALOG_2939_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12. R2 -- convergence generation remains distinct from current generation.
    obs12 = state.converged_generation
    exp12 = CATALOG_2939_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": CATALOG_2939_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13. R2 -- catalog state is a frozen value, not a mutable operator cache.
    obs13 = CatalogState.__dataclass_params__.frozen
    exp13 = CATALOG_2939_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": CATALOG_2939_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    # 14. R3 -- current catalog state is a permitted serving authority.
    obs14 = decide_serving_topology("catalog").source
    exp14 = CATALOG_2939_BEHAVIOR_MATRIX[13][1]
    checks.append({"name": CATALOG_2939_BEHAVIOR_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    # 15. R3/AC2 -- retained state is likewise an explicit serving authority.
    obs15 = decide_serving_topology("last-converged-cache").source
    exp15 = CATALOG_2939_BEHAVIOR_MATRIX[14][1]
    checks.append({"name": CATALOG_2939_BEHAVIOR_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    # 16. R4 -- matching independent seed identity and location is admitted.
    seed = BootstrapSeed(instance_id="lumen-a", seed_id="seed-a", hostname="host-a", zone="zone-a", generation=8)
    obs16 = _outcome(decide_bootstrap(seed, "lumen-a", 7))
    exp16 = CATALOG_2939_BEHAVIOR_MATRIX[15][1]
    checks.append({"name": CATALOG_2939_BEHAVIOR_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})

    # 17. R6/AC2 -- without a newer input, the explicit retained generation remains.
    obs17 = last_converged(state, None).converged_generation
    exp17 = CATALOG_2939_BEHAVIOR_MATRIX[16][1]
    checks.append({"name": CATALOG_2939_BEHAVIOR_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})

    # 18. R6/AC1 -- a newer catalog candidate advances retained routing state.
    current = CatalogState(
        shard_ranges=((0, 511, "orders-a"),), shard_group_ids=("orders-a",),
        member_roles=(("node-a", "voter"),), collection_schema_generations=(("orders", 7),),
        mutation_intent="none", current_generation=7, converged_generation=7,
    )
    advanced = decide_cache_update(current, state, quorum_available=True)
    obs18 = advanced.current_generation if not isinstance(advanced, Rejection) else _outcome(advanced)
    exp18 = CATALOG_2939_BEHAVIOR_MATRIX[17][1]
    checks.append({"name": CATALOG_2939_BEHAVIOR_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})

    return {"case_id": "catalog-2939-behavior", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
