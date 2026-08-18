"""Unit tests for domain service account parsing, review logic, and cache policy."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

SRC_ROOT = Path(__file__).resolve().parents[2] / "src"
if str(SRC_ROOT) not in sys.path:
    sys.path.insert(0, str(SRC_ROOT))

from service_auth.domain.cache_policy import CacheOutcome, CachePolicy, classify
from service_auth.domain.review import AccessReviewOutcome, ResourceAttributes
from service_auth.domain.service_account import (
    ANONYMOUS_USERNAME,
    UNAUTHENTICATED_GROUP,
    PrincipalRejection,
    ReviewedIdentity,
    ServiceAccountRef,
    is_dns1123_label,
    normalize_extra,
    parse_service_account,
    principal_from_review,
)


class TestDomainDelegation(unittest.TestCase):
    def test_is_dns1123_label(self) -> None:
        self.assertTrue(is_dns1123_label("default"))
        self.assertTrue(is_dns1123_label("my-app-1"))
        self.assertFalse(is_dns1123_label(""))
        self.assertFalse(is_dns1123_label("-start-hyphen"))
        self.assertFalse(is_dns1123_label("end-hyphen-"))
        self.assertFalse(is_dns1123_label("Upper"))
        self.assertFalse(is_dns1123_label("a" * 64))

    def test_parse_service_account(self) -> None:
        self.assertEqual(
            parse_service_account(""), PrincipalRejection.MISSING_USERNAME
        )
        self.assertEqual(
            parse_service_account(ANONYMOUS_USERNAME), PrincipalRejection.ANONYMOUS
        )
        self.assertEqual(
            parse_service_account("user@example.com"),
            PrincipalRejection.NOT_A_SERVICE_ACCOUNT,
        )
        self.assertEqual(
            parse_service_account("system:serviceaccount:ns:name:extra"),
            PrincipalRejection.MALFORMED_SERVICE_ACCOUNT,
        )
        self.assertEqual(
            parse_service_account("system:serviceaccount:ns:Name"),
            PrincipalRejection.MALFORMED_SERVICE_ACCOUNT,
        )

        ref = parse_service_account("system:serviceaccount:prod:web-svc")
        self.assertIsInstance(ref, ServiceAccountRef)
        assert isinstance(ref, ServiceAccountRef)
        self.assertEqual(ref.namespace, "prod")
        self.assertEqual(ref.name, "web-svc")

    def test_principal_from_review(self) -> None:
        identity = ReviewedIdentity("system:serviceaccount:default:sa", "123", (), ())
        self.assertEqual(
            principal_from_review(False, identity), PrincipalRejection.NOT_AUTHENTICATED
        )

        unauth_identity = ReviewedIdentity(
            "system:serviceaccount:default:sa",
            "123",
            (UNAUTHENTICATED_GROUP,),
            (),
        )
        self.assertEqual(
            principal_from_review(True, unauth_identity), PrincipalRejection.ANONYMOUS
        )

        ref = principal_from_review(True, identity)
        self.assertIsInstance(ref, ServiceAccountRef)
        assert isinstance(ref, ServiceAccountRef)
        self.assertEqual(ref.namespace, "default")
        self.assertEqual(ref.name, "sa")

    def test_normalize_extra_is_sorted_and_hashable(self) -> None:
        extra = normalize_extra({"zeta": ["z1"], "alpha": ["a1", "a2"]})
        self.assertEqual(extra, (("alpha", ("a1", "a2")), ("zeta", ("z1",))))
        identity = ReviewedIdentity("system:serviceaccount:ns:sa", "1", (), extra)
        self.assertIn(identity, {identity})

    def test_access_review_deny_outranks_allow(self) -> None:
        self.assertTrue(AccessReviewOutcome(allowed=True, denied=False).is_allowed())
        self.assertFalse(AccessReviewOutcome(allowed=True, denied=True).is_allowed())
        self.assertFalse(AccessReviewOutcome(allowed=False, denied=False).is_allowed())

    def test_resource_attributes_describe(self) -> None:
        attr = ResourceAttributes("apps", "default", "deployments", "web", "get")
        self.assertEqual(attr.describe(), "get apps/deployments/web in default")
        attr_no_name = ResourceAttributes("apps", "default", "deployments", None, "list")
        self.assertEqual(attr_no_name.describe(), "list apps/deployments in default")

    def test_cache_policy_classification(self) -> None:
        policy = CachePolicy(
            allow_ttl_seconds=300,
            deny_ttl_seconds=30,
            stale_window_seconds=60,
        )
        self.assertEqual(policy.ttl_for(True), 300)
        self.assertEqual(policy.ttl_for(False), 30)
        self.assertEqual(policy.revocation_bound_seconds(), 360)

        self.assertEqual(classify(policy, 100, 399, True), CacheOutcome.HIT)
        self.assertEqual(classify(policy, 100, 400, True), CacheOutcome.STALE)
        self.assertEqual(classify(policy, 100, 460, True), CacheOutcome.MISS)


if __name__ == "__main__":
    unittest.main()
