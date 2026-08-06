from __future__ import annotations

from dataclasses import dataclass, field
from typing import Protocol, Sequence


@dataclass
class Counter:
    value: int = 0

    def incr(self) -> None:
        self.value += 1

    def get(self) -> int:
        return self.value


@dataclass(frozen=True)
class Sample:
    name: str
    kind: str
    help_text: str
    value: int


class MetricsProvider(Protocol):
    def render_metrics(self) -> str:
        ...


def render(samples: Sequence[Sample]) -> str:
    """Prometheus text exposition. Each sample contributes exactly three lines."""
    lines: list[str] = []
    for s in samples:
        lines.append(f"# HELP {s.name} {s.help_text}")
        lines.append(f"# TYPE {s.name} {s.kind}")
        lines.append(f"{s.name} {s.value}")
    return "".join(line + "\n" for line in lines)


@dataclass
class LifecycleMetrics:
    accepted_counter: Counter = field(default_factory=Counter)
    rejected_counter: Counter = field(default_factory=Counter)
    closed_counter: Counter = field(default_factory=Counter)

    def connection_accepted(self) -> None:
        self.accepted_counter.incr()

    def connection_rejected(self) -> None:
        self.rejected_counter.incr()

    def connection_closed(self) -> None:
        self.closed_counter.incr()

    def accepted(self) -> int:
        return self.accepted_counter.get()

    def rejected(self) -> int:
        return self.rejected_counter.get()

    def closed(self) -> int:
        return self.closed_counter.get()

    def render_metrics(self) -> str:
        return render(
            (
                Sample(
                    "service_connections_accepted_total",
                    "counter",
                    "Total accepted service connections.",
                    self.accepted(),
                ),
                Sample(
                    "service_connections_rejected_total",
                    "counter",
                    "Total service connections rejected by admission.",
                    self.rejected(),
                ),
                Sample(
                    "service_connections_closed_total",
                    "counter",
                    "Total completed or failed service connections.",
                    self.closed(),
                ),
            )
        )
