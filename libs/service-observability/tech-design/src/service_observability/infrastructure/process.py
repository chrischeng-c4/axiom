from __future__ import annotations

from dataclasses import dataclass
from typing import Final

RSS_KIB_TO_BYTES: Final[int] = 1024
U64_MAX: Final[int] = 2**64 - 1
ASCII_DIGITS: Final[str] = "0123456789"


@dataclass(frozen=True)
class ProcessUsage:
    cpu_seconds: float
    rss_bytes: int


class ProcessSampleError(ValueError):
    pass


def parse_cpu_time(value: str) -> float:
    if "-" in value:
        day_part, clock = value.split("-", 1)
        try:
            days = float(day_part)
        except ValueError as exc:
            raise ProcessSampleError(
                f"invalid day part in CPU time: {value}"
            ) from exc
    else:
        days, clock = 0.0, value

    try:
        fields = [float(f) for f in clock.split(":")]
    except ValueError as exc:
        raise ProcessSampleError(
            f"invalid clock fields in CPU time: {value}"
        ) from exc

    if len(fields) == 2:
        seconds = fields[0] * 60.0 + fields[1]
    elif len(fields) == 3:
        seconds = fields[0] * 3600.0 + fields[1] * 60.0 + fields[2]
    else:
        raise ProcessSampleError(f"unexpected ps CPU time value {value}")

    return days * 86400.0 + seconds


def parse_rss_kib(field: str) -> int:
    """Read one unsigned resident-size field exactly as the platform reports it.

    ``ps`` reports RSS as an unsigned decimal count of KiB. Accept only that:
    an optional ``+``, at least one ASCII digit, nothing else, and a magnitude
    the published range can hold. A sign, a digit grouping separator, or a
    non-ASCII numeral is malformed output, not a quantity to be interpreted.
    """
    body = field[1:] if field[:1] == "+" else field
    if body == "" or any(ch not in ASCII_DIGITS for ch in body):
        raise ProcessSampleError("ps RSS is not numeric")
    rss_kib = int(body)
    if rss_kib > U64_MAX:
        raise ProcessSampleError("ps RSS exceeds the representable range")
    return rss_kib


def parse_ps_usage(output: str) -> ProcessUsage:
    fields = output.split()
    if len(fields) < 1:
        raise ProcessSampleError("ps output is missing RSS")
    rss_kib = parse_rss_kib(fields[0])

    if len(fields) < 2:
        raise ProcessSampleError("ps output is missing CPU time")

    return ProcessUsage(
        cpu_seconds=parse_cpu_time(fields[1]),
        rss_bytes=min(rss_kib * RSS_KIB_TO_BYTES, U64_MAX),
    )
