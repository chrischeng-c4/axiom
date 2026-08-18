from __future__ import annotations

import sys
import unittest

sys.path.insert(0, __file__.rsplit("/", 3)[0] + "/src")

from service_k8s.infrastructure.store import (
    FIELD_MANAGER,
    LIFECYCLE_ANNOTATION_KEYS,
    LIFECYCLE_DATA_KEYS,
    LIFECYCLE_LABEL_KEYS,
    REQUIRED_RBAC_VERBS,
    PatchDecision,
    StoreError,
    StoreErrorKind,
    classify_status,
    classify_transport,
    prepare_patch,
)


class TestInfrastructureStore(unittest.TestCase):
    def test_classify_status_403_forbidden(self) -> None:
        err = classify_status(403, "denied")
        self.assertEqual(err.kind, StoreErrorKind.FORBIDDEN)
        self.assertFalse(err.retryable())

    def test_classify_status_409_conflict(self) -> None:
        err = classify_status(409, "conflict")
        self.assertEqual(err.kind, StoreErrorKind.CONFLICT)
        self.assertTrue(err.retryable())

    def test_classify_status_500_503_unavailable(self) -> None:
        err500 = classify_status(500, "internal error")
        err503 = classify_status(503, "service unavailable")
        self.assertEqual(err500.kind, StoreErrorKind.UNAVAILABLE)
        self.assertTrue(err500.retryable())
        self.assertEqual(err503.kind, StoreErrorKind.UNAVAILABLE)
        self.assertTrue(err503.retryable())

    def test_classify_status_404_other_not_retryable(self) -> None:
        err = classify_status(404, "not found")
        self.assertEqual(err.kind, StoreErrorKind.OTHER)
        self.assertEqual(err.code, 404)
        self.assertFalse(err.retryable())

    def test_classify_status_429_other_retryable(self) -> None:
        err = classify_status(429, "too many requests")
        self.assertEqual(err.kind, StoreErrorKind.OTHER)
        self.assertEqual(err.code, 429)
        self.assertTrue(err.retryable())

    def test_classify_transport_unavailable(self) -> None:
        err = classify_transport("connection dropped")
        self.assertEqual(err.kind, StoreErrorKind.UNAVAILABLE)
        self.assertTrue(err.retryable())

    def test_store_error_redacts_pem_message(self) -> None:
        raw_msg = "denied for -----BEGIN PRIVATE KEY-----\nAAAA\n-----END PRIVATE KEY-----"
        err = StoreError.forbidden(raw_msg)
        self.assertIn("[redacted pem]", err.message)
        self.assertNotIn("AAAA", err.message)

    def test_store_error_redacts_bearer_message(self) -> None:
        raw_msg = "failed: Bearer secret-token-xyz"
        err = StoreError.unavailable(raw_msg)
        self.assertIn("Bearer [redacted]", err.message)
        self.assertNotIn("secret-token-xyz", err.message)

    def test_store_error_malformed_not_retryable(self) -> None:
        err = StoreError.malformed("bad schema")
        self.assertEqual(err.kind, StoreErrorKind.MALFORMED)
        self.assertFalse(err.retryable())

    def test_required_rbac_verbs(self) -> None:
        self.assertEqual(REQUIRED_RBAC_VERBS, ("get", "patch"))
        self.assertNotIn("delete", REQUIRED_RBAC_VERBS)
        self.assertNotIn("create", REQUIRED_RBAC_VERBS)

    def test_prepare_patch_missing_name_raises_malformed(self) -> None:
        desired = {"metadata": {"namespace": "ns"}}
        with self.assertRaises(StoreError) as ctx:
            prepare_patch(desired, None)
        self.assertEqual(ctx.exception.kind, StoreErrorKind.MALFORMED)

    def test_prepare_patch_missing_namespace_raises_malformed(self) -> None:
        desired = {"metadata": {"name": "name"}}
        with self.assertRaises(StoreError) as ctx:
            prepare_patch(desired, None)
        self.assertEqual(ctx.exception.kind, StoreErrorKind.MALFORMED)

    def test_prepare_patch_live_none_unchanged_false(self) -> None:
        desired = {
            "metadata": {"name": "my-sec", "namespace": "my-ns"},
            "stringData": {"ca.crt": "bundle"},
        }
        res = prepare_patch(desired, None)
        self.assertFalse(res.unchanged)
        self.assertEqual(res.patch["metadata"]["name"], "my-sec")

    def test_prepare_patch_identical_desired_and_live(self) -> None:
        desired = {
            "metadata": {
                "name": "my-sec",
                "namespace": "my-ns",
                "annotations": {"service-k8s.axiom.dev/trust-bundle": "pool-a"},
                "labels": {"app.kubernetes.io/name": "lumen"},
            },
            "stringData": {"ca.crt": "bundle-a"},
        }
        live = {
            "metadata": {
                "name": "my-sec",
                "namespace": "my-ns",
                "annotations": {"service-k8s.axiom.dev/trust-bundle": "pool-a"},
                "labels": {"app.kubernetes.io/name": "lumen"},
            },
            "data": {"ca.crt": "bundle-a"},
            "type": "Opaque",
        }
        res = prepare_patch(desired, live)
        self.assertTrue(res.unchanged)

    def test_prepare_patch_backfills_data(self) -> None:
        desired = {
            "metadata": {"name": "my-sec", "namespace": "my-ns"},
            "stringData": {"ca.crt": "BUNDLE"},
        }
        live = {
            "metadata": {"name": "my-sec", "namespace": "my-ns"},
            "data": {"tls.crt": "LEAF", "tls.key": "KEY", "ca.crt": "BUNDLE"},
            "type": "Opaque",
        }
        res = prepare_patch(desired, live)
        string_data = res.patch["stringData"]
        assert isinstance(string_data, dict)
        self.assertEqual(string_data.get("ca.crt"), "BUNDLE")
        self.assertEqual(string_data.get("tls.crt"), "LEAF")
        self.assertEqual(string_data.get("tls.key"), "KEY")

    def test_prepare_patch_trust_only_already_published_unchanged_true(
        self,
    ) -> None:
        desired = {
            "metadata": {"name": "my-sec", "namespace": "my-ns"},
            "stringData": {"ca.crt": "BUNDLE"},
        }
        live = {
            "metadata": {"name": "my-sec", "namespace": "my-ns"},
            "data": {"tls.crt": "LEAF", "tls.key": "KEY", "ca.crt": "BUNDLE"},
            "type": "Opaque",
        }
        res = prepare_patch(desired, live)
        self.assertTrue(res.unchanged)

    def test_prepare_patch_changed_tls_crt_unchanged_false(self) -> None:
        desired = {
            "metadata": {"name": "my-sec", "namespace": "my-ns"},
            "stringData": {
                "tls.crt": "NEW-LEAF",
                "tls.key": "KEY",
                "ca.crt": "BUNDLE",
            },
        }
        live = {
            "metadata": {"name": "my-sec", "namespace": "my-ns"},
            "data": {"tls.crt": "OLD-LEAF", "tls.key": "KEY", "ca.crt": "BUNDLE"},
            "type": "Opaque",
        }
        res = prepare_patch(desired, live)
        self.assertFalse(res.unchanged)

    def test_prepare_patch_changed_annotation_unchanged_false(self) -> None:
        desired = {
            "metadata": {
                "name": "my-sec",
                "namespace": "my-ns",
                "annotations": {"service-k8s.axiom.dev/trust-bundle": "pool-b"},
            },
            "stringData": {"ca.crt": "BUNDLE"},
        }
        live = {
            "metadata": {
                "name": "my-sec",
                "namespace": "my-ns",
                "annotations": {"service-k8s.axiom.dev/trust-bundle": "pool-a"},
            },
            "data": {"ca.crt": "BUNDLE"},
            "type": "Opaque",
        }
        res = prepare_patch(desired, live)
        self.assertFalse(res.unchanged)

    def test_prepare_patch_changed_label_unchanged_false(self) -> None:
        desired = {
            "metadata": {
                "name": "my-sec",
                "namespace": "my-ns",
                "labels": {"app.kubernetes.io/name": "lumen-new"},
            },
            "stringData": {"ca.crt": "BUNDLE"},
        }
        live = {
            "metadata": {
                "name": "my-sec",
                "namespace": "my-ns",
                "labels": {"app.kubernetes.io/name": "lumen-old"},
            },
            "data": {"ca.crt": "BUNDLE"},
            "type": "Opaque",
        }
        res = prepare_patch(desired, live)
        self.assertFalse(res.unchanged)

    def test_prepare_patch_labels_not_backfilled(self) -> None:
        desired = {
            "metadata": {"name": "my-sec", "namespace": "my-ns", "labels": {}},
            "stringData": {"ca.crt": "BUNDLE"},
        }
        live = {
            "metadata": {
                "name": "my-sec",
                "namespace": "my-ns",
                "labels": {"app.kubernetes.io/component": "peer-tls"},
            },
            "data": {"ca.crt": "BUNDLE"},
            "type": "Opaque",
        }
        res = prepare_patch(desired, live)
        patch_labels = res.patch["metadata"]["labels"]
        assert isinstance(patch_labels, dict)
        self.assertNotIn("app.kubernetes.io/component", patch_labels)
        self.assertFalse(res.unchanged)

    def test_prepare_patch_non_lifecycle_key_ignored(self) -> None:
        desired = {
            "metadata": {"name": "my-sec", "namespace": "my-ns"},
            "stringData": {"ca.crt": "BUNDLE"},
        }
        live = {
            "metadata": {"name": "my-sec", "namespace": "my-ns"},
            "data": {"ca.crt": "BUNDLE", "other.key": "OTHER-DATA"},
            "type": "Opaque",
        }
        res = prepare_patch(desired, live)
        self.assertTrue(res.unchanged)
        string_data = res.patch["stringData"]
        assert isinstance(string_data, dict)
        self.assertNotIn("other.key", string_data)

    def test_prepare_patch_owner_uid_change_unchanged_false(self) -> None:
        owner = {
            "apiVersion": "v1",
            "kind": "Deployment",
            "name": "dep",
            "uid": "u1",
            "controller": True,
            "blockOwnerDeletion": True,
        }
        desired = {
            "metadata": {
                "name": "my-sec",
                "namespace": "my-ns",
                "ownerReferences": [owner],
            },
            "stringData": {"ca.crt": "BUNDLE"},
        }
        live_owner = dict(owner)
        live_owner["uid"] = "u2"
        live = {
            "metadata": {
                "name": "my-sec",
                "namespace": "my-ns",
                "ownerReferences": [live_owner],
            },
            "data": {"ca.crt": "BUNDLE"},
            "type": "Opaque",
        }
        res = prepare_patch(desired, live)
        self.assertFalse(res.unchanged)

    def test_prepare_patch_extra_live_owner_ref_unchanged_true(self) -> None:
        owner = {
            "apiVersion": "v1",
            "kind": "Deployment",
            "name": "dep",
            "uid": "u1",
            "controller": True,
            "blockOwnerDeletion": True,
        }
        desired = {
            "metadata": {
                "name": "my-sec",
                "namespace": "my-ns",
                "ownerReferences": [owner],
            },
            "stringData": {"ca.crt": "BUNDLE"},
        }
        other_owner = {
            "apiVersion": "v1",
            "kind": "StatefulSet",
            "name": "sts",
            "uid": "u-other",
            "controller": False,
            "blockOwnerDeletion": True,
        }
        live = {
            "metadata": {
                "name": "my-sec",
                "namespace": "my-ns",
                "ownerReferences": [owner, other_owner],
            },
            "data": {"ca.crt": "BUNDLE"},
            "type": "Opaque",
        }
        res = prepare_patch(desired, live)
        self.assertTrue(res.unchanged)

    def test_prepare_patch_live_type_not_opaque_unchanged_false(self) -> None:
        desired = {
            "metadata": {"name": "my-sec", "namespace": "my-ns"},
            "stringData": {"ca.crt": "BUNDLE"},
        }
        live = {
            "metadata": {"name": "my-sec", "namespace": "my-ns"},
            "data": {"ca.crt": "BUNDLE"},
            "type": "kubernetes.io/tls",
        }
        res = prepare_patch(desired, live)
        self.assertFalse(res.unchanged)

    def test_prepare_patch_structure(self) -> None:
        desired = {
            "metadata": {"name": "my-sec", "namespace": "my-ns"},
            "stringData": {"ca.crt": "BUNDLE"},
        }
        res = prepare_patch(desired, None)
        self.assertEqual(res.patch["type"], "Opaque")
        self.assertEqual(FIELD_MANAGER, "service-k8s-certificate")


if __name__ == "__main__":
    unittest.main()
