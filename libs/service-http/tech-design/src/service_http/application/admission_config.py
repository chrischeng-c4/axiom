from __future__ import annotations

from dataclasses import dataclass
from typing import Callable

from service_http.domain.admission import (
    DEFAULT_MAX_KEYS,
    DEFAULT_REFILL_SECS,
    NANOS_PER_SECOND,
    AdmissionPolicy,
    policy_problem,
)
from service_http.domain.errors import (
    InvalidPolicy,
    InvalidValue,
    OrphanedCommonSetting,
)
from service_http.infrastructure.env import (
    ADMIN_CAPACITY_SUFFIX,
    MAX_KEYS_SUFFIX,
    READ_CAPACITY_SUFFIX,
    REFILL_SECS_SUFFIX,
    WRITE_CAPACITY_SUFFIX,
    all_keys,
    env_key,
)
from service_http.infrastructure.numbers import parse_positive


@dataclass(frozen=True)
class AdmissionConfig:
    read_capacity: int | None
    write_capacity: int | None
    admin_capacity: int | None
    refill_secs: int
    max_keys: int


def from_lookup(
    prefix: str,
    lookup: Callable[[str], str | None],
) -> AdmissionConfig | InvalidValue | OrphanedCommonSetting:
    keys = all_keys(prefix)
    raws = {k: lookup(k) for k in keys}

    for k in keys:
        raw = raws[k]
        if raw is not None and parse_positive(raw) is None:
            return InvalidValue(k, raw)

    read = parse_positive(raws[env_key(prefix, READ_CAPACITY_SUFFIX)])
    write = parse_positive(raws[env_key(prefix, WRITE_CAPACITY_SUFFIX)])
    admin = parse_positive(raws[env_key(prefix, ADMIN_CAPACITY_SUFFIX)])
    refill = parse_positive(raws[env_key(prefix, REFILL_SECS_SUFFIX)])
    max_k = parse_positive(raws[env_key(prefix, MAX_KEYS_SUFFIX)])

    enabled = (read is not None) or (write is not None) or (admin is not None)
    if not enabled:
        if raws[env_key(prefix, REFILL_SECS_SUFFIX)] is not None:
            return OrphanedCommonSetting(env_key(prefix, REFILL_SECS_SUFFIX))
        if raws[env_key(prefix, MAX_KEYS_SUFFIX)] is not None:
            return OrphanedCommonSetting(env_key(prefix, MAX_KEYS_SUFFIX))

    return AdmissionConfig(
        read_capacity=read,
        write_capacity=write,
        admin_capacity=admin,
        refill_secs=refill if refill is not None else DEFAULT_REFILL_SECS,
        max_keys=max_k if max_k is not None else DEFAULT_MAX_KEYS,
    )


def is_enabled(config: AdmissionConfig) -> bool:
    return (
        (config.read_capacity is not None)
        or (config.write_capacity is not None)
        or (config.admin_capacity is not None)
    )


def policies(
    config: AdmissionConfig,
    read_class: str,
    write_class: str,
    admin_class: str,
) -> dict[str, AdmissionPolicy] | InvalidPolicy:
    window_ns = config.refill_secs * NANOS_PER_SECOND
    built: dict[str, AdmissionPolicy] = {}
    items = [
        (config.read_capacity, read_class),
        (config.write_capacity, write_class),
        (config.admin_capacity, admin_class),
    ]
    for capacity, class_name in items:
        if capacity is None:
            continue
        policy = AdmissionPolicy(capacity, window_ns, config.max_keys)
        prob = policy_problem(policy)
        if prob is not None:
            return InvalidPolicy(class_name, prob)
        built[class_name] = policy
    return built


def controller_policies(
    config: AdmissionConfig,
    read_class: str,
    write_class: str,
    admin_class: str,
) -> dict[str, AdmissionPolicy] | InvalidPolicy | None:
    if not is_enabled(config):
        return None
    return policies(config, read_class, write_class, admin_class)
