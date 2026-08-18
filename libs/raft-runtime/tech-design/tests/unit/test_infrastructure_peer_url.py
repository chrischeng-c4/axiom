from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from raft_runtime.domain.errors import UnsupportedScheme
from raft_runtime.infrastructure.peer_url import (
    ALLOWED_SCHEMES,
    peer_host,
    peer_url,
    scheme_problem,
)


class TestInfrastructurePeerUrl(unittest.TestCase):
    def test_allowed_schemes_tuple(self) -> None:
        self.assertEqual(ALLOWED_SCHEMES, ("http", "https"))

    def test_scheme_problem_valid_schemes_return_none(self) -> None:
        self.assertIsNone(scheme_problem("http"))
        self.assertIsNone(scheme_problem("https"))

    def test_scheme_problem_case_sensitive_and_unsupported_schemes(
        self,
    ) -> None:
        expected = UnsupportedScheme("HTTP", ("http", "https"))
        self.assertEqual(scheme_problem("HTTP"), expected)
        self.assertEqual(
            scheme_problem("h2c"), UnsupportedScheme("h2c", ("http", "https"))
        )
        self.assertEqual(
            scheme_problem(""), UnsupportedScheme("", ("http", "https"))
        )

    def test_peer_host_formatting(self) -> None:
        self.assertEqual(
            peer_host("lumen-raft", 2, "lumen-raft-peers"),
            "lumen-raft-2.lumen-raft-peers",
        )

    def test_peer_url_valid_construction(self) -> None:
        self.assertEqual(
            peer_url("https", "lumen-raft", 0, "peers", 8443),
            "https://lumen-raft-0.peers:8443",
        )

    def test_peer_url_unsupported_scheme_returns_error(self) -> None:
        self.assertEqual(
            peer_url("h2c", "p", 0, "s", 1),
            UnsupportedScheme("h2c", ("http", "https")),
        )


if __name__ == "__main__":
    unittest.main()
