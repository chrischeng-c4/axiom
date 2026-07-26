"""Health projection for typed intake work items.

@spec #2598
"""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class IntakeQueueHealth:
    open_report_count: int
    expired_spike_count: int
    next_command: str | None

    @property
    def status(self) -> str:
        return "pending" if self.next_command else "clean"


def remediation(open_reports: tuple[str, ...], expired_spikes: tuple[str, ...]) -> str | None:
    if open_reports:
        return f"aw wi triage {sorted(open_reports)[0]} --verdict accepted"
    if expired_spikes:
        return f"aw wi spike expire {sorted(expired_spikes)[0]}"
    return None
