from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from raft_runtime.infrastructure.routes import (
    APPEND_ENTRIES_PATH,
    CONSENSUS_PATHS,
    INSTALL_SNAPSHOT_PATH,
    PEER_PATHS,
    PUBLISH_PATH,
    RAFTZ_PATH,
    REQUEST_VOTE_PATH,
    is_consensus_path,
    is_peer_path,
    requires_peer_identity,
)


class TestInfrastructureRoutes(unittest.TestCase):
    def test_route_path_literals(self) -> None:
        self.assertEqual(REQUEST_VOTE_PATH, "/raft/request-vote")
        self.assertEqual(APPEND_ENTRIES_PATH, "/raft/append-entries")
        self.assertEqual(INSTALL_SNAPSHOT_PATH, "/raft/install-snapshot")
        self.assertEqual(PUBLISH_PATH, "/raft/publish")
        self.assertEqual(RAFTZ_PATH, "/raftz")

    def test_peer_paths_length_and_exclusion_of_raftz(self) -> None:
        self.assertEqual(len(PEER_PATHS), 4)
        self.assertNotIn(RAFTZ_PATH, PEER_PATHS)

    def test_is_peer_path_exact_matches(self) -> None:
        self.assertTrue(is_peer_path("/raft/publish"))
        self.assertFalse(is_peer_path("/raft/publish/"))
        self.assertFalse(is_peer_path("/raft"))
        self.assertFalse(is_peer_path(""))

    def test_is_consensus_path_excludes_publish_path(self) -> None:
        self.assertFalse(is_consensus_path(PUBLISH_PATH))
        self.assertTrue(is_consensus_path(APPEND_ENTRIES_PATH))
        self.assertTrue(is_consensus_path(REQUEST_VOTE_PATH))
        self.assertTrue(is_consensus_path(INSTALL_SNAPSHOT_PATH))

    def test_requires_peer_identity_returns_false_for_raftz(self) -> None:
        self.assertFalse(requires_peer_identity(RAFTZ_PATH))

    def test_requires_peer_identity_matches_is_peer_path(self) -> None:
        paths = [
            REQUEST_VOTE_PATH,
            APPEND_ENTRIES_PATH,
            INSTALL_SNAPSHOT_PATH,
            PUBLISH_PATH,
            RAFTZ_PATH,
            "/unknown",
            "/raft/publish/",
            "",
        ]
        for p in paths:
            self.assertEqual(
                requires_peer_identity(p),
                is_peer_path(p),
                f"Mismatch for path: {p!r}",
            )


if __name__ == "__main__":
    unittest.main()
