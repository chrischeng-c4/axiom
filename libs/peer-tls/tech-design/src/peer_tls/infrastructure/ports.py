"""Infrastructure ports and protocol interfaces for peer TLS."""

from __future__ import annotations

from datetime import datetime
from typing import Protocol

from peer_tls.domain.material import LeafAttributes, PrivateKeyAttributes, TrustBundle


class EnvironmentSource(Protocol):
    def get(self, name: str) -> str | None:
        ...


class PemSource(Protocol):
    def read(self, location: str) -> LeafAttributes | PrivateKeyAttributes | TrustBundle:
        ...


class CryptoProviderInstaller(Protocol):
    def install_default(self) -> bool:
        ...


class Clock(Protocol):
    def now(self) -> datetime:
        ...
