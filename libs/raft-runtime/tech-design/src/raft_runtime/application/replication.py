from __future__ import annotations

from dataclasses import dataclass
from typing import Protocol, Sequence

from raft_runtime.domain.consensus import Command


class RaftStateMachine(Protocol):
    def apply(self, index: int, command: Command) -> None:
        ...

    def snapshot(self) -> bytes:
        ...

    def restore(self, blob: bytes) -> None:
        ...

    def applied_index(self) -> int:
        ...


@dataclass(frozen=True, slots=True)
class ApplyReport:
    applied: tuple[int, ...]
    skipped: tuple[int, ...]
    failed: tuple[int, ...]


def replay_plan(
    applied_floor: int, committed: Sequence[int]
) -> tuple[int, ...]:
    filtered = sorted(set(idx for idx in committed if idx > applied_floor))
    return tuple(filtered)


def apply_committed(
    machine: RaftStateMachine, entries: Sequence[tuple[int, Command]]
) -> ApplyReport:
    floor = machine.applied_index()
    sorted_entries = sorted(entries, key=lambda pair: pair[0])

    applied_list: list[int] = []
    skipped_list: list[int] = []
    failed_list: list[int] = []

    seen_indices: set[int] = set()

    for idx, cmd in sorted_entries:
        if idx <= floor or idx in seen_indices:
            skipped_list.append(idx)
            continue

        seen_indices.add(idx)

        try:
            machine.apply(idx, cmd)
            applied_list.append(idx)
        except Exception:
            applied_list.append(idx)
            failed_list.append(idx)

    return ApplyReport(
        applied=tuple(applied_list),
        skipped=tuple(skipped_list),
        failed=tuple(failed_list),
    )
