from __future__ import annotations

from dataclasses import FrozenInstanceError
import sys
import unittest

sys.path.insert(0, __file__.rsplit("/", 3)[0] + "/src")

from service_k8s.domain.digest import hex_sha256
from service_k8s.domain.profile import (
    CertificateIdentity,
    CertificateProfile,
    ForeignDnsName,
    ForeignSpiffeUri,
    JitterExceedsWindow,
    LifetimeOutOfBounds,
    NoNames,
    PeerNeedsSpiffeUri,
    PublicDnsName,
    RenewWindowTooNarrow,
    RenewWindowTooWide,
)
from service_k8s.domain.purpose import ExtendedUsage, Purpose
from service_k8s.domain.scope import InstanceScope


def make_scope(
    namespace: str = "lumen",
    instance: str = "lumen",
    trust_domain: str = "lumen-prod.svc.id.goog",
) -> InstanceScope:
    return InstanceScope(namespace, instance, trust_domain)


def make_serving_identity(
    dns_names: tuple[str, ...] = ("lumen.lumen.svc.cluster.local", "lumen.lumen.svc"),
    spiffe_uri: str | None = None,
) -> CertificateIdentity:
    return CertificateIdentity(dns_names=dns_names, spiffe_uri=spiffe_uri)


def make_peer_identity(
    dns_names: tuple[str, ...] = ("lumen.lumen.svc.cluster.local", "lumen.lumen.svc"),
    spiffe_uri: str
    | None = "spiffe://lumen-prod.svc.id.goog/ns/lumen/sa/lumen",
) -> CertificateIdentity:
    return CertificateIdentity(dns_names=dns_names, spiffe_uri=spiffe_uri)


def make_profile(
    scope: InstanceScope | None = None,
    purpose: Purpose = Purpose.SERVING,
    common_name: str = "lumen.lumen.svc.cluster.local",
    identity: CertificateIdentity | None = None,
    lifetime_secs: int = 3600,
    renew_before_secs: int = 1200,
    renew_jitter_secs: int = 300,
) -> CertificateProfile:
    if scope is None:
        scope = make_scope()
    if identity is None:
        if purpose is Purpose.PEER:
            identity = make_peer_identity()
        else:
            identity = make_serving_identity()
    return CertificateProfile(
        scope=scope,
        purpose=purpose,
        common_name=common_name,
        identity=identity,
        lifetime_secs=lifetime_secs,
        renew_before_secs=renew_before_secs,
        renew_jitter_secs=renew_jitter_secs,
    )


