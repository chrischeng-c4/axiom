from __future__ import annotations

from metrics_prometheus.application.accumulators import Histogram
from metrics_prometheus.domain.bucket import cumulative
from metrics_prometheus.domain.escaping import escape_label_value
from metrics_prometheus.domain.label_order import canonical
from metrics_prometheus.domain.sample import Label, Sample, SampleGroup
from metrics_prometheus.domain.scaling import scale_decimal


def render(samples: tuple[Sample, ...]) -> str:
    lines: list[str] = []
    for s in samples:
        lines.append(f"# HELP {s.name} {s.help}")
        lines.append(f"# TYPE {s.name} {s.kind.value}")
        lines.append(f"{s.name} {s.value}")
    return "".join(line + "\n" for line in lines)


def render_labeled(groups: tuple[SampleGroup, ...]) -> str:
    lines: list[str] = []
    for g in groups:
        lines.append(f"# HELP {g.name} {g.help}")
        lines.append(f"# TYPE {g.name} {g.kind.value}")
        for sample in g.samples:
            lines.append(f"{g.name}{render_label_set(sample.labels)} {sample.value}")
    return "".join(line + "\n" for line in lines)


def render_label_set(labels: tuple[Label, ...]) -> str:
    if not labels:
        return ""
    parts = [f'{l.name}="{escape_label_value(l.value)}"' for l in canonical(labels)]
    return "{" + ",".join(parts) + "}"


def render_histogram(histogram: Histogram, name: str, help: str, divisor: int) -> str:
    lines = [f"# HELP {name} {help}", f"# TYPE {name} histogram"]
    for bucket, running in zip(histogram.bounds, cumulative(histogram.bucket_counts())):
        lines.append(f'{name}_bucket{{le="{bucket.label}"}} {running}')
    lines.append(f'{name}_bucket{{le="+Inf"}} {histogram.count()}')
    lines.append(f"{name}_sum {scale_decimal(histogram.sum(), divisor)}")
    lines.append(f"{name}_count {histogram.count()}")
    return "".join(line + "\n" for line in lines)
