from __future__ import annotations

import sys
import unittest
from dataclasses import replace
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from transport_h2c.application.supervision import (
    SweepPlan,
    plan_shutdown,
    plan_sweep,
)
from transport_h2c.infrastructure.config import default_config
from transport_h2c.infrastructure.connection import ConnectionState


class TestApplicationSupervision(unittest.TestCase):
    def setUp(self) -> None:
        self.cfg = default_config(8)

    def test_plan_sweep_mixed_pool_b7a(self) -> None:
        c1 = ConnectionState(id=1, healthy=True, in_flight=0, total=10, errors=1, last_used_ms=9000)
        c2 = ConnectionState(id=2, healthy=False, in_flight=1, total=5, errors=2, last_used_ms=0)
        c3 = ConnectionState(id=3, healthy=True, in_flight=2, total=7, errors=0, last_used_ms=0)
        c4 = ConnectionState(id=4, healthy=True, in_flight=0, total=3, errors=1, last_used_ms=0)

        plan = plan_sweep([c1, c2, c3, c4], self.cfg, now_ms=10_000)
        self.assertEqual(
            plan,
            SweepPlan(evicted=(2,), shrunk=4, retired_requests=8, retired_errors=3, replenish=0),
        )

    def test_plan_sweep_one_shed_limit_b7b(self) -> None:
        conns = [
            ConnectionState(id=i, healthy=True, in_flight=0, total=1, errors=0, last_used_ms=0)
            for i in range(1, 6)
        ]
        plan = plan_sweep(conns, self.cfg, now_ms=10_000)
        self.assertEqual(
            plan,
            SweepPlan(evicted=(), shrunk=1, retired_requests=1, retired_errors=0, replenish=0),
        )

    def test_plan_sweep_min_connections_guard_b7c(self) -> None:
        cfg = replace(self.cfg, min_connections=2)
        c1 = ConnectionState(id=1, healthy=True, in_flight=0, total=4, errors=1, last_used_ms=0)
        c2 = ConnectionState(id=2, healthy=True, in_flight=0, total=6, errors=2, last_used_ms=0)

        plan = plan_sweep([c1, c2], cfg, now_ms=10_000)
        self.assertEqual(
            plan,
            SweepPlan(evicted=(), shrunk=None, retired_requests=0, retired_errors=0, replenish=0),
        )

    def test_plan_sweep_busy_survivors_excluded_b7d(self) -> None:
        c1 = ConnectionState(id=1, healthy=True, in_flight=1, total=4, errors=0, last_used_ms=0)
        c2 = ConnectionState(id=2, healthy=True, in_flight=3, total=4, errors=0, last_used_ms=0)
        c3 = ConnectionState(id=3, healthy=True, in_flight=2, total=4, errors=0, last_used_ms=0)

        plan = plan_sweep([c1, c2, c3], self.cfg, now_ms=10_000)
        self.assertEqual(
            plan,
            SweepPlan(evicted=(), shrunk=None, retired_requests=0, retired_errors=0, replenish=0),
        )

    def test_plan_sweep_keepalive_ceiling_disjunct_b7e(self) -> None:
        cfg = replace(self.cfg, max_keepalive_connections=2)
        conns = [
            ConnectionState(id=i, healthy=True, in_flight=0, total=1, errors=0, last_used_ms=10_000)
            for i in range(1, 4)
        ]
        plan = plan_sweep(conns, cfg, now_ms=10_000)
        self.assertEqual(
            plan,
            SweepPlan(evicted=(), shrunk=1, retired_requests=1, retired_errors=0, replenish=0),
        )

    def test_plan_sweep_ceiling_floor_min_connections_b7f(self) -> None:
        cfg = replace(self.cfg, min_connections=3, max_keepalive_connections=1)
        conns = [
            ConnectionState(id=i, healthy=True, in_flight=0, total=1, errors=0, last_used_ms=10_000)
            for i in range(1, 4)
        ]
        plan = plan_sweep(conns, cfg, now_ms=10_000)
        self.assertEqual(
            plan,
            SweepPlan(evicted=(), shrunk=None, retired_requests=0, retired_errors=0, replenish=0),
        )

    def test_plan_sweep_eviction_replenish_b7g(self) -> None:
        cfg = replace(self.cfg, min_connections=2)
        c1 = ConnectionState(id=1, healthy=False, in_flight=0, total=3, errors=3, last_used_ms=0)
        c2 = ConnectionState(id=2, healthy=False, in_flight=0, total=4, errors=0, last_used_ms=0)
        c3 = ConnectionState(id=3, healthy=True, in_flight=1, total=2, errors=0, last_used_ms=0)

        plan = plan_sweep([c1, c2, c3], cfg, now_ms=10_000)
        self.assertEqual(
            plan,
            SweepPlan(evicted=(1, 2), shrunk=None, retired_requests=7, retired_errors=3, replenish=1),
        )

    def test_plan_sweep_all_dead_b7h(self) -> None:
        c1 = ConnectionState(id=1, healthy=False, in_flight=0, total=5, errors=5, last_used_ms=0)
        plan = plan_sweep([c1], self.cfg, now_ms=10_000)
        self.assertEqual(
            plan,
            SweepPlan(evicted=(1,), shrunk=None, retired_requests=5, retired_errors=5, replenish=1),
        )

    def test_plan_sweep_empty_pool_b7i(self) -> None:
        plan = plan_sweep([], self.cfg, now_ms=0)
        self.assertEqual(
            plan,
            SweepPlan(evicted=(), shrunk=None, retired_requests=0, retired_errors=0, replenish=1),
        )

    def test_plan_sweep_no_immediate_reopen_b7j(self) -> None:
        cfg = replace(self.cfg, min_connections=2)
        conns = [
            ConnectionState(id=i, healthy=True, in_flight=0, total=2, errors=0, last_used_ms=0)
            for i in range(1, 4)
        ]
        plan = plan_sweep(conns, cfg, now_ms=10_000)
        self.assertEqual(
            plan,
            SweepPlan(evicted=(), shrunk=1, retired_requests=2, retired_errors=0, replenish=0),
        )

    def test_plan_shutdown_totals(self) -> None:
        self.assertEqual(plan_shutdown([]), (0, 0))

        c1 = ConnectionState(id=1, healthy=True, in_flight=0, total=10, errors=1, last_used_ms=9000)
        c2 = ConnectionState(id=2, healthy=False, in_flight=1, total=5, errors=2, last_used_ms=0)
        c3 = ConnectionState(id=3, healthy=True, in_flight=2, total=7, errors=0, last_used_ms=0)
        c4 = ConnectionState(id=4, healthy=True, in_flight=0, total=3, errors=1, last_used_ms=0)
        self.assertEqual(plan_shutdown([c1, c2, c3, c4]), (25, 4))

        c_dead = ConnectionState(id=1, healthy=False, in_flight=0, total=5, errors=5, last_used_ms=0)
        self.assertEqual(plan_shutdown([c_dead]), (5, 5))


if __name__ == "__main__":
    unittest.main()
