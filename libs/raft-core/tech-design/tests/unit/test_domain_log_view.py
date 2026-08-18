from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from raft_core.domain.entry import LogEntry
from raft_core.domain.log_view import LogView, backoff_hint, prev_entry_matches


class TestDomainLogView(unittest.TestCase):
    def test_last_index_and_last_term(self) -> None:
        empty_view = LogView(snapshot_index=0, snapshot_term=0, entries=())
        self.assertEqual(empty_view.last_index(), 0)
        self.assertEqual(empty_view.last_term(), 0)

        e1 = LogEntry(term=1, index=1, command=b"cmd1")
        e2 = LogEntry(term=2, index=2, command=b"cmd2")
        view_with_entries = LogView(snapshot_index=0, snapshot_term=0, entries=(e1, e2))
        self.assertEqual(view_with_entries.last_index(), 2)
        self.assertEqual(view_with_entries.last_term(), 2)

        compacted_empty = LogView(snapshot_index=4, snapshot_term=2, entries=())
        self.assertEqual(compacted_empty.last_index(), 4)
        self.assertEqual(compacted_empty.last_term(), 2)

    def test_term_at(self) -> None:
        e5 = LogEntry(term=3, index=5, command=b"a")
        e6 = LogEntry(term=3, index=6, command=b"b")
        e7 = LogEntry(term=4, index=7, command=b"c")
        view = LogView(snapshot_index=4, snapshot_term=2, entries=(e5, e6, e7))

        self.assertEqual(view.term_at(0), 0)
        self.assertEqual(view.term_at(1), 2)
        self.assertEqual(view.term_at(4), 2)
        self.assertEqual(view.term_at(5), 3)
        self.assertEqual(view.term_at(6), 3)
        self.assertEqual(view.term_at(7), 4)
        self.assertEqual(view.term_at(8), 0)
        self.assertEqual(view.term_at(100), 0)

    def test_prev_entry_matches(self) -> None:
        e5 = LogEntry(term=3, index=5, command=b"a")
        view = LogView(snapshot_index=4, snapshot_term=2, entries=(e5,))

        self.assertFalse(prev_entry_matches(view, prev_log_index=6, prev_log_term=3))
        self.assertFalse(prev_entry_matches(view, prev_log_index=5, prev_log_term=1))
        self.assertTrue(prev_entry_matches(view, prev_log_index=5, prev_log_term=3))
        self.assertTrue(prev_entry_matches(view, prev_log_index=3, prev_log_term=999))
        self.assertTrue(prev_entry_matches(view, prev_log_index=4, prev_log_term=999))

    def test_backoff_hint(self) -> None:
        e1 = LogEntry(term=1, index=1, command=b"a")
        view = LogView(snapshot_index=0, snapshot_term=0, entries=(e1,))

        self.assertEqual(backoff_hint(view, prev_log_index=0), 0)
        self.assertEqual(backoff_hint(view, prev_log_index=10), 1)


if __name__ == "__main__":
    unittest.main()
