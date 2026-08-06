from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from raft_runtime.application.peer_transport import (
    PeerTlsNotRequired,
    PeerTransport,
    TransportSnapshot,
    UnusableMaterial,
)
from raft_runtime.domain.peer_tls import (
    HandshakeOutcome,
    PeerCertificate,
    PeerTlsConfig,
    TrustBundle,
)


def good_builder(config: PeerTlsConfig) -> tuple[str, str] | UnusableMaterial:
    issuer = config.trust.issuers[0] if config.trust.issuers else "none"
    return (f"client-handle-{issuer}", f"server-handle-{issuer}")


def bad_builder(config: PeerTlsConfig) -> tuple[str, str] | UnusableMaterial:
    return UnusableMaterial(reason="certificate chain failed validation")


class TestApplicationPeerTransport(unittest.TestCase):
    def setUp(self) -> None:
        self.cert = PeerCertificate(
            subject="raft-0.raft.svc",
            issuer="ca1.raft.svc",
            dns_names=("raft-0.raft.svc",),
            not_before_ms=1000,
            not_after_ms=2000,
        )
        self.trust1 = TrustBundle(issuers=("ca1.raft.svc",))
        self.trust2 = TrustBundle(issuers=("ca2.raft.svc",))
        self.config1 = PeerTlsConfig(
            required=True,
            trust=self.trust1,
            client_cert=self.cert,
            server_cert=self.cert,
        )
        self.config2 = PeerTlsConfig(
            required=True,
            trust=self.trust2,
            client_cert=self.cert,
            server_cert=self.cert,
        )

    def test_from_config_returns_not_required_when_disabled(self) -> None:
        config = PeerTlsConfig(
            required=False,
            trust=self.trust1,
            client_cert=self.cert,
            server_cert=self.cert,
        )
        result = PeerTransport.from_config(config, good_builder)
        self.assertIsInstance(result, PeerTlsNotRequired)

    def test_from_config_returns_unusable_material_on_build_failure(
        self,
    ) -> None:
        result = PeerTransport.from_config(self.config1, bad_builder)
        self.assertIsInstance(result, UnusableMaterial)
        self.assertEqual(
            result.reason, "certificate chain failed validation"
        )

    def test_from_config_success_initializes_generation_one(self) -> None:
        transport = PeerTransport.from_config(self.config1, good_builder)
        self.assertIsInstance(transport, PeerTransport)
        self.assertEqual(transport.generation(), 1)
        self.assertEqual(transport.client_handle(), "client-handle-ca1.raft.svc")
        self.assertEqual(transport.server_handle(), "server-handle-ca1.raft.svc")

    def test_reload_success_increments_generation(self) -> None:
        transport = PeerTransport.from_config(self.config1, good_builder)
        self.assertIsInstance(transport, PeerTransport)

        gen2 = transport.reload(self.config2)
        self.assertEqual(gen2, 2)
        self.assertEqual(transport.generation(), 2)
        self.assertEqual(transport.client_handle(), "client-handle-ca2.raft.svc")

        gen3 = transport.reload(self.config1)
        self.assertEqual(gen3, 3)
        self.assertEqual(transport.generation(), 3)

    def test_reload_unusable_material_preserves_existing_state(self) -> None:
        transport = PeerTransport.from_config(self.config1, good_builder)
        self.assertIsInstance(transport, PeerTransport)

        # Reconfigure the internal builder to return bad material
        transport._build = bad_builder
        result = transport.reload(self.config2)

        self.assertIsInstance(result, UnusableMaterial)
        self.assertEqual(transport.generation(), 1)
        self.assertEqual(transport.client_handle(), "client-handle-ca1.raft.svc")
        self.assertEqual(transport.server_handle(), "server-handle-ca1.raft.svc")
        self.assertEqual(transport.snapshot().trust, self.trust1)

    def test_reload_not_required_preserves_existing_state(self) -> None:
        transport = PeerTransport.from_config(self.config1, good_builder)
        self.assertIsInstance(transport, PeerTransport)

        disabled_config = PeerTlsConfig(
            required=False,
            trust=self.trust2,
            client_cert=self.cert,
            server_cert=self.cert,
        )
        result = transport.reload(disabled_config)

        self.assertIsInstance(result, PeerTlsNotRequired)
        self.assertEqual(transport.generation(), 1)
        self.assertEqual(transport.client_handle(), "client-handle-ca1.raft.svc")
        self.assertEqual(transport.snapshot().trust, self.trust1)

    def test_accept_ignores_expected_identity(self) -> None:
        transport = PeerTransport.from_config(self.config1, good_builder)
        self.assertIsInstance(transport, PeerTransport)

        # Accept evaluates client_cert trust and validity, but no expected identity
        outcome = transport.accept(self.cert, now_ms=1500)
        self.assertEqual(outcome, HandshakeOutcome.ACCEPTED)

    def test_connect_pins_expected_identity(self) -> None:
        transport = PeerTransport.from_config(self.config1, good_builder)
        self.assertIsInstance(transport, PeerTransport)

        outcome_ok = transport.connect(
            self.cert, expected_identity="raft-0.raft.svc", now_ms=1500
        )
        self.assertEqual(outcome_ok, HandshakeOutcome.ACCEPTED)

        outcome_bad = transport.connect(
            self.cert, expected_identity="other.raft.svc", now_ms=1500
        )
        self.assertEqual(outcome_bad, HandshakeOutcome.HOSTNAME_MISMATCH)

    def test_reload_swaps_trust_bundle_changing_accept_verdict(self) -> None:
        cert_ca2 = PeerCertificate(
            subject="raft-1.raft.svc",
            issuer="ca2.raft.svc",
            dns_names=("raft-1.raft.svc",),
            not_before_ms=1000,
            not_after_ms=2000,
        )

        transport = PeerTransport.from_config(self.config1, good_builder)
        self.assertIsInstance(transport, PeerTransport)

        # Before reload: ca2 is untrusted
        self.assertEqual(
            transport.accept(cert_ca2, now_ms=1500),
            HandshakeOutcome.UNTRUSTED_ISSUER,
        )

        # Reload with config2 which trusts ca2
        res = transport.reload(self.config2)
        self.assertEqual(res, 2)

        # After reload: ca2 is accepted
        self.assertEqual(
            transport.accept(cert_ca2, now_ms=1500),
            HandshakeOutcome.ACCEPTED,
        )

    def test_snapshot_getter_returns_transport_snapshot_instance(
        self,
    ) -> None:
        transport = PeerTransport.from_config(self.config1, good_builder)
        self.assertIsInstance(transport, PeerTransport)

        snap = transport.snapshot()
        self.assertIsInstance(snap, TransportSnapshot)
        self.assertEqual(
            snap,
            TransportSnapshot(
                generation=1,
                client_handle="client-handle-ca1.raft.svc",
                server_handle="server-handle-ca1.raft.svc",
                trust=self.trust1,
            ),
        )


if __name__ == "__main__":
    unittest.main()
