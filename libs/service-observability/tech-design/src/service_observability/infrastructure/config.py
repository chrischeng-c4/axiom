from __future__ import annotations

from dataclasses import dataclass
from enum import Enum


class LogFormat(Enum):
    JSON = "json"
    PRETTY = "pretty"


@dataclass(frozen=True)
class ObservabilityConfig:
    log_level: str = "info"
    log_format: LogFormat = LogFormat.JSON
    otlp_endpoint: str | None = None


def collector_compatible(log_format: LogFormat) -> bool:
    return log_format is LogFormat.JSON
