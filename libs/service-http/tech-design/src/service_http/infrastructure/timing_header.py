from __future__ import annotations

from service_http.domain.timing import (
    BASELINE_METRIC,
    DURATION_PARAM,
    EXTRA_TOKEN_CHARS,
    FALLBACK_TOKEN,
    METRIC_SEPARATOR,
    REPLACEMENT_CHAR,
    Disclosure,
    Phase,
    reveals_phases,
)

NANOS_PER_MILLISECOND = 1_000_000
MS_DECIMAL_PLACES = 3


def format_ms(duration_ns: int) -> str:
    ms = duration_ns / NANOS_PER_MILLISECOND
    return f"{ms:.{MS_DECIMAL_PLACES}f}"


def sanitize_token(name: str) -> str:
    out = "".join(
        c
        if (c.isascii() and c.isalnum()) or c in EXTRA_TOKEN_CHARS
        else REPLACEMENT_CHAR
        for c in name
    )
    if out == "":
        return FALLBACK_TOKEN
    return out


def render_metric(name: str, duration_ns: int) -> str:
    return sanitize_token(name) + DURATION_PARAM + format_ms(duration_ns)


def render_header(
    total_ns: int,
    disclosure: Disclosure,
    phases: tuple[Phase, ...],
) -> str:
    header = BASELINE_METRIC + DURATION_PARAM + format_ms(total_ns)
    if reveals_phases(disclosure):
        for phase in phases:
            header += METRIC_SEPARATOR + render_metric(
                phase.name, phase.duration_ns
            )
    return header
