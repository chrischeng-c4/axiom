from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from raft_runtime.infrastructure.env import (
    ALL_KEYS,
    NODE_ID_KEY,
    PEER_OVERRIDES_KEY,
    POD_NAME_KEY,
    REPLICAS_PER_SHARD_KEY,
    SHARD_COUNT_KEY,
    VOTER_COUNT_KEY,
    parse_peer_overrides,
    read_int,
    replica_mode,
)


class TestInfrastructureEnv(unittest.TestCase):
    def test_all_keys_tuple_order_and_count(self) -> None:
        self.assertEqual(len(ALL_KEYS), 6)
        expected = (
            POD_NAME_KEY,
            SHARD_COUNT_KEY,
            REPLICAS_PER_SHARD_KEY,
            VOTER_COUNT_KEY,
            NODE_ID_KEY,
            PEER_OVERRIDES_KEY,
        )
        self.assertEqual(ALL_KEYS, expected)

    def test_parse_peer_overrides_none_and_empty_returns_empty_tuple(
        self,
    ) -> None:
        self.assertEqual(parse_peer_overrides(None), ())
        self.assertEqual(parse_peer_overrides(""), ())

    def test_parse_peer_overrides_strips_items_and_drops_empties(
        self,
    ) -> None:
        self.assertEqual(
            parse_peer_overrides(" a , ,b ,, c "), ("a", "b", "c")
        )

    def test_parse_peer_overrides_single_item(self) -> None:
        self.assertEqual(parse_peer_overrides("a"), ("a",))

    def test_read_int_absent_key_returns_default(self) -> None:
        lookup = lambda k: None
        self.assertEqual(read_int(lookup, "K", 7), 7)

    def test_read_int_whitespace_value_returns_default(self) -> None:
        lookup = lambda k: "  "
        self.assertEqual(read_int(lookup, "K", 7), 7)

    def test_read_int_valid_integer_parses(self) -> None:
        lookup = lambda k: "3"
        self.assertEqual(read_int(lookup, "K", 7), 3)

    def test_read_int_invalid_integer_returns_none(self) -> None:
        lookup = lambda k: "x"
        self.assertIsNone(read_int(lookup, "K", 7))

    def test_replica_mode_evaluation_rules(self) -> None:
        self.assertFalse(replica_mode(lambda k: None))
        self.assertFalse(replica_mode(lambda k: "1"))
        self.assertFalse(replica_mode(lambda k: "0"))
        self.assertTrue(replica_mode(lambda k: "2"))
        self.assertFalse(replica_mode(lambda k: "many"))


if __name__ == "__main__":
    unittest.main()
