"""Evaluates downgrade headroom safety across resource constraints."""
from __future__ import annotations

from typing import Final

from lumen.capacity.spec import CapacitySignals
from lumen.capacity.verdict import ActionKind, CapacityAction, DowngradeVerdict

__aw_artifact_id__: Final[str] = "artifact:lumen/capacity/projection"


def evaluate_downgrade(
    signals: CapacitySignals,
    target_profile: str,
    headroom: float = 20.0,
) -> DowngradeVerdict:
    """Project p95 utilization plus required headroom onto target capacity.

    Checks CPU, memory/working set, compaction, recovery, and system reserve.
    """
    max_allowed = 100.0 - headroom

    if signals.cpu_p95 > max_allowed:
        return DowngradeVerdict(
            action=CapacityAction(kind=ActionKind.HOLD),
            failing_constraint="cpu",
            reason="HOLD",
            field_path="cpu",
        )

    if signals.memory_p95 > max_allowed:
        return DowngradeVerdict(
            action=CapacityAction(kind=ActionKind.HOLD),
            failing_constraint="memory_or_working_set",
            reason="HOLD",
            field_path="memory_or_working_set",
        )

    if signals.recovery_p95 > max_allowed:
        return DowngradeVerdict(
            action=CapacityAction(kind=ActionKind.HOLD),
            failing_constraint="recovery",
            reason="HOLD",
            field_path="recovery",
        )

    if signals.system_reserve_p95 > max_allowed:
        return DowngradeVerdict(
            action=CapacityAction(kind=ActionKind.HOLD),
            failing_constraint="system_reserve",
            reason="HOLD",
            field_path="system_reserve",
        )

    if signals.compaction_p95 > max_allowed:
        return DowngradeVerdict(
            action=CapacityAction(kind=ActionKind.HOLD),
            failing_constraint="compaction",
            reason="HOLD",
            field_path="compaction",
        )

    return DowngradeVerdict(
        action=CapacityAction(
            kind=ActionKind.MACHINE_DOWNGRADE, target=target_profile
        ),
        failing_constraint=None,
        reason="ok",
        field_path="",
    )
