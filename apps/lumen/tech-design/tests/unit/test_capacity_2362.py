"""Unit tests for lumen.capacity modules with inputs outside EC matrix."""
from __future__ import annotations

import unittest

from lumen.capacity.arbitration import decide_capacity
from lumen.capacity.catalog import select_profile
from lumen.capacity.projection import evaluate_downgrade
from lumen.capacity.spec import (
    CapacityInput,
    CapacityPolicy,
    CapacitySignals,
    CapacityState,
    ProfileAvailability,
    ProfileCatalog,
    SyntheticClock,
    TransitionGraph,
)
from lumen.capacity.status import CapacityStatus
from lumen.capacity.verdict import ActionKind


class TestCapacityArbitration2362(unittest.TestCase):
    def test_arbitration_custom_generations_and_clock(self) -> None:
        policy = CapacityPolicy(
            cooldown_seconds=600,
            scale_out_sustained_seconds=120,
            scale_in_sustained_seconds=3600,
        )
        clock = SyntheticClock(now=5_000)

        # Generation mismatch with custom generation 42 vs 43
        res_mismatch = decide_capacity(
            CapacityInput(
                signals=CapacitySignals(disk_pressure=True, signal_generation=42),
                state=CapacityState(current_generation=43),
                policy=policy,
            ),
            clock,
        )
        self.assertEqual(res_mismatch.action.kind, ActionKind.HOLD)
        self.assertEqual(res_mismatch.field_path, "signal_generation")

        # Cooldown check with custom cooldown 600
        res_cooldown = decide_capacity(
            CapacityInput(
                signals=CapacitySignals(disk_pressure=True, signal_generation=42),
                state=CapacityState(current_generation=42, last_change_at=4_500),
                policy=policy,
            ),
            clock,
        )
        self.assertEqual(res_cooldown.action.kind, ActionKind.HOLD)
        self.assertEqual(res_cooldown.reason, "cooldown")

        # Expired cooldown -> PVC_GROW
        clock_after_cooldown = SyntheticClock(now=5_601)
        res_ok = decide_capacity(
            CapacityInput(
                signals=CapacitySignals(disk_pressure=True, signal_generation=42),
                state=CapacityState(current_generation=42, last_change_at=4_500),
                policy=policy,
            ),
            clock_after_cooldown,
        )
        self.assertEqual(res_ok.action.kind, ActionKind.PVC_GROW)

    def test_arbitration_priority_combinations(self) -> None:
        policy = CapacityPolicy.default()
        clock = SyntheticClock(now=10_000)

        # Disk pressure + Memory pressure -> Disk wins (PVC_GROW)
        res_disk_mem = decide_capacity(
            CapacityInput(
                signals=CapacitySignals(disk_pressure=True, memory_pressure=True),
                state=CapacityState(),
                policy=policy,
            ),
            clock,
        )
        self.assertEqual(res_disk_mem.action.kind, ActionKind.PVC_GROW)

        # Write pressure + Memory pressure -> Machine upgrade
        res_write_mem = decide_capacity(
            CapacityInput(
                signals=CapacitySignals(
                    write_cpu_pressure=True, memory_pressure=True
                ),
                state=CapacityState(),
                policy=policy,
            ),
            clock,
        )
        self.assertEqual(res_write_mem.action.kind, ActionKind.MACHINE_UPGRADE)

    def test_catalog_selection_arbitrary_profiles(self) -> None:
        catalog = ProfileCatalog(
            installed=("db-small", "db-medium", "db-large"),
            availability={
                "db-small": ProfileAvailability.AVAILABLE,
                "db-medium": ProfileAvailability.AVAILABLE,
                "db-large": ProfileAvailability.FULL,
            },
        )
        graph = TransitionGraph(
            edges={
                "db-small": ("db-medium",),
                "db-medium": ("db-large",),
            }
        )

        # Valid transition
        sel1 = select_profile(catalog, graph, "db-small", "db-medium")
        self.assertEqual(sel1.profile, "db-medium")
        self.assertEqual(sel1.reason, "ok")

        # Full target profile
        sel2 = select_profile(catalog, graph, "db-medium", "db-large")
        self.assertIsNone(sel2.profile)
        self.assertEqual(sel2.reason, "CapacityBlocked")
        self.assertEqual(sel2.field_path, "availability")

        # Invalid transition edge (db-small -> db-large directly)
        sel3 = select_profile(catalog, graph, "db-small", "db-large")
        self.assertIsNone(sel3.profile)
        self.assertEqual(sel3.reason, "CapacityBlocked")

    def test_projection_custom_headroom(self) -> None:
        # Headroom 30 -> max allowed 70
        signals = CapacitySignals(
            cpu_p95=65.0,
            memory_p95=75.0,
            compaction_p95=10.0,
            recovery_p95=10.0,
            system_reserve_p95=10.0,
        )

        res = evaluate_downgrade(signals, "n1-standard-8", headroom=30.0)
        self.assertEqual(res.action.kind, ActionKind.HOLD)
        self.assertEqual(res.failing_constraint, "memory_or_working_set")

        # With headroom 20 -> max allowed 80 -> ok
        res_pass = evaluate_downgrade(signals, "n1-standard-8", headroom=20.0)
        self.assertEqual(res_pass.action.kind, ActionKind.MACHINE_DOWNGRADE)
        self.assertEqual(res_pass.action.target, "n1-standard-8")
        self.assertIsNone(res_pass.failing_constraint)

    def test_capacity_status_generation_bound(self) -> None:
        st_ok = CapacityStatus(
            recommendation_generation=100,
            action_generation=100,
            status_generation=100,
        )
        self.assertTrue(st_ok.is_generation_bound())

        st_bad = CapacityStatus(
            recommendation_generation=100,
            action_generation=100,
            status_generation=101,
        )
        self.assertFalse(st_bad.is_generation_bound())


if __name__ == "__main__":
    unittest.main()
