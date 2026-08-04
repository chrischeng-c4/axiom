"""Unit tests for application layer use-case services."""

from __future__ import annotations

from datetime import datetime, timezone
import sys
import unittest
from pathlib import Path

SRC_ROOT = Path(__file__).resolve().parents[2] / "src"
sys.path.insert(0, str(SRC_ROOT))

from peer_tls.application.build_mtls_config import BuildMtlsConfigService  # noqa: E402
from peer_tls.application.rotate_material import RotateMaterialService  # noqa: E402
from peer_tls.application.validate_material import ValidateMaterialService  # noqa: E402
from peer_tls.domain.identity import DnsName, ExpectationKind, IdentityExpectation  # noqa: E402
from peer_tls.domain.material import (  # noqa: E402
    LeafAttributes,
    MaterialTriple,
    PrivateKeyAttributes,
    SubjectAltNames,
    TrustAnchor,
    TrustBundle,
)
from peer_tls.domain.rotation import Generation, RotationPhase, RotationState  # noqa: E402
from peer_tls.domain.verdict import ValidatedMaterial  # noqa: E402
from peer_tls.infrastructure.env_resolver import EnvPrefixError  # noqa: E402


class FakeClock:
    def __init__(self, now_dt: datetime) -> None:
        self._now_dt = now_dt

    def now(self) -> datetime:
        return self._now_dt


class FakeEnv:
    def __init__(self, env_dict: dict[str, str]) -> None:
        self._env_dict = env_dict

    def get(self, name: str) -> str | None:
        return self._env_dict.get(name)


class FakeCryptoInstaller:
    def __init__(self) -> None:
        self.call_count = 0

    def install_default(self) -> bool:
        self.call_count += 1
        return self.call_count == 1


class TestApplicationServices(unittest.TestCase):
    def setUp(self) -> None:
        self.now = datetime(2026, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        self.clock = FakeClock(self.now)
        self.leaf = LeafAttributes(
            subject_alt_names=SubjectAltNames(dns_names=(DnsName("svc.internal"),)),
            not_before=datetime(2026, 1, 1, 0, 0, 0, tzinfo=timezone.utc),
            not_after=datetime(2026, 1, 2, 0, 0, 0, tzinfo=timezone.utc),
            public_key_fingerprint="fp123",
            issuer_key_id="ca123",
        )
        self.key = PrivateKeyAttributes(public_key_fingerprint="fp123")
        self.trust = TrustBundle(anchors=(TrustAnchor(key_id="ca123", label="ca"),))
        self.triple = MaterialTriple(leaf=self.leaf, key=self.key, trust=self.trust)
        self.exp = IdentityExpectation(kind=ExpectationKind.SERVING, dns_names=(DnsName("svc.internal"),))

    def test_validate_material_service(self) -> None:
        svc = ValidateMaterialService(clock=self.clock)
        verdict = svc.execute(self.triple, self.exp)
        self.assertIsInstance(verdict, ValidatedMaterial)

    def test_build_mtls_config_service_unset_prefix(self) -> None:
        env = FakeEnv({})
        installer = FakeCryptoInstaller()
        svc = BuildMtlsConfigService(env=env, installer=installer, clock=self.clock)
        res = svc.execute("TEST", self.triple, self.exp, "leaf1")
        self.assertIsNone(res)
        self.assertEqual(installer.call_count, 0)

    def test_build_mtls_config_service_partial_prefix(self) -> None:
        env = FakeEnv({"TEST_CERT": "/path/cert.pem"})
        installer = FakeCryptoInstaller()
        svc = BuildMtlsConfigService(env=env, installer=installer, clock=self.clock)
        with self.assertRaises(EnvPrefixError):
            svc.execute("TEST", self.triple, self.exp, "leaf1")

    def test_build_mtls_config_service_success_and_installer_called_once(self) -> None:
        env = FakeEnv({"TEST_CERT": "/cert", "TEST_KEY": "/key", "TEST_CA": "/ca"})
        installer = FakeCryptoInstaller()
        svc = BuildMtlsConfigService(env=env, installer=installer, clock=self.clock)
        res = svc.execute("TEST", self.triple, self.exp, "leaf1")
        self.assertIsNotNone(res)
        assert res is not None
        server_plan, client_plan = res
        self.assertTrue(server_plan.peer_certificate_required)
        self.assertTrue(client_plan.presents_client_certificate)
        self.assertEqual(installer.call_count, 1)

        # Call execute a second time to verify installer.install_default is called each execute call
        svc.execute("TEST", self.triple, self.exp, "leaf1")
        self.assertEqual(installer.call_count, 2)

    def test_rotate_material_service(self) -> None:
        svc = RotateMaterialService()
        initial = RotationState(
            phase=RotationPhase.STEADY,
            outgoing=TrustAnchor("ca1", "old"),
            incoming=TrustAnchor("ca2", "new"),
            active=Generation(1, "leaf1"),
            activation_observed=False,
        )
        s1 = svc.execute(initial, Generation(1, "leaf1"))
        self.assertEqual(s1.active.number, 2)
        self.assertEqual(s1.phase, RotationPhase.INCOMING_TRUSTED)

        s2 = svc.observe_activation(s1)
        self.assertTrue(s2.activation_observed)


if __name__ == "__main__":
    unittest.main()
