from __future__ import annotations

import sys
import unittest

sys.path.insert(0, __file__.rsplit("/", 3)[0] + "/src")

from transport_h2c.infrastructure.client_pool import (
    PRIOR_KNOWLEDGE,
    ClientPool,
    ClientSettings,
    builder_settings,
    client_index,
    handout,
    next_cursor,
    pool_for_concurrency,
    pool_of,
)
from transport_h2c.infrastructure.connection import ConnectionState, release, reserve


class TestInfrastructureClientPool(unittest.TestCase):
    def test_pool_of_normal(self) -> None:
        p = pool_of(3)
        self.assertEqual(p.size, 3)

    def test_pool_of_zero(self) -> None:
        p = pool_of(0)
        self.assertEqual(p.size, 1)

    def test_pool_of_negative(self) -> None:
        p = pool_of(-3)
        self.assertEqual(p.size, 1)

    def test_pool_of_one(self) -> None:
        p = pool_of(1)
        self.assertEqual(p.size, 1)

    def test_pool_for_concurrency_normal(self) -> None:
        p = pool_for_concurrency(256, 64)
        self.assertEqual(p.size, 6)

    def test_pool_for_concurrency_small(self) -> None:
        p = pool_for_concurrency(1, 64)
        self.assertEqual(p.size, 1)

    def test_pool_for_concurrency_zero_zero(self) -> None:
        p = pool_for_concurrency(0, 0)
        self.assertEqual(p.size, 1)

    def test_handout_exact_size(self) -> None:
        p = pool_of(3)
        self.assertEqual(handout(p, 0, 3), (0, 1, 2))

    def test_handout_wraps_and_repeats(self) -> None:
        p = pool_of(3)
        self.assertEqual(handout(p, 0, 7), (0, 1, 2, 0, 1, 2, 0))

    def test_handout_single_element_pool(self) -> None:
        p = pool_of(1)
        self.assertEqual(handout(p, 0, 4), (0, 0, 0, 0))

    def test_handout_nonzero_starting_cursor(self) -> None:
        p = pool_of(3)
        self.assertEqual(handout(p, 5, 3), (2, 0, 1))

    def test_handout_nonpositive_count(self) -> None:
        p = pool_of(3)
        self.assertEqual(handout(p, 0, 0), ())
        self.assertEqual(handout(p, 0, -1), ())

    def test_handout_reaches_every_client(self) -> None:
        p = pool_of(4)
        res = handout(p, 0, 8)
        self.assertEqual(set(res), {0, 1, 2, 3})

    def test_next_cursor_monotone(self) -> None:
        big = 2**40
        self.assertEqual(next_cursor(big), big + 1)

    def test_client_index_modulo(self) -> None:
        p = pool_of(3)
        big = 2**40
        self.assertEqual(client_index(p, big), big % 3)

    def test_builder_settings_default(self) -> None:
        st = ClientSettings()
        self.assertEqual(builder_settings(st), {"http2_prior_knowledge": True})

    def test_builder_settings_timeout_only(self) -> None:
        st = ClientSettings(timeout_seconds=2.5)
        d = builder_settings(st)
        self.assertEqual(len(d), 2)
        self.assertIn("timeout_seconds", d)
        self.assertNotIn("user_agent", d)
        self.assertTrue(d["http2_prior_knowledge"])

    def test_builder_settings_user_agent_only(self) -> None:
        st = ClientSettings(user_agent="keep/1")
        d = builder_settings(st)
        self.assertEqual(len(d), 2)
        self.assertIn("user_agent", d)
        self.assertNotIn("timeout_seconds", d)
        self.assertTrue(d["http2_prior_knowledge"])

    def test_builder_settings_both(self) -> None:
        st = ClientSettings(timeout_seconds=2.5, user_agent="keep/1")
        d = builder_settings(st)
        self.assertEqual(len(d), 3)
        self.assertIn("timeout_seconds", d)
        self.assertIn("user_agent", d)
        self.assertTrue(d["http2_prior_knowledge"])

    def test_builder_settings_prior_knowledge_unconditional(self) -> None:
        self.assertTrue(PRIOR_KNOWLEDGE)
        for st in [
            ClientSettings(),
            ClientSettings(timeout_seconds=5.0),
            ClientSettings(user_agent="agent"),
            ClientSettings(timeout_seconds=5.0, user_agent="agent"),
        ]:
            self.assertIs(builder_settings(st)["http2_prior_knowledge"], True)

    def test_release_saturates_at_zero(self) -> None:
        conn = ConnectionState(id=1, in_flight=0)
        release(conn)
        self.assertEqual(conn.in_flight, 0)
        release(conn)
        self.assertEqual(conn.in_flight, 0)

    def test_release_after_reserve(self) -> None:
        conn = ConnectionState(id=1, in_flight=0)
        reserve(conn)
        self.assertEqual(conn.in_flight, 1)
        release(conn)
        self.assertEqual(conn.in_flight, 0)


if __name__ == "__main__":
    unittest.main()
