from __future__ import annotations

import dataclasses
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from raft_core.domain.entry import LogEntry
from raft_core.infrastructure.messages import (
    AppendReq,
    AppendResp,
    InstallSnapshotReq,
    InstallSnapshotResp,
    Outgoing,
    VoteReq,
    VoteResp,
)


class TestInfrastructureMessages(unittest.TestCase):
    def test_message_dataclasses_constructible_equal_and_frozen(self) -> None:
        e = LogEntry(term=1, index=1, command=b"a")

        vreq1 = VoteReq(term=1, candidate=0, last_log_index=2, last_log_term=1)
        vreq2 = VoteReq(term=1, candidate=0, last_log_index=2, last_log_term=1)
        self.assertEqual(vreq1, vreq2)

        vresp1 = VoteResp(term=1, granted=True)
        vresp2 = VoteResp(term=1, granted=True)
        self.assertEqual(vresp1, vresp2)

        areq1 = AppendReq(
            term=1, leader=0, prev_log_index=0, prev_log_term=0, entries=(e,), leader_commit=0
        )
        areq2 = AppendReq(
            term=1, leader=0, prev_log_index=0, prev_log_term=0, entries=(e,), leader_commit=0
        )
        self.assertEqual(areq1, areq2)

        aresp1 = AppendResp(term=1, success=True, match_index=1)
        aresp2 = AppendResp(term=1, success=True, match_index=1)
        self.assertEqual(aresp1, aresp2)

        sreq1 = InstallSnapshotReq(
            term=1, leader=0, snapshot_index=4, snapshot_term=2, data=b"snap"
        )
        sreq2 = InstallSnapshotReq(
            term=1, leader=0, snapshot_index=4, snapshot_term=2, data=b"snap"
        )
        self.assertEqual(sreq1, sreq2)

        sresp1 = InstallSnapshotResp(term=1, snapshot_index=4)
        sresp2 = InstallSnapshotResp(term=1, snapshot_index=4)
        self.assertEqual(sresp1, sresp2)

        with self.assertRaises(dataclasses.FrozenInstanceError):
            setattr(vreq1, "term", 2)

    def test_outgoing_constructible_with_all_msg_types(self) -> None:
        e = LogEntry(term=1, index=1, command=b"a")
        msgs = [
            VoteReq(term=1, candidate=0, last_log_index=0, last_log_term=0),
            VoteResp(term=1, granted=True),
            AppendReq(
                term=1, leader=0, prev_log_index=0, prev_log_term=0, entries=(e,), leader_commit=0
            ),
            AppendResp(term=1, success=True, match_index=1),
            InstallSnapshotReq(term=1, leader=0, snapshot_index=0, snapshot_term=0, data=b""),
            InstallSnapshotResp(term=1, snapshot_index=0),
        ]
        for msg in msgs:
            out = Outgoing(to=1, msg=msg)
            self.assertEqual(out.to, 1)
            self.assertEqual(out.msg, msg)


if __name__ == "__main__":
    unittest.main()
