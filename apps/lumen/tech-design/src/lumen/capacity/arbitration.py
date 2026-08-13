"""Pure decision engine for Lumen capacity arbitration."""
from __future__ import annotations

from typing import Final

from lumen.capacity.spec import CapacityInput, SyntheticClock
from lumen.capacity.verdict import ActionKind, CapacityAction, CapacityDecision

__aw_artifact_id__: Final[str] = "artifact:lumen/capacity/arbitration"


def decide_capacity(
    input: CapacityInput,
    clock: SyntheticClock | None = None,
) -> CapacityDecision:
    """Arbitrate single safe capacity or topology action based on signals and policy."""
    signals = input.signals
    state = input.state
    policy = input.policy

    # 1. R1 Telemetry completeness check
    if not signals.telemetry_complete:
        return CapacityDecision(
            action=CapacityAction(kind=ActionKind.HOLD),
            reason="HOLD",
            field_path="telemetry_complete",
        )

    # 2. R1 Telemetry freshness check
    if not signals.telemetry_fresh:
        return CapacityDecision(
            action=CapacityAction(kind=ActionKind.HOLD),
            reason="HOLD",
            field_path="telemetry_fresh",
        )

    # 3. R1 Generation binding check
    if signals.signal_generation != state.current_generation:
        return CapacityDecision(
            action=CapacityAction(kind=ActionKind.HOLD),
            reason="HOLD",
            field_path="signal_generation",
        )

    # 4. R1 Mutation fencing check
    if state.mutation_active:
        return CapacityDecision(
            action=CapacityAction(kind=ActionKind.HOLD),
            reason="HOLD",
            field_path="mutation_active",
        )

    # 5. R7 Cooldown check
    if state.last_change_at is not None and clock is not None:
        elapsed = clock.now - state.last_change_at
        if elapsed < policy.cooldown_seconds:
            return CapacityDecision(
                action=CapacityAction(kind=ActionKind.HOLD),
                reason="cooldown",
                field_path="last_change_at",
            )

    # 6. R7 Automatic change limit check
    if state.automatic_change_limit_reached:
        return CapacityDecision(
            action=CapacityAction(kind=ActionKind.HOLD),
            reason="automatic_change_limit",
            field_path="automatic_change_limit_reached",
        )

    # 7. R7 Deadband check
    if signals.within_deadband:
        return CapacityDecision(
            action=CapacityAction(kind=ActionKind.HOLD),
            reason="deadband",
            field_path="within_deadband",
        )

    # 8. Expansion signals (R2, R3, R4, R5) - Expansion outranks contraction
    if signals.disk_pressure:
        if state.capacity_ceiling_reached or state.io_ceiling_reached:
            return CapacityDecision(
                action=CapacityAction(kind=ActionKind.SPLIT),
                reason="ok",
                field_path="",
            )
        return CapacityDecision(
            action=CapacityAction(kind=ActionKind.PVC_GROW),
            reason="ok",
            field_path="",
        )

    if (
        signals.write_cpu_pressure
        or signals.compaction_cpu_pressure
        or signals.recovery_cpu_pressure
    ):
        if state.vertical_ceiling_reached:
            return CapacityDecision(
                action=CapacityAction(kind=ActionKind.SPLIT),
                reason="ok",
                field_path="",
            )
        return CapacityDecision(
            action=CapacityAction(kind=ActionKind.MACHINE_UPGRADE),
            reason="ok",
            field_path="",
        )

    if signals.memory_pressure:
        return CapacityDecision(
            action=CapacityAction(kind=ActionKind.HIGHMEM_UPGRADE),
            reason="ok",
            field_path="",
        )

    if signals.read_dominated:
        if signals.sustained_since is not None and clock is not None:
            elapsed_sustained = clock.now - signals.sustained_since
            if elapsed_sustained < policy.scale_out_sustained_seconds:
                return CapacityDecision(
                    action=CapacityAction(kind=ActionKind.HOLD),
                    reason="post_convergence_window",
                    field_path="sustained_since",
                )
        return CapacityDecision(
            action=CapacityAction(kind=ActionKind.READ_REPLICA),
            reason="ok",
            field_path="",
        )

    # 9. Contraction signals (R5, R6, R7)
    if signals.low_utilization:
        if state.converged_at is not None and signals.window_started_at is not None:
            if signals.window_started_at <= state.converged_at:
                return CapacityDecision(
                    action=CapacityAction(kind=ActionKind.HOLD),
                    reason="post_convergence_window",
                    field_path="window_started_at",
                )

        if signals.window_started_at is not None and clock is not None:
            elapsed_window = clock.now - signals.window_started_at
            if elapsed_window < policy.scale_in_sustained_seconds:
                return CapacityDecision(
                    action=CapacityAction(kind=ActionKind.HOLD),
                    reason="post_convergence_window",
                    field_path="window_started_at",
                )

        if state.excess_read_replicas > 0:
            return CapacityDecision(
                action=CapacityAction(kind=ActionKind.READ_REPLICA_REMOVE),
                reason="ok",
                field_path="",
            )
        return CapacityDecision(
            action=CapacityAction(kind=ActionKind.MACHINE_DOWNGRADE),
            reason="ok",
            field_path="",
        )

    # Default to HOLD if no actionable signal
    return CapacityDecision(
        action=CapacityAction(kind=ActionKind.HOLD),
        reason="ok",
        field_path="",
    )
