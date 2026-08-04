"""Unit tests for domain validation decision logic and refusal ordering."""

from __future__ import annotations

from datetime import datetime, timezone
import sys
import unittest
from pathlib import Path

SRC_ROOT = Path(__file__).resolve().parents[2] / "src"
sys.path.insert(0, str(SRC_ROOT))

from peer_tls.domain.identity import DnsName, ExpectationKind, IdentityExpectation, SpiffeId, TrustDomain  # noqa: E402
from peer_tls.domain.material import (  # noqa: E402
    LeafAttributes,
    MaterialTriple,
    PrivateKeyAttributes,
    SubjectAltNames,
    TrustAnchor,
    TrustBundle,
)
from peer_tls.domain.validation import decide_material  # noqa: E402
from peer_tls.domain.verdict import RejectionReason, ValidatedMaterial, ValidityWindow  # noqa: E402


class TestDomainValidation(unittest.TestCase):
    def setUp(self) -> None:
        self.now = datetime(2026, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        self.not_before = datetime(2026, 1, 1, 0, 0, 0, tzinfo=timezone.utc)
        self.not_after = datetime(2026, 1, 2, 0, 0, 0, tzinfo=timezone.utc)
        self.valid_leaf = LeafAttributes(
            subject_alt_names=SubjectAltNames(
                dns_names=(DnsName("service.internal"),),
                uris=("spiffe://example.com/ns/default/sa/app",),
            ),
            not_before=self.not_before,
            not_after=self.not_after,
            public_key_fingerprint="fp123",
            issuer_key_id="ca123",
            common_name="service.internal",
        )
        self.valid_key = PrivateKeyAttributes(public_key_fingerprint="fp123")
        self.valid_trust = TrustBundle(anchors=(TrustAnchor(key_id="ca123", label="root-ca"),))
        self.valid_triple = MaterialTriple(
            leaf=self.valid_leaf,
            key=self.valid_key,
            trust=self.valid_trust,
        )
        self.serving_exp = IdentityExpectation(
            kind=ExpectationKind.SERVING,
            dns_names=(DnsName("service.internal"),),
        )
        self.peer_exp = IdentityExpectation(
            kind=ExpectationKind.PEER,
            spiffe_id=SpiffeId(trust_domain=TrustDomain("example.com"), path="ns/default/sa/app"),
        )

    def test_accept_path_serving(self) -> None:
        verdict = decide_material(self.valid_triple, self.serving_exp, self.now)
        self.assertIsInstance(verdict, ValidatedMaterial)

    def test_accept_path_peer(self) -> None:
        verdict = decide_material(self.valid_triple, self.peer_exp, self.now)
        self.assertIsInstance(verdict, ValidatedMaterial)

    def test_validity_window_arithmetic(self) -> None:
        window = ValidityWindow(not_before=self.not_before, not_after=self.not_after)
        self.assertTrue(window.contains(self.now))
        self.assertFalse(window.contains(datetime(2025, 1, 1, tzinfo=timezone.utc)))
        self.assertEqual(window.seconds_to_expiry(self.now), 43200)

    def test_refusal_1_malformed_expectation(self) -> None:
        malformed = IdentityExpectation(kind=ExpectationKind.SERVING, dns_names=())
        verdict = decide_material(self.valid_triple, malformed, self.now)
        self.assertEqual(verdict.reason, RejectionReason.MALFORMED_EXPECTATION)

    def test_refusal_2_key_does_not_match_leaf(self) -> None:
        bad_key_triple = MaterialTriple(
            leaf=self.valid_leaf,
            key=PrivateKeyAttributes(public_key_fingerprint="wrong_fp"),
            trust=self.valid_trust,
        )
        verdict = decide_material(bad_key_triple, self.serving_exp, self.now)
        self.assertEqual(verdict.reason, RejectionReason.KEY_DOES_NOT_MATCH_LEAF)

    def test_refusal_3_issuer_not_in_trust_bundle(self) -> None:
        bad_trust_triple = MaterialTriple(
            leaf=self.valid_leaf,
            key=self.valid_key,
            trust=TrustBundle(anchors=(TrustAnchor(key_id="other_ca", label="other"),)),
        )
        verdict = decide_material(bad_trust_triple, self.serving_exp, self.now)
        self.assertEqual(verdict.reason, RejectionReason.ISSUER_NOT_IN_TRUST_BUNDLE)

    def test_refusal_4_identity_in_wrong_extension(self) -> None:
        leaf_cn_only = LeafAttributes(
            subject_alt_names=SubjectAltNames(dns_names=(), uris=()),
            not_before=self.not_before,
            not_after=self.not_after,
            public_key_fingerprint="fp123",
            issuer_key_id="ca123",
            common_name="service.internal",
        )
        triple = MaterialTriple(leaf=leaf_cn_only, key=self.valid_key, trust=self.valid_trust)
        verdict = decide_material(triple, self.serving_exp, self.now)
        self.assertEqual(verdict.reason, RejectionReason.IDENTITY_IN_WRONG_EXTENSION)

    def test_refusal_5_trust_domain_mismatch(self) -> None:
        leaf_wrong_td = LeafAttributes(
            subject_alt_names=SubjectAltNames(
                dns_names=(),
                uris=("spiffe://other.com/ns/default/sa/app",),
            ),
            not_before=self.not_before,
            not_after=self.not_after,
            public_key_fingerprint="fp123",
            issuer_key_id="ca123",
        )
        triple = MaterialTriple(leaf=leaf_wrong_td, key=self.valid_key, trust=self.valid_trust)
        verdict = decide_material(triple, self.peer_exp, self.now)
        self.assertEqual(verdict.reason, RejectionReason.TRUST_DOMAIN_MISMATCH)

    def test_refusal_6_identity_mismatch(self) -> None:
        leaf_different = LeafAttributes(
            subject_alt_names=SubjectAltNames(
                dns_names=(DnsName("other.internal"),),
                uris=("spiffe://example.com/ns/default/sa/other",),
            ),
            not_before=self.not_before,
            not_after=self.not_after,
            public_key_fingerprint="fp123",
            issuer_key_id="ca123",
        )
        triple = MaterialTriple(leaf=leaf_different, key=self.valid_key, trust=self.valid_trust)
        verdict = decide_material(triple, self.serving_exp, self.now)
        self.assertEqual(verdict.reason, RejectionReason.IDENTITY_MISMATCH)

    def test_refusal_7_not_yet_valid(self) -> None:
        early_instant = datetime(2025, 12, 31, 0, 0, 0, tzinfo=timezone.utc)
        verdict = decide_material(self.valid_triple, self.serving_exp, early_instant)
        self.assertEqual(verdict.reason, RejectionReason.NOT_YET_VALID)

    def test_refusal_8_expired(self) -> None:
        late_instant = datetime(2026, 1, 3, 0, 0, 0, tzinfo=timezone.utc)
        verdict = decide_material(self.valid_triple, self.serving_exp, late_instant)
        self.assertEqual(verdict.reason, RejectionReason.EXPIRED)

    def test_precedence_key_mismatch_before_issuer(self) -> None:
        bad_both = MaterialTriple(
            leaf=self.valid_leaf,
            key=PrivateKeyAttributes(public_key_fingerprint="wrong_fp"),
            trust=TrustBundle(anchors=()),
        )
        verdict = decide_material(bad_both, self.serving_exp, self.now)
        self.assertEqual(verdict.reason, RejectionReason.KEY_DOES_NOT_MATCH_LEAF)

    def test_precedence_wrong_extension_before_trust_domain_mismatch(self) -> None:
        leaf_both_wrong = LeafAttributes(
            subject_alt_names=SubjectAltNames(
                dns_names=(),
                uris=("spiffe://other.com/ns/default/sa/app",),
            ),
            not_before=self.not_before,
            not_after=self.not_after,
            public_key_fingerprint="fp123",
            issuer_key_id="ca123",
            common_name="spiffe://example.com/ns/default/sa/app",
        )
        triple = MaterialTriple(leaf=leaf_both_wrong, key=self.valid_key, trust=self.valid_trust)
        verdict = decide_material(triple, self.peer_exp, self.now)
        self.assertEqual(verdict.reason, RejectionReason.IDENTITY_IN_WRONG_EXTENSION)

    def test_precedence_identity_before_validity_window(self) -> None:
        bad_identity_expired = MaterialTriple(
            leaf=LeafAttributes(
                subject_alt_names=SubjectAltNames(dns_names=(DnsName("other.internal"),)),
                not_before=self.not_before,
                not_after=self.not_after,
                public_key_fingerprint="fp123",
                issuer_key_id="ca123",
            ),
            key=self.valid_key,
            trust=self.valid_trust,
        )
        late_instant = datetime(2026, 1, 3, 0, 0, 0, tzinfo=timezone.utc)
        verdict = decide_material(bad_identity_expired, self.serving_exp, late_instant)
        self.assertEqual(verdict.reason, RejectionReason.IDENTITY_MISMATCH)


if __name__ == "__main__":
    unittest.main()
