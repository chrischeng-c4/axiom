from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from raft_core.domain.membership import Membership, auto_membership, majority


class TestDomainMembership(unittest.TestCase):
    def test_auto_membership_cases(self) -> None:
        self.assertEqual(auto_membership(0), Membership(voters=(0,), learners=()))
        self.assertEqual(auto_membership(1), Membership(voters=(0,), learners=()))
        self.assertEqual(auto_membership(2), Membership(voters=(0,), learners=(1,)))
        self.assertEqual(auto_membership(3), Membership(voters=(0, 1, 2), learners=()))
        self.assertEqual(auto_membership(4), Membership(voters=(0, 1, 2), learners=(3,)))
        self.assertEqual(auto_membership(5), Membership(voters=(0, 1, 2, 3, 4), learners=()))
        self.assertEqual(auto_membership(6), Membership(voters=(0, 1, 2, 3, 4), learners=(5,)))

    def test_voter_count_is_odd(self) -> None:
        for n in range(13):
            mem = auto_membership(n)
            self.assertEqual(len(mem.voters) % 2, 1)

    def test_majority(self) -> None:
        self.assertEqual(majority(1), 1)
        self.assertEqual(majority(3), 2)
        self.assertEqual(majority(5), 3)
        self.assertEqual(majority(7), 4)


if __name__ == "__main__":
    unittest.main()
