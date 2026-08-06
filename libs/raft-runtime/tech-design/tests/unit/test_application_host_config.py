from __future__ import annotations

from dataclasses import MISSING
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from raft_runtime.application.host_config import (
    DEFAULT_PROPOSE_TIMEOUT_MS,
    DEFAULT_PUMP_MS,
    DEFAULT_RPC_TIMEOUT_MS,
    DEFAULT_TICK_MS,
    PROPOSE_RETRY_MS,
    HostConfig,
    compact_upto,
    drain_budget_ms,
    propose_attempts,
)
from raft_runtime.domain.snapshot import (
    DEFAULT_SNAPSHOT_POLICY,
    Disabled,
    EveryEntries,
    External,
)


class TestApplicationHostConfig(unittest.TestCase):
    def test_host_config_constants(self) -> None:
        self.assertEqual(DEFAULT_TICK_MS, 20)
        self.assertEqual(DEFAULT_PUMP_MS, 5)
        self.assertEqual(DEFAULT_RPC_TIMEOUT_MS, 400)
        self.assertEqual(DEFAULT_PROPOSE_TIMEOUT_MS, 10_000)
        self.assertEqual(PROPOSE_RETRY_MS, 20)

    def test_host_config_defaults_and_factory(self) -> None:
        cfg = HostConfig()
        self.assertEqual(cfg.tick_ms, 20)
        self.assertEqual(cfg.pump_ms, 5)
        self.assertEqual(cfg.rpc_timeout_ms, 400)
        self.assertEqual(cfg.propose_timeout_ms, 10_000)
        self.assertEqual(cfg.snapshot_policy, Disabled())
        self.assertEqual(cfg.snapshot_policy, DEFAULT_SNAPSHOT_POLICY)

        field_spec = HostConfig.__dataclass_fields__["snapshot_policy"]
        self.assertIs(field_spec.default, MISSING)
        self.assertIs(field_spec.default_factory, Disabled)

    def test_drain_budget_ms(self) -> None:
        self.assertEqual(drain_budget_ms(HostConfig()), 800)
        self.assertEqual(
            drain_budget_ms(HostConfig(rpc_timeout_ms=250)), 500
        )

    def test_propose_attempts_floors_down(self) -> None:
        self.assertEqual(propose_attempts(HostConfig()), 500)
        self.assertEqual(
            propose_attempts(HostConfig(propose_timeout_ms=9_999)), 499
        )
        self.assertEqual(
            propose_attempts(HostConfig(propose_timeout_ms=10)), 0
        )

    def test_compact_upto_applied_zero_guard_precedes_policy(self) -> None:
        cfg = HostConfig(snapshot_policy=EveryEntries(1))
        self.assertEqual(compact_upto(cfg, 0, 0), 0)
        self.assertEqual(compact_upto(cfg, 5, 0), 5)

    def test_compact_upto_external_policy_returns_zero(self) -> None:
        cfg = HostConfig(snapshot_policy=External())
        self.assertEqual(compact_upto(cfg, 10**6, 0), 0)

    def test_compact_upto_disabled_policy_returns_zero(self) -> None:
        cfg = HostConfig()
        self.assertEqual(compact_upto(cfg, 0, 0), 0)
        self.assertEqual(compact_upto(cfg, 1, 0), 0)
        self.assertEqual(compact_upto(cfg, 10**6, 0), 0)

    def test_compact_upto_every_entries_policy(self) -> None:
        cfg = HostConfig(snapshot_policy=EveryEntries(100))
        self.assertEqual(compact_upto(cfg, 99, 0), 0)
        self.assertEqual(compact_upto(cfg, 100, 0), 100)
        self.assertEqual(compact_upto(cfg, 150, 100), 0)


if __name__ == "__main__":
    unittest.main()
