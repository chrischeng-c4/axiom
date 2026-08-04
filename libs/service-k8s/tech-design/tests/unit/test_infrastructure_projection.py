from __future__ import annotations

import base64
import sys
import unittest
from datetime import datetime, timezone

sys.path.insert(0, __file__.rsplit("/", 3)[0] + "/src")

from service_k8s.application.rotation import IssuerId, ObservedLeaf
from service_k8s.application.trust_bundle import TrustBundle
from service_k8s.domain.digest import hex_sha256
from service_k8s.domain.purpose import Purpose
from service_k8s.domain.scope import InstanceScope
from service_k8s.infrastructure.projection import (
    CERT_KEY,
    IDENTITY_DIGEST_ANNOTATION,
    LEAF_ISSUER_ANNOTATION,
    MANAGED_BY,
    PRIVATE_KEY_KEY,
    TRUST_BUNDLE_ANNOTATION,
    TRUST_BUNDLE_KEY,
    IssuedMaterial,
    LeafFacts,
    LeafParseError,
    Owner,
    ProjectedState,
    base_secret,
    labels,
    material_secret,
    parse_leaf,
    pem_body_to_der,
    read_state,
    trust_bundle_secret,
)


def stub_validity(der: bytes) -> tuple[datetime, datetime]:
    return (
        datetime(2026, 1, 1, tzinfo=timezone.utc),
        datetime(2026, 4, 1, tzinfo=timezone.utc),
    )


def make_pem(der: bytes, line_length: int = 64) -> str:
    body = base64.b64encode(der).decode("ascii")
    if line_length > 0:
        lines = [
            body[i : i + line_length]
            for i in range(0, len(body), line_length)
        ]
        body = "\n".join(lines)
    return f"-----BEGIN CERTIFICATE-----\n{body}\n-----END CERTIFICATE-----"


