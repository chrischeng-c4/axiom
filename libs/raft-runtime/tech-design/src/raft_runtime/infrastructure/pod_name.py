from __future__ import annotations

from dataclasses import dataclass

from raft_runtime.domain.errors import BadOrdinal, NamelessPod, TopologyError

ASCII_DIGITS: str = "0123456789"


@dataclass(frozen=True, slots=True)
class PodIdentity:
    prefix: str  # the StatefulSet name
    ordinal: int


def split_pod_name(pod_name: str) -> PodIdentity | TopologyError:
    if "-" not in pod_name:
        return NamelessPod(pod_name=pod_name)
    last_hyphen_idx = pod_name.rfind("-")
    prefix = pod_name[:last_hyphen_idx]
    suffix = pod_name[last_hyphen_idx + 1 :]
    if not prefix:
        return NamelessPod(pod_name=pod_name)
    if not suffix or not all(c in ASCII_DIGITS for c in suffix):
        return BadOrdinal(pod_name=pod_name, suffix=suffix)
    return PodIdentity(prefix=prefix, ordinal=int(suffix))