class TestDomainProfile(unittest.TestCase):
    def test_valid_serving_profile_construction(self) -> None:
        p = make_profile(purpose=Purpose.SERVING)
        self.assertEqual(p.scope.namespace, "lumen")
        self.assertEqual(p.purpose, Purpose.SERVING)
        self.assertIn("lumen.lumen.svc", p.identity.dns_names)

    def test_valid_peer_profile_construction(self) -> None:
        p = make_profile(purpose=Purpose.PEER)
        self.assertEqual(p.purpose, Purpose.PEER)
        self.assertIsNotNone(p.identity.spiffe_uri)

    def test_secret_name_derived(self) -> None:
        p_serving = make_profile(purpose=Purpose.SERVING)
        p_peer = make_profile(purpose=Purpose.PEER)
        self.assertEqual(p_serving.secret_name(), "lumen-serving-tls")
        self.assertEqual(p_peer.secret_name(), "lumen-peer-tls")

    def test_extended_key_usages_delegation(self) -> None:
        p_serving = make_profile(purpose=Purpose.SERVING)
        p_peer = make_profile(purpose=Purpose.PEER)
        self.assertEqual(
            p_serving.extended_key_usages(), (ExtendedUsage.SERVER_AUTH,)
        )
        self.assertEqual(
            p_peer.extended_key_usages(),
            (ExtendedUsage.SERVER_AUTH, ExtendedUsage.CLIENT_AUTH),
        )

    def test_dataclass_is_frozen(self) -> None:
        p = make_profile()
        with self.assertRaises(FrozenInstanceError):
            p.common_name = "other.lumen.svc"  # type: ignore[misc]

    def test_refusal_empty_dns_names(self) -> None:
        identity = CertificateIdentity(dns_names=())
        with self.assertRaises(NoNames):
            make_profile(identity=identity)

    def test_refusal_public_dns_name(self) -> None:
        identity = CertificateIdentity(dns_names=("lumen.example.com",))
        with self.assertRaises(PublicDnsName) as cm:
            make_profile(identity=identity)
        self.assertEqual(cm.exception.name, "lumen.example.com")
        self.assertIn("lumen.example.com", str(cm.exception))

    def test_refusal_foreign_dns_name(self) -> None:
        identity = CertificateIdentity(
            dns_names=("lumen.other.svc.cluster.local",)
        )
        with self.assertRaises(ForeignDnsName) as cm:
            make_profile(identity=identity)
        self.assertEqual(cm.exception.name, "lumen.other.svc.cluster.local")
        self.assertEqual(cm.exception.namespace, "lumen")
        self.assertIn("lumen.other.svc.cluster.local", str(cm.exception))

    def test_refusal_prefix_trap_namespace_prefix_is_not_namespace_match(
        self,
    ) -> None:
        scope = make_scope(namespace="prod")
        identity = CertificateIdentity(
            dns_names=("evil.lumen-prod.svc.cluster.local",)
        )
        with self.assertRaises(ForeignDnsName) as cm:
            make_profile(scope=scope, identity=identity)
        self.assertEqual(cm.exception.name, "evil.lumen-prod.svc.cluster.local")
        self.assertEqual(cm.exception.namespace, "prod")

    def test_refusal_public_and_foreign_dns_name_order(self) -> None:
        identity = CertificateIdentity(dns_names=("evil.example.com",))
        with self.assertRaises(PublicDnsName):
            make_profile(identity=identity)

    def test_refusal_peer_needs_spiffe_uri(self) -> None:
        identity = CertificateIdentity(
            dns_names=("lumen.lumen.svc.cluster.local",), spiffe_uri=None
        )
        with self.assertRaises(PeerNeedsSpiffeUri):
            make_profile(purpose=Purpose.PEER, identity=identity)

    def test_serving_spiffe_uri_none_accepted(self) -> None:
        identity = CertificateIdentity(
            dns_names=("lumen.lumen.svc.cluster.local",), spiffe_uri=None
        )
        p = make_profile(purpose=Purpose.SERVING, identity=identity)
        self.assertIsNone(p.identity.spiffe_uri)

    def test_refusal_foreign_spiffe_uri_namespace(self) -> None:
        identity = CertificateIdentity(
            dns_names=("lumen.lumen.svc.cluster.local",),
            spiffe_uri="spiffe://lumen-prod.svc.id.goog/ns/other/sa/x",
        )
        with self.assertRaises(ForeignSpiffeUri) as cm:
            make_profile(identity=identity)
        self.assertTrue(cm.exception.expected_prefix.endswith("/ns/lumen/"))

    def test_refusal_foreign_spiffe_uri_trust_domain(self) -> None:
        identity = CertificateIdentity(
            dns_names=("lumen.lumen.svc.cluster.local",),
            spiffe_uri="spiffe://other-domain.svc.id.goog/ns/lumen/sa/x",
        )
        with self.assertRaises(ForeignSpiffeUri):
            make_profile(identity=identity)

    def test_serving_with_valid_spiffe_uri_accepted(self) -> None:
        identity = CertificateIdentity(
            dns_names=("lumen.lumen.svc.cluster.local",),
            spiffe_uri="spiffe://lumen-prod.svc.id.goog/ns/lumen/sa/serving",
        )
        p = make_profile(purpose=Purpose.SERVING, identity=identity)
        self.assertEqual(
            p.identity.spiffe_uri,
            "spiffe://lumen-prod.svc.id.goog/ns/lumen/sa/serving",
        )

    def test_refusal_lifetime_out_of_bounds(self) -> None:
        with self.assertRaises(LifetimeOutOfBounds):
            make_profile(lifetime_secs=299)
        p = make_profile(lifetime_secs=601, renew_before_secs=600, renew_jitter_secs=0)
        self.assertEqual(p.lifetime_secs, 601)

    # MIN_LIFETIME_SECS (300) is a floor on step 4 alone; combined with the MIN_RENEW_BEFORE_SECS (600) floor and step 6 (renew_before < lifetime), the smallest constructible lifetime is 601.
    def test_the_minimum_lifetime_cannot_hold_the_minimum_renewal_window(
        self,
    ) -> None:
        with self.assertRaises(RenewWindowTooWide):
            make_profile(
                lifetime_secs=300, renew_before_secs=600, renew_jitter_secs=0
            )

    def test_refusal_lifetime_max_out_of_bounds(self) -> None:
        with self.assertRaises(LifetimeOutOfBounds):
            make_profile(lifetime_secs=604801)
        p = make_profile(lifetime_secs=604800, renew_before_secs=3600)
        self.assertEqual(p.lifetime_secs, 604800)

    def test_refusal_renew_window_too_narrow(self) -> None:
        with self.assertRaises(RenewWindowTooNarrow):
            make_profile(renew_before_secs=599)
        p = make_profile(renew_before_secs=600, renew_jitter_secs=600)
        self.assertEqual(p.renew_before_secs, 600)

    def test_refusal_renew_window_too_wide(self) -> None:
        with self.assertRaises(RenewWindowTooWide):
            make_profile(lifetime_secs=3600, renew_before_secs=3600)

    def test_refusal_jitter_exceeds_window(self) -> None:
        with self.assertRaises(JitterExceedsWindow):
            make_profile(renew_before_secs=600, renew_jitter_secs=601)
        p = make_profile(renew_before_secs=600, renew_jitter_secs=600)
        self.assertEqual(p.renew_jitter_secs, 600)

    def test_digest_dns_names_order_invariant(self) -> None:
        ident1 = CertificateIdentity(
            dns_names=("lumen.lumen.svc", "lumen.lumen.svc.cluster.local")
        )
        ident2 = CertificateIdentity(
            dns_names=("lumen.lumen.svc.cluster.local", "lumen.lumen.svc")
        )
        p1 = make_profile(identity=ident1)
        p2 = make_profile(identity=ident2)
        self.assertEqual(p1.identity_digest(), p2.identity_digest())

    def test_digest_dns_names_content_distinct(self) -> None:
        ident1 = CertificateIdentity(
            dns_names=("lumen.lumen.svc.cluster.local",)
        )
        ident2 = CertificateIdentity(
            dns_names=("lumen.lumen.svc.cluster.local", "lumen.lumen.svc")
        )
        p1 = make_profile(identity=ident1)
        p2 = make_profile(identity=ident2)
        self.assertNotEqual(p1.identity_digest(), p2.identity_digest())

    def test_digest_cadence_fields_invariant(self) -> None:
        base = make_profile(
            lifetime_secs=3600, renew_before_secs=1200, renew_jitter_secs=300
        )
        d_base = base.identity_digest()

        p_lifetime = make_profile(
            lifetime_secs=7200, renew_before_secs=1200, renew_jitter_secs=300
        )
        self.assertEqual(p_lifetime.identity_digest(), d_base)

        p_renew = make_profile(
            lifetime_secs=3600, renew_before_secs=1800, renew_jitter_secs=300
        )
        self.assertEqual(p_renew.identity_digest(), d_base)

        p_jitter = make_profile(
            lifetime_secs=3600, renew_before_secs=1200, renew_jitter_secs=600
        )
        self.assertEqual(p_jitter.identity_digest(), d_base)

    def test_digest_purpose_and_common_name_change(self) -> None:
        p_serving = make_profile(purpose=Purpose.SERVING)
        p_peer = make_profile(purpose=Purpose.PEER)
        self.assertNotEqual(p_serving.identity_digest(), p_peer.identity_digest())

        p_cn2 = make_profile(common_name="alt.lumen.svc.cluster.local")
        self.assertNotEqual(p_serving.identity_digest(), p_cn2.identity_digest())

    def test_digest_format(self) -> None:
        p = make_profile()
        d = p.identity_digest()
        self.assertEqual(len(d), 64)
        valid_chars = set("0123456789abcdef")
        self.assertTrue(all(c in valid_chars for c in d))

    def test_digest_exact_preimage_hash(self) -> None:
        scope = InstanceScope("lumen", "lumen", "lumen-prod.svc.id.goog")
        identity = CertificateIdentity(
            dns_names=("lumen.lumen.svc.cluster.local", "lumen.lumen.svc"),
            spiffe_uri=None,
        )
        p = CertificateProfile(
            scope=scope,
            purpose=Purpose.SERVING,
            common_name="lumen.lumen.svc.cluster.local",
            identity=identity,
            lifetime_secs=3600,
            renew_before_secs=1200,
            renew_jitter_secs=300,
        )
        preimage = (
            "purpose=serving|"
            "cn=lumen.lumen.svc.cluster.local|"
            "dns=lumen.lumen.svc,lumen.lumen.svc.cluster.local|"
            "uri=|"
            "eku=serverAuth"
        )
        expected = hex_sha256(preimage.encode("utf-8"))
        self.assertEqual(p.identity_digest(), expected)


if __name__ == "__main__":
    unittest.main()