class TestInfrastructureProjection(unittest.TestCase):
    def test_constant_strings(self) -> None:
        self.assertEqual(CERT_KEY, "tls.crt")
        self.assertEqual(PRIVATE_KEY_KEY, "tls.key")
        self.assertEqual(TRUST_BUNDLE_KEY, "ca.crt")
        self.assertEqual(
            TRUST_BUNDLE_ANNOTATION, "service-k8s.axiom.dev/trust-bundle"
        )
        self.assertEqual(
            LEAF_ISSUER_ANNOTATION, "service-k8s.axiom.dev/leaf-issuer"
        )
        self.assertEqual(
            IDENTITY_DIGEST_ANNOTATION, "service-k8s.axiom.dev/identity-digest"
        )

    def test_owner_reference(self) -> None:
        owner = Owner("v1", "Deployment", "my-dep", "uid-123")
        ref = owner.reference()
        expected = {
            "apiVersion": "v1",
            "kind": "Deployment",
            "name": "my-dep",
            "uid": "uid-123",
            "controller": True,
            "blockOwnerDeletion": True,
        }
        self.assertEqual(ref, expected)
        self.assertEqual(len(ref), 6)

    def test_labels(self) -> None:
        scope = InstanceScope("lumen-ns", "lumen-app", "lumen-td")
        lbls = labels(scope, Purpose.SERVING)
        expected = {
            "app.kubernetes.io/name": "lumen-app",
            "app.kubernetes.io/managed-by": MANAGED_BY,
            "app.kubernetes.io/component": "serving-tls",
        }
        self.assertEqual(lbls, expected)
        self.assertEqual(len(lbls), 3)

    def test_labels_component(self) -> None:
        scope = InstanceScope("ns", "app", "td")
        self.assertEqual(
            labels(scope, Purpose.PEER)["app.kubernetes.io/component"],
            "peer-tls",
        )
        self.assertEqual(
            labels(scope, Purpose.SERVING)["app.kubernetes.io/component"],
            "serving-tls",
        )

    def test_base_secret_type(self) -> None:
        scope = InstanceScope("ns", "app", "td")
        owner = Owner("v1", "Kind", "name", "uid")
        sec = base_secret(scope, Purpose.SERVING, owner)
        self.assertEqual(sec["type"], "Opaque")
        self.assertNotEqual(sec["type"], "kubernetes.io/tls")

    def test_base_secret_metadata(self) -> None:
        scope = InstanceScope("ns", "app", "td")
        owner = Owner("v1", "Kind", "name", "uid")
        sec = base_secret(scope, Purpose.SERVING, owner)
        meta = sec["metadata"]
        assert isinstance(meta, dict)
        self.assertEqual(meta["name"], "app-serving-tls")
        self.assertEqual(meta["namespace"], "ns")

    def test_base_secret_owner_references(self) -> None:
        scope = InstanceScope("ns", "app", "td")
        owner = Owner("v1", "Kind", "name", "uid")
        sec = base_secret(scope, Purpose.SERVING, owner)
        meta = sec["metadata"]
        assert isinstance(meta, dict)
        refs = meta["ownerReferences"]
        self.assertEqual(refs, [owner.reference()])

    def test_material_secret_string_data_keys(self) -> None:
        scope = InstanceScope("ns", "app", "td")
        owner = Owner("v1", "Kind", "name", "uid")
        mat = IssuedMaterial(
            issuer=IssuerId("iss"),
            certificate_pem="cert-pem",
            chain_pem="chain-pem",
            not_before=datetime(2026, 1, 1, tzinfo=timezone.utc),
            not_after=datetime(2026, 4, 1, tzinfo=timezone.utc),
            fingerprint="fp",
        )
        bundle = TrustBundle().with_anchor(
            IssuerId("iss"), "-----BEGIN CERTIFICATE-----\nA\n-----END CERTIFICATE-----"
        )
        sec = material_secret(
            scope, Purpose.SERVING, owner, mat, "key-pem", bundle, "digest-123"
        )
        string_data = sec["stringData"]
        assert isinstance(string_data, dict)
        self.assertEqual(set(string_data.keys()), {"tls.crt", "tls.key", "ca.crt"})

    def test_material_secret_omits_chain_pem(self) -> None:
        scope = InstanceScope("ns", "app", "td")
        owner = Owner("v1", "Kind", "name", "uid")
        mat = IssuedMaterial(
            issuer=IssuerId("iss"),
            certificate_pem="cert-pem",
            chain_pem="UNIQUE-CHAIN-PEM-VALUE",
            not_before=datetime(2026, 1, 1, tzinfo=timezone.utc),
            not_after=datetime(2026, 4, 1, tzinfo=timezone.utc),
            fingerprint="fp",
        )
        bundle = TrustBundle()
        sec = material_secret(
            scope, Purpose.SERVING, owner, mat, "key-pem", bundle, "digest-123"
        )
        string_data = sec["stringData"]
        assert isinstance(string_data, dict)
        for val in string_data.values():
            self.assertNotIn("UNIQUE-CHAIN-PEM-VALUE", str(val))

    def test_material_secret_annotations(self) -> None:
        scope = InstanceScope("ns", "app", "td")
        owner = Owner("v1", "Kind", "name", "uid")
        mat = IssuedMaterial(
            issuer=IssuerId("iss-1"),
            certificate_pem="cert",
            chain_pem="chain",
            not_before=datetime(2026, 1, 1, tzinfo=timezone.utc),
            not_after=datetime(2026, 4, 1, tzinfo=timezone.utc),
            fingerprint="fp",
        )
        bundle = TrustBundle().with_anchor(
            IssuerId("iss-1"), "-----BEGIN CERTIFICATE-----\nA\n-----END CERTIFICATE-----"
        )
        sec = material_secret(
            scope, Purpose.SERVING, owner, mat, "key", bundle, "digest-789"
        )
        meta = sec["metadata"]
        assert isinstance(meta, dict)
        ann = meta["annotations"]
        assert isinstance(ann, dict)
        self.assertEqual(ann[TRUST_BUNDLE_ANNOTATION], "iss-1")
        self.assertEqual(ann[LEAF_ISSUER_ANNOTATION], "iss-1")
        self.assertEqual(ann[IDENTITY_DIGEST_ANNOTATION], "digest-789")

    def test_trust_bundle_secret_string_data_keys(self) -> None:
        scope = InstanceScope("ns", "app", "td")
        owner = Owner("v1", "Kind", "name", "uid")
        bundle = TrustBundle().with_anchor(
            IssuerId("iss"), "-----BEGIN CERTIFICATE-----\nA\n-----END CERTIFICATE-----"
        )
        sec = trust_bundle_secret(scope, Purpose.SERVING, owner, bundle)
        string_data = sec["stringData"]
        assert isinstance(string_data, dict)
        self.assertEqual(set(string_data.keys()), {"ca.crt"})
        self.assertNotIn("tls.crt", string_data)
        self.assertNotIn("tls.key", string_data)

    def test_trust_bundle_secret_annotations(self) -> None:
        scope = InstanceScope("ns", "app", "td")
        owner = Owner("v1", "Kind", "name", "uid")
        bundle = TrustBundle().with_anchor(
            IssuerId("iss"), "-----BEGIN CERTIFICATE-----\nA\n-----END CERTIFICATE-----"
        )
        sec = trust_bundle_secret(scope, Purpose.SERVING, owner, bundle)
        meta = sec["metadata"]
        assert isinstance(meta, dict)
        ann = meta["annotations"]
        assert isinstance(ann, dict)
        self.assertEqual(set(ann.keys()), {TRUST_BUNDLE_ANNOTATION})

    def test_pem_body_to_der_valid(self) -> None:
        der = b"sample-der-bytes"
        pem = make_pem(der)
        decoded = pem_body_to_der(pem)
        self.assertEqual(decoded, der)

    def test_pem_body_to_der_invalid(self) -> None:
        pem = "-----BEGIN CERTIFICATE-----\nNOT-VALID-BASE64!@#$\n-----END CERTIFICATE-----"
        with self.assertRaises(LeafParseError):
            pem_body_to_der(pem)

    def test_parse_leaf_no_pem_block(self) -> None:
        with self.assertRaises(LeafParseError):
            parse_leaf("no pem block here", stub_validity)

    def test_parse_leaf_fingerprint_matches_hex_sha256(self) -> None:
        der = b"test-der-payload-123"
        pem = make_pem(der)
        facts = parse_leaf(pem, stub_validity)
        expected_fp = hex_sha256(der)
        self.assertEqual(facts.fingerprint, expected_fp)

    def test_parse_leaf_line_wrapping_invariant(self) -> None:
        der = b"same-der-different-wrapping"
        pem1 = make_pem(der, line_length=32)
        pem2 = make_pem(der, line_length=64)
        facts1 = parse_leaf(pem1, stub_validity)
        facts2 = parse_leaf(pem2, stub_validity)
        self.assertEqual(facts1.fingerprint, facts2.fingerprint)

    def test_parse_leaf_uses_first_block_only(self) -> None:
        der_leaf = b"leaf-der"
        der_chain = b"chain-der"
        pem_combined = f"{make_pem(der_leaf)}\n{make_pem(der_chain)}"
        facts = parse_leaf(pem_combined, stub_validity)
        self.assertEqual(facts.fingerprint, hex_sha256(der_leaf))

    def test_read_state_valid(self) -> None:
        der = b"valid-cert-der"
        pem_cert = make_pem(der)
        bundle_pem = "-----BEGIN CERTIFICATE-----\nA\n-----END CERTIFICATE-----"
        data = {
            CERT_KEY: pem_cert.encode("utf-8"),
            TRUST_BUNDLE_KEY: bundle_pem.encode("utf-8"),
        }
        annotations = {
            LEAF_ISSUER_ANNOTATION: "pool-1",
            IDENTITY_DIGEST_ANNOTATION: "digest-123",
            TRUST_BUNDLE_ANNOTATION: "pool-1",
        }
        state = read_state(data, annotations, stub_validity)
        self.assertIsNotNone(state.leaf)
        assert state.leaf is not None
        self.assertEqual(state.leaf.issuer, IssuerId("pool-1"))
        self.assertEqual(state.leaf.identity_digest, "digest-123")
        self.assertEqual(
            state.leaf.not_after, datetime(2026, 4, 1, tzinfo=timezone.utc)
        )
        self.assertEqual(state.bundle.issuers(), (IssuerId("pool-1"),))

    def test_read_state_missing_annotation_returns_none_leaf(self) -> None:
        der = b"valid-cert-der"
        pem_cert = make_pem(der)
        data = {CERT_KEY: pem_cert.encode("utf-8")}

        # Missing LEAF_ISSUER_ANNOTATION
        ann1 = {IDENTITY_DIGEST_ANNOTATION: "digest-123"}
        state1 = read_state(data, ann1, stub_validity)
        self.assertIsNone(state1.leaf)

        # Missing IDENTITY_DIGEST_ANNOTATION
        ann2 = {LEAF_ISSUER_ANNOTATION: "pool-1"}
        state2 = read_state(data, ann2, stub_validity)
        self.assertIsNone(state2.leaf)

    def test_read_state_unparseable_cert_preserves_bundle(self) -> None:
        bundle_pem = "-----BEGIN CERTIFICATE-----\nA\n-----END CERTIFICATE-----"
        data = {
            CERT_KEY: b"INVALID-CERT-DATA",
            TRUST_BUNDLE_KEY: bundle_pem.encode("utf-8"),
        }
        annotations = {
            LEAF_ISSUER_ANNOTATION: "pool-1",
            IDENTITY_DIGEST_ANNOTATION: "digest-123",
            TRUST_BUNDLE_ANNOTATION: "pool-1",
        }
        state = read_state(data, annotations, stub_validity)
        self.assertIsNone(state.leaf)
        self.assertFalse(state.bundle.is_empty())
        self.assertEqual(state.bundle.issuers(), (IssuerId("pool-1"),))

    def test_read_state_bundle_count_mismatch_preserves_leaf(self) -> None:
        der = b"valid-cert-der"
        pem_cert = make_pem(der)
        bundle_pem = "-----BEGIN CERTIFICATE-----\nA\n-----END CERTIFICATE-----"
        data = {
            CERT_KEY: pem_cert.encode("utf-8"),
            TRUST_BUNDLE_KEY: bundle_pem.encode("utf-8"),
        }
        # Annotation claims 2 issuers, but bundle_pem has 1 block => count mismatch
        annotations = {
            LEAF_ISSUER_ANNOTATION: "pool-1",
            IDENTITY_DIGEST_ANNOTATION: "digest-123",
            TRUST_BUNDLE_ANNOTATION: "pool-1,pool-2",
        }
        state = read_state(data, annotations, stub_validity)
        self.assertIsNotNone(state.leaf)
        self.assertTrue(state.bundle.is_empty())


if __name__ == "__main__":
    unittest.main()
