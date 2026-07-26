"""Terminal convergence contract for timeboxed Spike work items.

@spec #2595
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import StrEnum


class SpikeTerminalState(StrEnum):
    DECIDED = "decided"
    GAVE_UP = "gave_up"


@dataclass(frozen=True)
class SpikeDecision:
    state: SpikeTerminalState
    decision: str
    spawned_work_items: tuple[str, ...] = ()
    no_action: bool = False

    def is_terminal(self) -> bool:
        if self.state is SpikeTerminalState.GAVE_UP:
            return True
        return bool(self.decision) and (bool(self.spawned_work_items) ^ self.no_action)


def product_source_edits_allowed(work_item_type: str) -> bool:
    """Spike is evidence-only; product source starts from a spawned Change."""

    return work_item_type == "change"
