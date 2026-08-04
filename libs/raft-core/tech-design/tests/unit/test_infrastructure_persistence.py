from __future__ import annotations

import dataclasses
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from raft_core.domain.entry import LogEntry
from raft_core.infrastructure.messages import VoteReq
from raft_core.infrastructure.persistence import PersistedState
from raft_core.infrastructure.transport import Outbox


class TestInfrastructurePersistence(unittest.TestCase):
    def test_persisted_state_defaults(self) -> None:
        state = PersistedState()
        self.assertEqual(state.term, 0)
        self.assertIsNone(state.voted_for)
        self.assertEqual(state.log, ())
        self.assertEqual(state.commit_index, 0)
        self.assertEqual(state.snapshot_index, 0)
        self.assertEqual(state.snapshot_term, 0)
        self.assertEqual(state.snapshot, b"")

    def test_persisted_state_replace(self) -> None:
        e = LogEntry(term=1, index=1, command=b"cmd")
        original = PersistedState(
            term=1,
            voted_for=2,
            log=(e,),
            commit_index=1,
            snapshot_index=0,
            snapshot_term=0,
            snapshot=b"data",
        )
        modified = dataclasses.replace(original, term=2)
        self.assertEqual(modified.term, 2)
        self.assertEqual(modified.voted_for, 2)
        self.assertEqual(modified.log, (e,))
        self.assertEqual(modified.commit_index, 1)
        self.assertEqual(modified.snapshot_index, 0)
        self.assertEqual(modified.snapshot_term, 0)
        self.assertEqual(modified.snapshot, b"data")

    def test_outbox_send_and_drain(self) -> None:
        outbox = Outbox()
        msg1 = VoteReq(term=1, candidate=0, last_log_index=0, last_log_term=0)
        msg2 = VoteReq(term=1, candidate=1, last_log_index=0, last_log_term=0)

        outbox.send(to=1, msg=msg1)
        outbox.send(to=2, msg=msg2)

        drained = outbox.drain()
        self.assertEqual(len(drained), 2)
        self.assertEqual(drained[0].to, 1)
        self.assertEqual(drained[0].msg, msg1)
        self.assertEqual(drained[1].to, 2)
        self.assertEqual(drained[1].msg, msg2)

        second_drain = outbox.drain()
        self.assertEqual(second_drain, ())


if __name__ == "__main__":
    unittest.main()
