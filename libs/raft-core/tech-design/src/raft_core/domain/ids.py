from __future__ import annotations

from enum import Enum

NodeId = int  # type alias
Term = int  # type alias
Index = int  # type alias, 1-based; 0 means "before the first entry"


class Role(Enum):
    FOLLOWER = "follower"
    CANDIDATE = "candidate"
    LEADER = "leader"
