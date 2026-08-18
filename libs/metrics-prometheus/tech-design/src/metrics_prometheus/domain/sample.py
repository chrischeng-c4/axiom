from __future__ import annotations

from dataclasses import dataclass
from enum import Enum


class MetricKind(str, Enum):
    COUNTER = "counter"
    GAUGE = "gauge"
    HISTOGRAM = "histogram"


@dataclass(frozen=True)
class Label:
    name: str
    value: str


@dataclass(frozen=True)
class Sample:
    name: str
    kind: MetricKind
    help: str
    value: int


@dataclass(frozen=True)
class LabeledSample:
    labels: tuple[Label, ...]
    value: int


@dataclass(frozen=True)
class SampleGroup:
    name: str
    kind: MetricKind
    help: str
    samples: tuple[LabeledSample, ...]
