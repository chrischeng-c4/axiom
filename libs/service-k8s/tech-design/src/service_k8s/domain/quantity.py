from __future__ import annotations

from dataclasses import dataclass
from typing import Final, Union

SUFFIXES: Final[tuple[tuple[str, int], ...]] = (
    ("Ei", 1152921504606846976),
    ("Pi", 1125899906842624),
    ("Ti", 1099511627776),
    ("Gi", 1073741824),
    ("Mi", 1048576),
    ("Ki", 1024),
    ("E", 1000000000000000000),
    ("P", 1000000000000000),
    ("T", 1000000000000),
    ("G", 1000000000),
    ("M", 1000000),
    ("k", 1000),
)


class QuantityError(ValueError):
    """Base exception for quantity parsing errors."""


def parse_storage_bytes(qty: str) -> int:
    s = qty.strip()
    if not s:
        raise QuantityError("empty storage quantity")

    for suffix, multiplier in SUFFIXES:
        if s.endswith(suffix):
            num_str = s[: -len(suffix)].strip()
            try:
                val = float(num_str)
            except ValueError:
                raise QuantityError(
                    f"invalid numeric part in storage quantity '{qty}'"
                )
            if val < 0:
                raise QuantityError(f"negative storage quantity '{qty}'")
            return int(round(val * multiplier))

    try:
        val_int = int(s)
    except ValueError:
        raise QuantityError(f"unrecognized storage quantity '{qty}'")

    if val_int < 0:
        raise QuantityError(f"negative storage quantity '{qty}'")

    return val_int


@dataclass(frozen=True)
class Grow:
    current_bytes: int
    desired_bytes: int


@dataclass(frozen=True)
class NoOp:
    pass


@dataclass(frozen=True)
class ShrinkUnsupported:
    current_bytes: int
    desired_bytes: int


@dataclass(frozen=True)
class Unparseable:
    detail: str


ResizeAction = Union[Grow, NoOp, ShrinkUnsupported, Unparseable]


def decide(current: str, desired: str) -> ResizeAction:
    try:
        current_bytes = parse_storage_bytes(current)
    except QuantityError as e:
        return Unparseable(f"current quantity '{current}': {e}")

    try:
        desired_bytes = parse_storage_bytes(desired)
    except QuantityError as e:
        return Unparseable(f"desired quantity '{desired}': {e}")

    if desired_bytes > current_bytes:
        return Grow(current_bytes, desired_bytes)
    if desired_bytes == current_bytes:
        return NoOp()
    return ShrinkUnsupported(current_bytes, desired_bytes)
