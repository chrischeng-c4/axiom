from __future__ import annotations

REQUEST_VOTE_PATH: str = "/raft/request-vote"
APPEND_ENTRIES_PATH: str = "/raft/append-entries"
INSTALL_SNAPSHOT_PATH: str = "/raft/install-snapshot"
PUBLISH_PATH: str = "/raft/publish"
RAFTZ_PATH: str = "/raftz"

CONSENSUS_PATHS: tuple[str, ...] = (
    REQUEST_VOTE_PATH,
    APPEND_ENTRIES_PATH,
    INSTALL_SNAPSHOT_PATH,
)
PEER_PATHS: tuple[str, ...] = CONSENSUS_PATHS + (PUBLISH_PATH,)


def is_peer_path(path: str) -> bool:
    return path in PEER_PATHS


def is_consensus_path(path: str) -> bool:
    return path in CONSENSUS_PATHS


def requires_peer_identity(path: str) -> bool:
    return is_peer_path(path)
