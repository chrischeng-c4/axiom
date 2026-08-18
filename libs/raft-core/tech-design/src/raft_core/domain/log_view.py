from __future__ import annotations

from dataclasses import dataclass

from raft_core.domain.entry import LogEntry
from raft_core.domain.ids import Index, Term


@dataclass(frozen=True)
class LogView:
    snapshot_index: Index
    snapshot_term: Term
    entries: tuple[LogEntry, ...]  # entries[0] has index snapshot_index + 1

    def last_index(self) -> Index:
        return self.snapshot_index + len(self.entries)

    def last_term(self) -> Term:
        if self.entries:
            return self.entries[-1].term
        return self.snapshot_term

    def position_of(self, index: Index) -> int:
        return index - self.snapshot_index - 1

    def term_at(self, index: Index) -> Term:
        if index == 0:
            return 0
        if index <= self.snapshot_index:
            return self.snapshot_term
        pos = self.position_of(index)
        if 0 <= pos < len(self.entries):
            return self.entries[pos].term
        return 0

    def entry_at(self, index: Index) -> LogEntry | None:
        pos = self.position_of(index)
        if 0 <= pos < len(self.entries):
            return self.entries[pos]
        return None


def prev_entry_matches(view: LogView, prev_log_index: Index, prev_log_term: Term) -> bool:
    if prev_log_index > view.last_index():
        return False
    if prev_log_index > view.snapshot_index and view.term_at(prev_log_index) != prev_log_term:
        return False
    return True


def backoff_hint(view: LogView, prev_log_index: Index) -> Index:
    return min(max(prev_log_index - 1, 0), view.last_index())
