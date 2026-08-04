from __future__ import annotations

from service_http.infrastructure.numbers import (
    parse_ascii_unsigned,
    parse_positive,
)

ADMISSION_INFIX = "_ADMISSION_"
READ_CAPACITY_SUFFIX = "READ_CAPACITY"
WRITE_CAPACITY_SUFFIX = "WRITE_CAPACITY"
ADMIN_CAPACITY_SUFFIX = "ADMIN_CAPACITY"
REFILL_SECS_SUFFIX = "REFILL_SECS"
MAX_KEYS_SUFFIX = "MAX_KEYS"
CAPACITY_SUFFIXES = (
    READ_CAPACITY_SUFFIX,
    WRITE_CAPACITY_SUFFIX,
    ADMIN_CAPACITY_SUFFIX,
)
COMMON_SUFFIXES = (REFILL_SECS_SUFFIX, MAX_KEYS_SUFFIX)


def env_key(prefix: str, suffix: str) -> str:
    return prefix + ADMISSION_INFIX + suffix


def capacity_keys(prefix: str) -> tuple[str, ...]:
    return tuple(env_key(prefix, s) for s in CAPACITY_SUFFIXES)


def common_keys(prefix: str) -> tuple[str, ...]:
    return tuple(env_key(prefix, s) for s in COMMON_SUFFIXES)


def all_keys(prefix: str) -> tuple[str, ...]:
    return capacity_keys(prefix) + common_keys(prefix)
