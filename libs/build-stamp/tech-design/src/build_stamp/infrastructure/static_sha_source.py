from __future__ import annotations


class StaticShaSource:
    def __init__(self, success: bool, stdout: bytes) -> None:
        self._success = success
        self._stdout = stdout

    def read_short_sha(self) -> tuple[bool, bytes]:
        return self._success, self._stdout
