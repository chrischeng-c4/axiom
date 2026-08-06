from __future__ import annotations

from metrics_prometheus.domain.sample import Label


def canonical(labels: tuple[Label, ...]) -> tuple[Label, ...]:
    return tuple(sorted(labels, key=lambda l: (l.name, l.value)))
