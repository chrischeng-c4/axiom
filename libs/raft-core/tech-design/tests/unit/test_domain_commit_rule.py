from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from raft_core.domain.commit_rule import highest_committed
from raft_core.domain.entry import LogEntry
from raft_core.domain.log_view import LogView


class TestDomainCommitRule(unittest.TestCase):
    def test_commit_advances_for_current_term_majority(self) -> None:
        e1 = LogEntry(term=2, index=1, command=b"cmd1")
        view = LogView(snapshot_index=0, snapshot_term=0, entries=(e1,))
        voters = (0, 1, 2)
        matches = {1: 1, 2: 0}
        result = highest_committed(
            view=view,
            voters=voters,
            matches=matches,
            leader_id=0,
            current_term=2,
            commit_index=0,
        )
        self.assertEqual(result, 1)

    def test_commit_does_not_advance_without_majority(self) -> None:
        e1 = LogEntry(term=2, index=1, command=b"cmd1")
        view = LogView(snapshot_index=0, snapshot_term=0, entries=(e1,))
        voters = (0, 1, 2)
        matches = {1: 0, 2: 0}
        result = highest_committed(
            view=view,
            voters=voters,
            matches=matches,
            leader_id=0,
            current_term=2,
            commit_index=0,
        )
        self.assertEqual(result, 0)

    def test_earlier_term_entry_committed_by_current_term_entry(self) -> None:
        e1 = LogEntry(term=1, index=1, command=b"cmd1")
        e2 = LogEntry(term=2, index=2, command=b"cmd2")

        view1 = LogView(snapshot_index=0, snapshot_term=0, entries=(e1,))
        voters = (0, 1, 2)
        matches1 = {1: 1, 2: 1}
        result1 = highest_committed(
            view=view1,
            voters=voters,
            matches=matches1,
            leader_id=0,
            current_term=2,
            commit_index=0,
        )
        self.assertEqual(result1, 0)

        view2 = LogView(snapshot_index=0, snapshot_term=0, entries=(e1, e2))
        matches2 = {1: 2, 2: 0}
        result2 = highest_committed(
            view=view2,
            voters=voters,
            matches=matches2,
            leader_id=0,
            current_term=2,
            commit_index=0,
        )
        self.assertEqual(result2, 2)

    def test_learner_does_not_count(self) -> None:
        e1 = LogEntry(term=2, index=1, command=b"cmd1")
        view = LogView(snapshot_index=0, snapshot_term=0, entries=(e1,))
        voters = (0, 1, 2)
        matches = {1: 0, 2: 0, 3: 1}
        result = highest_committed(
            view=view,
            voters=voters,
            matches=matches,
            leader_id=0,
            current_term=2,
            commit_index=0,
        )
        self.assertEqual(result, 0)

    def test_never_below_incoming_commit_index(self) -> None:
        e1 = LogEntry(term=1, index=1, command=b"cmd1")
        view = LogView(snapshot_index=0, snapshot_term=0, entries=(e1,))
        voters = (0, 1, 2)
        matches = {1: 0, 2: 0}
        result = highest_committed(
            view=view,
            voters=voters,
            matches=matches,
            leader_id=0,
            current_term=2,
            commit_index=5,
        )
        self.assertEqual(result, 5)


if __name__ == "__main__":
    unittest.main()
