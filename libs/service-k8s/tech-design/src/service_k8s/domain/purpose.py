from __future__ import annotations

from enum import Enum


class ExtendedUsage(Enum):
    SERVER_AUTH = "serverAuth"
    CLIENT_AUTH = "clientAuth"

    @property
    def token(self) -> str:
        return self.value


class Purpose(Enum):
    SERVING = "serving"
    PEER = "peer"

    @property
    def token(self) -> str:
        return self.value

    def extended_key_usages(self) -> tuple[ExtendedUsage, ...]:
        if self is Purpose.SERVING:
            return (ExtendedUsage.SERVER_AUTH,)
        return (ExtendedUsage.SERVER_AUTH, ExtendedUsage.CLIENT_AUTH)
