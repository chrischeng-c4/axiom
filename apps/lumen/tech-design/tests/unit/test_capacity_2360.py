"""Unit tests for Lumen capacity design package (#2360)."""

from __future__ import annotations

import unittest

from lumen.capacity.blocking import decide_capacity_block
from lumen.capacity.ownership import decide_reapply
from lumen.capacity.placement import decide_pool_assignments
from lumen.capacity.resume import decide_resume
from lumen.capacity.storage import decide_member_storage
from lumen.capacity.transition import decide_downgrade
from lumen.capacity.verdict import (
    CapacityReason,
    CapacityRejection,
    ReapplyAction,
    ReclaimAction,
    TransitionKind,
)
from lumen.topology.availability import AvailabilityPromise, availability_promise


class TestCapacity2360Design(unittest.TestCase):
    """Test capacity decision rules with custom parameters beyond contract fixtures."""

    def test_decide_resume_custom_and_edge_cases(self) -> None:
        # Active mutation prevents resuming another action
        res1 = decide_resume(
            interrupted_state={"active_mutation": "resize-999", "phase": "running"},
            persisted_action={"identifier": "resize-1000", "kind": "machine_upgrade"},
        )
        self.assertIsInstance(res1, CapacityRejection)
        self.assertEqual(res1.reason, CapacityReason.ANOTHER_MUTATION_ACTIVE)
        self.assertEqual(res1.field_path, "interrupted_state.active_mutation")

        # Inactive state allows resuming persisted action
        res2 = decide_resume(
            interrupted_state={"active_mutation": None, "phase": "interrupted"},
            persisted_action={"identifier": "resize-777", "kind": "pvc_growth"},
        )
        self.assertNotIsInstance(res2, CapacityRejection)
        self.assertEqual(res2.next_mutation.identifier, "resize-777")
        self.assertEqual(res2.next_mutation.kind, "pvc_growth")

        # Missing persisted action fails closed
        res3 = decide_resume(
            interrupted_state={"active_mutation": None},
            persisted_action=None,
        )
        self.assertIsInstance(res3, CapacityRejection)
        self.assertEqual(res3.reason, CapacityReason.INVALID_INPUT)
        self.assertEqual(res3.field_path, "persisted_action")

    def test_decide_reapply_custom_and_edge_cases(self) -> None:
        # Unchanged CR reapply preserves automatic target
        re1 = decide_reapply(
            initial={"machine_type": "e2-standard-2", "owner": "user"},
            current={"machine_type": "e2-standard-2", "owner": "automatic"},
            target={"machine_type": "e2-standard-4", "owner": "automatic"},
            rendered_input={"machine_type": "e2-standard-2"},
        )
        self.assertNotIsInstance(re1, CapacityRejection)
        self.assertEqual(re1.target.machine_type, "e2-standard-4")
        self.assertEqual(re1.initial.machine_type, "e2-standard-2")
        self.assertEqual(re1.initial.owner, "user")
        self.assertEqual(re1.current.owner, "automatic")
        self.assertEqual(re1.action, ReapplyAction.NO_OP)

        # Competing rendered input is rejected
        re2 = decide_reapply(
            initial={"machine_type": "e2-standard-2", "owner": "user"},
            current={"machine_type": "e2-standard-2", "owner": "automatic"},
            target={"machine_type": "e2-standard-4", "owner": "automatic"},
            rendered_input={"machine_type": "c2-standard-8"},
        )
        self.assertIsInstance(re2, CapacityRejection)
        self.assertEqual(re2.reason, CapacityReason.COMPETING_MUTATION)
        self.assertEqual(re2.field_path, "rendered_input.machine_type")

    def test_decide_downgrade_custom_and_edge_cases(self) -> None:
        policy = {"stable_window_seconds": 200, "cooldown_seconds": 100, "pool_maximum": 5}

        # Gate 1: stable window not elapsed
        st1 = decide_downgrade(
            policy=policy,
            current={"machine_type": "m1", "node_count": 5, "stable_since": 400, "last_transition_at": 0},
            proposed={"machine_type": "m2", "node_count": 3, "projected_allocatable_headroom": 2},
            observed_at=500,
        )
        self.assertIsInstance(st1, CapacityRejection)
        self.assertEqual(st1.reason, CapacityReason.STABLE_WINDOW_NOT_ELAPSED)
        self.assertEqual(st1.field_path, "current.stable_since")

        # Gate 2: cooldown active
        cd1 = decide_downgrade(
            policy=policy,
            current={"machine_type": "m1", "node_count": 5, "stable_since": 0, "last_transition_at": 950},
            proposed={"machine_type": "m2", "node_count": 3, "projected_allocatable_headroom": 2},
            observed_at=1000,
        )
        self.assertIsInstance(cd1, CapacityRejection)
        self.assertEqual(cd1.reason, CapacityReason.COOLDOWN_ACTIVE)
        self.assertEqual(cd1.field_path, "current.last_transition_at")

        # Gate 3: pool maximum exceeded
        mx1 = decide_downgrade(
            policy=policy,
            current={"machine_type": "m1", "node_count": 5, "stable_since": 0, "last_transition_at": 0},
            proposed={"machine_type": "m2", "node_count": 10, "projected_allocatable_headroom": 2},
            observed_at=1000,
        )
        self.assertIsInstance(mx1, CapacityRejection)
        self.assertEqual(mx1.reason, CapacityReason.POOL_MAXIMUM_EXCEEDED)
        self.assertEqual(mx1.field_path, "proposed.node_count")

        # Gate 4: insufficient headroom
        hr1 = decide_downgrade(
            policy=policy,
            current={"machine_type": "m1", "node_count": 5, "stable_since": 0, "last_transition_at": 0},
            proposed={"machine_type": "m2", "node_count": 3, "projected_allocatable_headroom": 0},
            observed_at=1000,
        )
        self.assertIsInstance(hr1, CapacityRejection)
        self.assertEqual(hr1.reason, CapacityReason.INSUFFICIENT_HEADROOM)
        self.assertEqual(hr1.field_path, "proposed.projected_allocatable_headroom")

        # All gates pass -> admitted
        adm = decide_downgrade(
            policy=policy,
            current={"machine_type": "m1", "node_count": 5, "stable_since": 0, "last_transition_at": 0},
            proposed={"machine_type": "m2", "node_count": 3, "projected_allocatable_headroom": 2},
            observed_at=1000,
        )
        self.assertNotIsInstance(adm, CapacityRejection)
        self.assertEqual(adm.kind, TransitionKind.ADMITTED)
        self.assertEqual(adm.target_machine_type, "m2")
        self.assertEqual(adm.target_node_count, 3)

    def test_decide_member_storage_custom_and_edge_cases(self) -> None:
        # Member creation size
        st1 = decide_member_storage(
            catalog={"committed_desired_size": "500Gi"},
            member_role="voter",
            lifecycle_event="created",
        )
        self.assertNotIsInstance(st1, CapacityRejection)
        self.assertEqual(st1.desired_size, "500Gi")
        self.assertEqual(st1.reclaim, ReclaimAction.RETAIN)

        # Drained read replica reclaims storage
        st2 = decide_member_storage(
            catalog={"committed_desired_size": "500Gi"},
            member_role="read_replica",
            lifecycle_event="drained",
        )
        self.assertEqual(st2.reclaim, ReclaimAction.RECLAIM)

        # Drained voter retains storage
        st3 = decide_member_storage(
            catalog={"committed_desired_size": "500Gi"},
            member_role="voter",
            lifecycle_event="drained",
        )
        self.assertEqual(st3.reclaim, ReclaimAction.RETAIN)

    def test_decide_pool_assignments_custom_and_edge_cases(self) -> None:
        # Pool key derivation and distinct nodes
        pl1 = decide_pool_assignments(
            instances=(
                {"namespace": "ns1", "instance": "i1", "machine_type": "c2-standard-4"},
                {"namespace": "ns2", "instance": "i2", "machine_type": "c2-standard-4"},
            ),
            placements=(
                {"instance": "i1", "node": "node-10"},
                {"instance": "i2", "node": "node-20"},
            ),
        )
        self.assertNotIsInstance(pl1, CapacityRejection)
        self.assertEqual(pl1.assignments["i1"].pool_key, "data-c2-standard-4")
        self.assertEqual(pl1.assignments["i2"].pool_key, "data-c2-standard-4")

        # Node conflict in same pool key
        pl2 = decide_pool_assignments(
            instances=(
                {"namespace": "ns1", "instance": "i1", "machine_type": "c2-standard-4"},
                {"namespace": "ns2", "instance": "i2", "machine_type": "c2-standard-4"},
            ),
            placements=(
                {"instance": "i1", "node": "node-99"},
                {"instance": "i2", "node": "node-99"},
            ),
        )
        self.assertIsInstance(pl2, CapacityRejection)
        self.assertEqual(pl2.reason, CapacityReason.DATA_MEMBER_NODE_CONFLICT)
        self.assertEqual(pl2.field_path, "placements")

    def test_decide_capacity_block_custom_and_edge_cases(self) -> None:
        # Blocked condition preserves old member and generation
        blk1 = decide_capacity_block(
            condition={"kind": "at_maximum"},
            old_member={"identifier": "mem-88", "healthy": True},
            generation=42,
        )
        self.assertNotIsInstance(blk1, CapacityRejection)
        self.assertEqual(blk1.condition.type, "CapacityBlocked")
        self.assertEqual(blk1.old_member.identifier, "mem-88")
        self.assertTrue(blk1.old_member.healthy)
        self.assertEqual(blk1.generation, 42)

        # Corrected condition resumes same generation
        blk2 = decide_capacity_block(
            condition={"kind": "corrected"},
            old_member={"identifier": "mem-88", "healthy": True},
            generation=42,
        )
        self.assertEqual(blk2.resume_generation, 42)

    def test_availability_promise_contract(self) -> None:
        self.assertEqual(availability_promise(1), AvailabilityPromise.NO_PROMISE_ON_UNEXPECTED_NODE_LOSS)
        self.assertEqual(availability_promise(3), AvailabilityPromise.SURVIVES_ONE_UNEXPECTED_NODE_LOSS)
        with self.assertRaises(ValueError):
            availability_promise(2)


if __name__ == "__main__":
    unittest.main()
