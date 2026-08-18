from __future__ import annotations

from dataclasses import FrozenInstanceError
import sys
import unittest

sys.path.insert(0, __file__.rsplit("/", 3)[0] + "/src")

from service_k8s.domain.purpose import Purpose
from service_k8s.domain.scope import InstanceScope


class TestDomainScope(unittest.TestCase):
    def test_secret_name_serving(self) -> None:
        scope = InstanceScope("lumen", "lumen", "lumen-prod.svc.id.goog")
        self.assertEqual(scope.secret_name(Purpose.SERVING), "lumen-serving-tls")

    def test_secret_name_peer(self) -> None:
        scope = InstanceScope("lumen", "lumen", "lumen-prod.svc.id.goog")
        self.assertEqual(scope.secret_name(Purpose.PEER), "lumen-peer-tls")

    def test_secret_names_differ_by_purpose(self) -> None:
        scope = InstanceScope("lumen", "lumen", "lumen-prod.svc.id.goog")
        self.assertNotEqual(
            scope.secret_name(Purpose.SERVING),
            scope.secret_name(Purpose.PEER),
        )

    def test_spiffe_prefix_exact_string(self) -> None:
        scope = InstanceScope("lumen", "lumen", "lumen-prod.svc.id.goog")
        expected = "spiffe://lumen-prod.svc.id.goog/ns/lumen/"
        self.assertEqual(scope.spiffe_prefix(), expected)

    def test_spiffe_prefix_ends_with_slash(self) -> None:
        scope = InstanceScope("lumen", "lumen", "lumen-prod.svc.id.goog")
        self.assertTrue(scope.spiffe_prefix().endswith("/"))

    def test_covers_equal_scope(self) -> None:
        scope1 = InstanceScope("lumen", "lumen", "lumen-prod.svc.id.goog")
        scope2 = InstanceScope("lumen", "lumen", "lumen-prod.svc.id.goog")
        self.assertTrue(scope1.covers(scope2))

    def test_covers_false_when_namespace_differs(self) -> None:
        scope1 = InstanceScope("lumen", "lumen", "lumen-prod.svc.id.goog")
        scope2 = InstanceScope("other", "lumen", "lumen-prod.svc.id.goog")
        self.assertFalse(scope1.covers(scope2))

    def test_covers_false_when_instance_differs(self) -> None:
        scope1 = InstanceScope("lumen", "lumen", "lumen-prod.svc.id.goog")
        scope2 = InstanceScope("lumen", "other", "lumen-prod.svc.id.goog")
        self.assertFalse(scope1.covers(scope2))

    def test_covers_false_when_trust_domain_differs(self) -> None:
        scope1 = InstanceScope("lumen", "lumen", "lumen-prod.svc.id.goog")
        scope2 = InstanceScope("lumen", "lumen", "other.svc.id.goog")
        self.assertFalse(scope1.covers(scope2))

    def test_dataclass_is_frozen(self) -> None:
        scope = InstanceScope("lumen", "lumen", "lumen-prod.svc.id.goog")
        with self.assertRaises(FrozenInstanceError):
            scope.namespace = "x"  # type: ignore[misc]


if __name__ == "__main__":
    unittest.main()
