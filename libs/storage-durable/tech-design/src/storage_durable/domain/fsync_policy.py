from __future__ import annotations

from enum import Enum

class FsyncPolicy(str, Enum):
    ALWAYS = "always"
    EVERY_SEC = "every-sec"
    INTERVAL = "interval"
    OS = "os"

    def should_sync_immediately(self) -> bool:
        return self is FsyncPolicy.ALWAYS

    def forces_stable_storage(self) -> bool:
        return self is not FsyncPolicy.OS
