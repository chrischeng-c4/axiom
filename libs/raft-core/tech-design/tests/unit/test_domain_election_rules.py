from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from raft_core.domain.election_rules import is_up_to_date, vote_granted


class TestDomainElectionRules(unittest.TestCase):
    def test_is_up_to_date(self) -> None:
        self.assertTrue(
            is_up_to_date(
                candidate_last_term=2,
                candidate_last_index=1,
                local_last_term=1,
                local_last_index=5,
            )
        )
        self.assertTrue(
            is_up_to_date(
                candidate_last_term=2,
                candidate_last_index=5,
                local_last_term=2,
                local_last_index=4,
            )
        )
        self.assertTrue(
            is_up_to_date(
                candidate_last_term=2,
                candidate_last_index=4,
                local_last_term=2,
                local_last_index=4,
            )
        )
        self.assertFalse(
            is_up_to_date(
                candidate_last_term=2,
                candidate_last_index=3,
                local_last_term=2,
                local_last_index=4,
            )
        )
        self.assertFalse(
            is_up_to_date(
                candidate_last_term=1,
                candidate_last_index=10,
                local_last_term=2,
                local_last_index=4,
            )
        )

    def test_vote_granted(self) -> None:
        self.assertTrue(
            vote_granted(
                request_term=1,
                current_term=1,
                voted_for=None,
                candidate=2,
                up_to_date=True,
            )
        )
        self.assertTrue(
            vote_granted(
                request_term=1,
                current_term=1,
                voted_for=2,
                candidate=2,
                up_to_date=True,
            )
        )
        self.assertFalse(
            vote_granted(
                request_term=1,
                current_term=1,
                voted_for=2,
                candidate=3,
                up_to_date=True,
            )
        )
        self.assertFalse(
            vote_granted(
                request_term=1,
                current_term=2,
                voted_for=None,
                candidate=2,
                up_to_date=True,
            )
        )
        self.assertFalse(
            vote_granted(
                request_term=1,
                current_term=1,
                voted_for=None,
                candidate=2,
                up_to_date=False,
            )
        )


if __name__ == "__main__":
    unittest.main()
