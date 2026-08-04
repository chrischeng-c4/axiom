"""In-memory AuthEventSink adapter that records events for inspection."""

from __future__ import annotations

from dataclasses import dataclass, field

from service_auth.domain.audit import AuthEvent


@dataclass
class RecordingEventSink:
    events: list[AuthEvent] = field(default_factory=list)

    def record(self, event: AuthEvent) -> None:
        self.events.append(event)
