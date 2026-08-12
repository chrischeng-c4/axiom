"""Unit tests for Lumen catalog design (#2939)."""
from __future__ import annotations

from dataclasses import FrozenInstanceError
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from lumen.topology.catalog_access import AdmittedServingSource, decide_serving_topology
from lumen.topology.catalog_admission import decide_catalog_spec
from lumen.topology.catalog_bootstrap import AdmittedBootstrapSeed, decide_bootstrap
from lumen.topology.catalog_cache import decide_cache_update, last_converged
from lumen.topology.catalog_spec import BootstrapSeed, CatalogSpec, EligibleMember
from lumen.topology.catalog_state import CatalogState
from lumen.topology.catalog_status import CatalogStatus
from lumen.topology.catalog_verdict import (
    AdmittedCatalogPlan,
    CatalogRejectionReason,
    Rejection,
)


class TestCatalog2939Design(unittest.TestCase):
    def test_catalog_spec_admission_non_ha(self) -> None:
        eligible = (
            EligibleMember(member_id="m-2", hostname="h-2", zone="z-b"),
            EligibleMember(member_id="m-1", hostname="h-1", zone="z-a"),
        )
        plan = decide_catalog_spec(CatalogSpec(instance_id="inst-1", mode="non-ha"), eligible)
        self.assertIsInstance(plan, AdmittedCatalogPlan)
        assert isinstance(plan, AdmittedCatalogPlan)
        self.assertEqual(plan.voter_count, 1)
        self.assertEqual(plan.member_ids, ("m-1",))
        self.assertEqual(plan.hostnames, ("h-1",))
        self.assertEqual(plan.zones, ("z-a",))
        self.assertIsNotNone(plan.limitation)

        # Verify order independence: passing members in different input order yields exact same plan
        reversed_eligible = tuple(reversed(eligible))
        plan_rev = decide_catalog_spec(CatalogSpec(instance_id="inst-1", mode="non-ha"), reversed_eligible)
        self.assertEqual(plan, plan_rev)

    def test_catalog_spec_admission_non_ha_insufficient_members(self) -> None:
        rejection = decide_catalog_spec(CatalogSpec(instance_id="inst-1", mode="non-ha"), ())
        self.assertIsInstance(rejection, Rejection)
        assert isinstance(rejection, Rejection)
        self.assertEqual(rejection.reason, CatalogRejectionReason.INSUFFICIENT_ELIGIBLE_MEMBERS)
        self.assertEqual(rejection.field_path, "eligible_members")

    def test_catalog_spec_admission_three_voter_ha(self) -> None:
        members = (
            EligibleMember(member_id="node-z", hostname="host-3", zone="zone-3"),
            EligibleMember(member_id="node-x", hostname="host-1", zone="zone-1"),
            EligibleMember(member_id="node-y", hostname="host-2", zone="zone-2"),
        )
        plan = decide_catalog_spec(CatalogSpec(instance_id="inst-9", mode="three-voter-ha"), members)
        self.assertIsInstance(plan, AdmittedCatalogPlan)
        assert isinstance(plan, AdmittedCatalogPlan)
        self.assertEqual(plan.voter_count, 3)
        self.assertEqual(len(plan.member_ids), 3)
        self.assertEqual(set(plan.hostnames), {"host-1", "host-2", "host-3"})
        self.assertEqual(set(plan.zones), {"zone-1", "zone-2", "zone-3"})
        self.assertIsNone(plan.limitation)

        # Order independence check
        plan_shuffled = decide_catalog_spec(
            CatalogSpec(instance_id="inst-9", mode="three-voter-ha"),
            (members[1], members[2], members[0]),
        )
        self.assertEqual(plan, plan_shuffled)

    def test_catalog_spec_admission_three_voter_ha_insufficient_members(self) -> None:
        members = (
            EligibleMember(member_id="node-x", hostname="host-1", zone="zone-1"),
            EligibleMember(member_id="node-y", hostname="host-2", zone="zone-2"),
        )
        rejection = decide_catalog_spec(CatalogSpec(instance_id="inst-9", mode="three-voter-ha"), members)
        self.assertIsInstance(rejection, Rejection)
        assert isinstance(rejection, Rejection)
        self.assertEqual(rejection.reason, CatalogRejectionReason.INSUFFICIENT_ELIGIBLE_MEMBERS)
        self.assertEqual(rejection.field_path, "eligible_members")

    def test_catalog_spec_admission_unsupported_mode(self) -> None:
        rejection = decide_catalog_spec(CatalogSpec(instance_id="inst-9", mode="custom-mode"), ())
        self.assertIsInstance(rejection, Rejection)
        assert isinstance(rejection, Rejection)
        self.assertEqual(rejection.reason, CatalogRejectionReason.UNSUPPORTED_CATALOG_MODE)
        self.assertEqual(rejection.field_path, "mode")

    def test_catalog_state_immutability_and_fields(self) -> None:
        state = CatalogState(
            shard_ranges=((0, 100, "group-1"),),
            shard_group_ids=("group-1",),
            member_roles=(("m-1", "voter"),),
            collection_schema_generations=(("col-1", 3),),
            mutation_intent="split",
            current_generation=10,
            converged_generation=9,
        )
        self.assertEqual(state.current_generation, 10)
        self.assertEqual(state.converged_generation, 9)

        with self.assertRaises(FrozenInstanceError):
            state.current_generation = 11  # type: ignore[misc]

    def test_serving_topology_authority(self) -> None:
        res_cat = decide_serving_topology("catalog")
        self.assertIsInstance(res_cat, AdmittedServingSource)
        assert isinstance(res_cat, AdmittedServingSource)
        self.assertEqual(res_cat.source, "catalog")

        res_cache = decide_serving_topology("last-converged-cache")
        self.assertIsInstance(res_cache, AdmittedServingSource)
        assert isinstance(res_cache, AdmittedServingSource)
        self.assertEqual(res_cache.source, "last-converged-cache")

        rej_op = decide_serving_topology("operator")
        self.assertIsInstance(rej_op, Rejection)
        assert isinstance(rej_op, Rejection)
        self.assertEqual(rej_op.reason, CatalogRejectionReason.OPERATOR_NOT_SERVING_AUTHORITY)
        self.assertEqual(rej_op.field_path, "source")

    def test_bootstrap_seed(self) -> None:
        seed = BootstrapSeed(instance_id="inst-10", seed_id="s-1", hostname="h-1", zone="z-1", generation=5)
        admitted = decide_bootstrap(seed, "inst-10", 4)
        self.assertIsInstance(admitted, AdmittedBootstrapSeed)

        rej_mismatch = decide_bootstrap(seed, "inst-other", 4)
        self.assertIsInstance(rej_mismatch, Rejection)
        assert isinstance(rej_mismatch, Rejection)
        self.assertEqual(rej_mismatch.reason, CatalogRejectionReason.INSTANCE_ID_MISMATCH)
        self.assertEqual(rej_mismatch.field_path, "seed.instance_id")

        rej_stale = decide_bootstrap(seed, "inst-10", 6)
        self.assertIsInstance(rej_stale, Rejection)
        assert isinstance(rej_stale, Rejection)
        self.assertEqual(rej_stale.reason, CatalogRejectionReason.STALE_SEED_GENERATION)
        self.assertEqual(rej_stale.field_path, "generation")

    def test_cache_update_and_last_converged(self) -> None:
        s1 = CatalogState(current_generation=1, converged_generation=1)
        s2 = CatalogState(current_generation=2, converged_generation=1)

        # Quorum unavailable
        rej_q = decide_cache_update(s1, s2, quorum_available=False)
        self.assertIsInstance(rej_q, Rejection)
        assert isinstance(rej_q, Rejection)
        self.assertEqual(rej_q.reason, CatalogRejectionReason.CATALOG_QUORUM_UNAVAILABLE)
        self.assertEqual(rej_q.field_path, "quorum_available")

        # Stale candidate
        rej_stale = decide_cache_update(s2, s1, quorum_available=True)
        self.assertIsInstance(rej_stale, Rejection)
        assert isinstance(rej_stale, Rejection)
        self.assertEqual(rej_stale.reason, CatalogRejectionReason.STALE_CATALOG_GENERATION)
        self.assertEqual(rej_stale.field_path, "candidate.current_generation")

        # Valid update
        ok = decide_cache_update(s1, s2, quorum_available=True)
        self.assertEqual(ok, s2)

        # last_converged
        self.assertEqual(last_converged(s2, None), s2)
        self.assertEqual(last_converged(s2, s1), s2)
        self.assertEqual(last_converged(s1, s2), s2)

    def test_catalog_status(self) -> None:
        spec = CatalogSpec(instance_id="i-1", mode="non-ha")
        s_unconverged = CatalogState(current_generation=5, converged_generation=4)
        s_converged = CatalogState(current_generation=5, converged_generation=5)

        status1 = CatalogStatus(spec=spec, state=s_unconverged)
        self.assertFalse(status1.is_converged())

        status2 = CatalogStatus(spec=spec, state=s_converged)
        self.assertTrue(status2.is_converged())


if __name__ == "__main__":
    unittest.main()
