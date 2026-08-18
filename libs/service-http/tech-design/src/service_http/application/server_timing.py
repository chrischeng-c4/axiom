from __future__ import annotations

from service_http.domain.timing import Disclosure, Phase, drains_phases
from service_http.infrastructure.timing_header import render_header


class PhaseCollector:
    def __init__(self) -> None:
        self._phases: list[Phase] = []

    def push(self, name: str, duration_ns: int) -> None:
        self._phases.append(Phase(name, duration_ns))

    def pending(self) -> tuple[Phase, ...]:
        return tuple(self._phases)

    def render(self, total_ns: int, disclosure: Disclosure) -> str:
        header = render_header(total_ns, disclosure, self.pending())
        if drains_phases(disclosure):
            self._phases.clear()
        return header
