from __future__ import annotations

from dataclasses import dataclass, field

from raft_runtime.domain.snapshot import (
    Disabled,
    SnapshotPolicy,
    compactable_upto,
    should_snapshot,
)

DEFAULT_TICK_MS: int = 20
DEFAULT_PUMP_MS: int = 5
DEFAULT_RPC_TIMEOUT_MS: int = 400
DEFAULT_PROPOSE_TIMEOUT_MS: int = 10_000
PROPOSE_RETRY_MS: int = 20


@dataclass(frozen=True, slots=True)
class HostConfig:
    tick_ms: int = DEFAULT_TICK_MS
    pump_ms: int = DEFAULT_PUMP_MS
    rpc_timeout_ms: int = DEFAULT_RPC_TIMEOUT_MS
    propose_timeout_ms: int = DEFAULT_PROPOSE_TIMEOUT_MS
    snapshot_policy: SnapshotPolicy = field(default_factory=Disabled)


def drain_budget_ms(config: HostConfig) -> int:
    return 2 * config.rpc_timeout_ms


def propose_attempts(config: HostConfig) -> int:
    return config.propose_timeout_ms // PROPOSE_RETRY_MS


def compact_upto(
    config: HostConfig, applied_index: int, last_snapshot_index: int
) -> int:
    if applied_index == 0:
        return 0
    if not should_snapshot(
        config.snapshot_policy, applied_index, last_snapshot_index
    ):
        return 0
    return compactable_upto(applied_index)
