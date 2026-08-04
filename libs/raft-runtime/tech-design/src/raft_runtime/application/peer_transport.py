from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass

from raft_runtime.domain.peer_tls import (
    HandshakeOutcome,
    PeerCertificate,
    PeerTlsConfig,
    TrustBundle,
    verify_peer,
)


@dataclass(frozen=True, slots=True)
class PeerTlsNotRequired:
    pass


@dataclass(frozen=True, slots=True)
class UnusableMaterial:
    reason: str


TransportError = PeerTlsNotRequired | UnusableMaterial


@dataclass(frozen=True, slots=True)
class TransportSnapshot:
    generation: int
    client_handle: str
    server_handle: str
    trust: TrustBundle


MaterialBuilder = Callable[[PeerTlsConfig], tuple[str, str] | UnusableMaterial]


class PeerTransport:
    __slots__ = ("_snapshot", "_build")

    def __init__(
        self, snapshot: TransportSnapshot, build: MaterialBuilder
    ) -> None:
        self._snapshot = snapshot
        self._build = build

    @staticmethod
    def from_config(
        config: PeerTlsConfig, build: MaterialBuilder
    ) -> PeerTransport | TransportError:
        if not config.required:
            return PeerTlsNotRequired()
        built = build(config)
        if isinstance(built, UnusableMaterial):
            return built
        return PeerTransport(
            TransportSnapshot(1, built[0], built[1], config.trust), build
        )

    def generation(self) -> int:
        return self._snapshot.generation

    def client_handle(self) -> str:
        return self._snapshot.client_handle

    def server_handle(self) -> str:
        return self._snapshot.server_handle

    def snapshot(self) -> TransportSnapshot:
        return self._snapshot

    def reload(self, config: PeerTlsConfig) -> int | TransportError:
        if not config.required:
            return PeerTlsNotRequired()
        built = self._build(config)
        if isinstance(built, UnusableMaterial):
            return built
        next_generation = self.generation() + 1
        self._snapshot = TransportSnapshot(
            next_generation, built[0], built[1], config.trust
        )
        return next_generation

    def accept(
        self, client_cert: PeerCertificate, now_ms: int
    ) -> HandshakeOutcome:
        return verify_peer(client_cert, self._snapshot.trust, None, now_ms)

    def connect(
        self,
        server_cert: PeerCertificate,
        expected_identity: str,
        now_ms: int,
    ) -> HandshakeOutcome:
        return verify_peer(
            server_cert, self._snapshot.trust, expected_identity, now_ms
        )
