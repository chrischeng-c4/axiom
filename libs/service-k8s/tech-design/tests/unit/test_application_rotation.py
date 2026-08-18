from __future__ import annotations

import sys
import unittest
from datetime import datetime, timedelta, timezone

sys.path.insert(0, __file__.rsplit("/", 3)[0] + "/src")

from service_k8s.application.rotation import (
    ACTIVATION_RECHECK,
    RETRY_BASE_SECS,
    RETRY_CEILING_SECS,
    Action,
    AwaitActivation,
    Desired,
    Issue,
    IssueReason,
    IssuerId,
    Observed,
    ObservedLeaf,
    PublishTrustBundle,
    RetireIssuers,
    Wait,
    _fingerprint_offset,
    next_action,
    renew_at,
    retry_after,
)
from service_k8s.domain.profile import (
    CertificateIdentity,
    CertificateProfile,
)
from service_k8s.domain.purpose import Purpose
from service_k8s.domain.scope import InstanceScope


def make_profile(
    renew_before_secs: int = 3600,
    renew_jitter_secs: int = 0,
    lifetime_secs: int = 86400,
) -> CertificateProfile:
    scope = InstanceScope(
        namespace="lumen",
        instance="lumen",
        trust_domain="lumen-prod.svc.id.goog",
    )
    identity = CertificateIdentity(
        dns_names=("lumen.lumen.svc.cluster.local",),
        spiffe_uri=None,
    )
    return CertificateProfile(
        scope=scope,
        purpose=Purpose.SERVING,
        common_name="lumen.lumen.svc.cluster.local",
        identity=identity,
        lifetime_secs=lifetime_secs,
        renew_before_secs=renew_before_secs,
        renew_jitter_secs=renew_jitter_secs,
    )


class TestApplicationRotation(unittest.TestCase):
    # --- Ladder order & Ordering Witnesses (11) ---
    def test_step1_trust_bundle_missing_returns_publish_trust_bundle(
        self,
    ) -> None:
        prof = make_profile()
        desired = Desired(prof, IssuerId("issuer-a"))
        obs = Observed(trust_bundle=())
        now = datetime(2026, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        action = next_action(desired, obs, now)
        self.assertEqual(action, PublishTrustBundle((IssuerId("issuer-a"),)))

    def test_witness_untrusted_issuer_no_leaf_returns_publish_trust_bundle(
        self,
    ) -> None:
        prof = make_profile()
        desired = Desired(prof, IssuerId("issuer-b"))
        obs = Observed(leaf=None, trust_bundle=(IssuerId("issuer-a"),))
        now = datetime(2026, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        action = next_action(desired, obs, now)
        self.assertEqual(
            action, PublishTrustBundle((IssuerId("issuer-a"), IssuerId("issuer-b")))
        )
        self.assertNotIsInstance(action, Issue)

    def test_witness_untrusted_issuer_expired_leaf_returns_publish_trust_bundle(
        self,
    ) -> None:
        prof = make_profile()
        desired = Desired(prof, IssuerId("issuer-b"))
        t_before = datetime(2026, 1, 1, 0, 0, 0, tzinfo=timezone.utc)
        t_after = datetime(2026, 1, 2, 0, 0, 0, tzinfo=timezone.utc)
        leaf = ObservedLeaf(
            issuer=IssuerId("issuer-a"),
            not_before=t_before,
            not_after=t_after,
            fingerprint="fp1",
            identity_digest=prof.identity_digest(),
        )
        obs = Observed(leaf=leaf, trust_bundle=(IssuerId("issuer-a"),))
        now = datetime(2026, 1, 3, 0, 0, 0, tzinfo=timezone.utc)
        action = next_action(desired, obs, now)
        self.assertEqual(
            action, PublishTrustBundle((IssuerId("issuer-a"), IssuerId("issuer-b")))
        )

    def test_witness_wrong_issuer_and_wrong_identity_returns_issuer_rotation(
        self,
    ) -> None:
        prof = make_profile()
        desired = Desired(prof, IssuerId("issuer-b"))
        t_before = datetime(2026, 1, 1, 0, 0, 0, tzinfo=timezone.utc)
        t_after = datetime(2026, 1, 2, 0, 0, 0, tzinfo=timezone.utc)
        leaf = ObservedLeaf(
            issuer=IssuerId("issuer-a"),
            not_before=t_before,
            not_after=t_after,
            fingerprint="fp1",
            identity_digest="wrong-digest",
        )
        obs = Observed(
            leaf=leaf,
            trust_bundle=(IssuerId("issuer-a"), IssuerId("issuer-b")),
        )
        now = datetime(2026, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        action = next_action(desired, obs, now)
        self.assertEqual(
            action, Issue(IssuerId("issuer-b"), IssueReason.ISSUER_ROTATION)
        )

    def test_witness_expired_leaf_and_stale_issuers_returns_expired(
        self,
    ) -> None:
        prof = make_profile()
        desired = Desired(prof, IssuerId("issuer-b"))
        t_before = datetime(2026, 1, 1, 0, 0, 0, tzinfo=timezone.utc)
        t_after = datetime(2026, 1, 2, 0, 0, 0, tzinfo=timezone.utc)
        leaf = ObservedLeaf(
            issuer=IssuerId("issuer-b"),
            not_before=t_before,
            not_after=t_after,
            fingerprint="fp1",
            identity_digest=prof.identity_digest(),
        )
        obs = Observed(
            leaf=leaf,
            trust_bundle=(IssuerId("issuer-a"), IssuerId("issuer-b")),
            activated_fingerprint=None,
        )
        now = datetime(2026, 1, 3, 0, 0, 0, tzinfo=timezone.utc)
        action = next_action(desired, obs, now)
        self.assertEqual(
            action, Issue(IssuerId("issuer-b"), IssueReason.EXPIRED)
        )

    def test_witness_correct_leaf_stale_issuers_not_activated_returns_await_activation(
        self,
    ) -> None:
        prof = make_profile()
        desired = Desired(prof, IssuerId("issuer-b"))
        t_before = datetime(2026, 1, 1, 0, 0, 0, tzinfo=timezone.utc)
        t_after = datetime(2026, 1, 10, 0, 0, 0, tzinfo=timezone.utc)
        leaf = ObservedLeaf(
            issuer=IssuerId("issuer-b"),
            not_before=t_before,
            not_after=t_after,
            fingerprint="fp1",
            identity_digest=prof.identity_digest(),
        )
        obs = Observed(
            leaf=leaf,
            trust_bundle=(IssuerId("issuer-a"), IssuerId("issuer-b")),
            activated_fingerprint=None,
        )
        now = datetime(2026, 1, 2, 0, 0, 0, tzinfo=timezone.utc)
        action = next_action(desired, obs, now)
        self.assertEqual(
            action, AwaitActivation("fp1", timedelta(seconds=15))
        )

    def test_step2_no_leaf_returns_bootstrap(self) -> None:
        prof = make_profile()
        desired = Desired(prof, IssuerId("issuer-a"))
        obs = Observed(trust_bundle=(IssuerId("issuer-a"),))
        now = datetime(2026, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        action = next_action(desired, obs, now)
        self.assertEqual(
            action, Issue(IssuerId("issuer-a"), IssueReason.BOOTSTRAP)
        )

    def test_step3_wrong_issuer_returns_issuer_rotation(self) -> None:
        prof = make_profile()
        desired = Desired(prof, IssuerId("issuer-b"))
        leaf = ObservedLeaf(
            issuer=IssuerId("issuer-a"),
            not_before=datetime(2026, 1, 1, tzinfo=timezone.utc),
            not_after=datetime(2026, 1, 10, tzinfo=timezone.utc),
            fingerprint="fp1",
            identity_digest=prof.identity_digest(),
        )
        obs = Observed(
            leaf=leaf, trust_bundle=(IssuerId("issuer-a"), IssuerId("issuer-b"))
        )
        now = datetime(2026, 1, 2, tzinfo=timezone.utc)
        action = next_action(desired, obs, now)
        self.assertEqual(
            action, Issue(IssuerId("issuer-b"), IssueReason.ISSUER_ROTATION)
        )

    def test_step4_wrong_identity_returns_identity_changed(self) -> None:
        prof = make_profile()
        desired = Desired(prof, IssuerId("issuer-a"))
        leaf = ObservedLeaf(
            issuer=IssuerId("issuer-a"),
            not_before=datetime(2026, 1, 1, tzinfo=timezone.utc),
            not_after=datetime(2026, 1, 10, tzinfo=timezone.utc),
            fingerprint="fp1",
            identity_digest="old-digest",
        )
        obs = Observed(leaf=leaf, trust_bundle=(IssuerId("issuer-a"),))
        now = datetime(2026, 1, 2, tzinfo=timezone.utc)
        action = next_action(desired, obs, now)
        self.assertEqual(
            action, Issue(IssuerId("issuer-a"), IssueReason.IDENTITY_CHANGED)
        )

    def test_step5_expired_leaf_returns_expired(self) -> None:
        prof = make_profile()
        desired = Desired(prof, IssuerId("issuer-a"))
        leaf = ObservedLeaf(
            issuer=IssuerId("issuer-a"),
            not_before=datetime(2026, 1, 1, tzinfo=timezone.utc),
            not_after=datetime(2026, 1, 2, tzinfo=timezone.utc),
            fingerprint="fp1",
            identity_digest=prof.identity_digest(),
        )
        obs = Observed(leaf=leaf, trust_bundle=(IssuerId("issuer-a"),))
        now = datetime(2026, 1, 2, tzinfo=timezone.utc)
        action = next_action(desired, obs, now)
        self.assertEqual(
            action, Issue(IssuerId("issuer-a"), IssueReason.EXPIRED)
        )

    def test_step7_routine_renewal(self) -> None:
        prof = make_profile(renew_before_secs=3600, renew_jitter_secs=0)
        desired = Desired(prof, IssuerId("issuer-a"))
        t_after = datetime(2026, 1, 2, 12, 0, 0, tzinfo=timezone.utc)
        leaf = ObservedLeaf(
            issuer=IssuerId("issuer-a"),
            not_before=datetime(2026, 1, 1, tzinfo=timezone.utc),
            not_after=t_after,
            fingerprint="fp1",
            identity_digest=prof.identity_digest(),
        )
        obs = Observed(leaf=leaf, trust_bundle=(IssuerId("issuer-a"),))
        now = datetime(2026, 1, 2, 11, 0, 0, tzinfo=timezone.utc)
        action = next_action(desired, obs, now)
        self.assertEqual(
            action, Issue(IssuerId("issuer-a"), IssueReason.RENEWAL)
        )

    # --- Trust bundle (4) ---
    def test_trust_bundle_published_tuple_is_sorted(self) -> None:
        prof = make_profile()
        desired = Desired(prof, IssuerId("b"))
        obs = Observed(trust_bundle=(IssuerId("c"), IssuerId("a")))
        now = datetime(2026, 1, 1, tzinfo=timezone.utc)
        action = next_action(desired, obs, now)
        expected = PublishTrustBundle(
            (IssuerId("a"), IssuerId("b"), IssuerId("c"))
        )
        self.assertEqual(action, expected)

    def test_trust_bundle_contains_existing_plus_desired(self) -> None:
        prof = make_profile()
        desired = Desired(prof, IssuerId("new-issuer"))
        obs = Observed(
            trust_bundle=(IssuerId("old-1"), IssuerId("old-2"))
        )
        now = datetime(2026, 1, 1, tzinfo=timezone.utc)
        action = next_action(desired, obs, now)
        self.assertIsInstance(action, PublishTrustBundle)
        assert isinstance(action, PublishTrustBundle)
        self.assertIn(IssuerId("new-issuer"), action.issuers)

    def test_trust_bundle_deduplication(self) -> None:
        prof = make_profile()
        desired = Desired(prof, IssuerId("a"))
        obs = Observed(trust_bundle=(IssuerId("a"), IssuerId("a")))
        # Desired issuer is in trust_bundle, so step 1 does not trigger
        action = next_action(desired, obs, datetime(2026, 1, 1, tzinfo=timezone.utc))
        self.assertNotIsInstance(action, PublishTrustBundle)

    def test_trust_bundle_idempotent(self) -> None:
        prof = make_profile()
        desired = Desired(prof, IssuerId("b"))
        obs = Observed(trust_bundle=(IssuerId("a"),))
        now = datetime(2026, 1, 1, tzinfo=timezone.utc)
        a1 = next_action(desired, obs, now)
        a2 = next_action(desired, obs, now)
        self.assertEqual(a1, a2)

    # --- Activation gate (4) ---
    def test_activation_stale_issuer_none_fingerprint(self) -> None:
        prof = make_profile()
        desired = Desired(prof, IssuerId("b"))
        leaf = ObservedLeaf(
            issuer=IssuerId("b"),
            not_before=datetime(2026, 1, 1, tzinfo=timezone.utc),
            not_after=datetime(2026, 1, 10, tzinfo=timezone.utc),
            fingerprint="fp1",
            identity_digest=prof.identity_digest(),
        )
        obs = Observed(
            leaf=leaf,
            trust_bundle=(IssuerId("a"), IssuerId("b")),
            activated_fingerprint=None,
        )
        action = next_action(
            desired, obs, datetime(2026, 1, 2, tzinfo=timezone.utc)
        )
        self.assertEqual(
            action, AwaitActivation("fp1", timedelta(seconds=15))
        )

    def test_activation_stale_issuer_different_fingerprint(self) -> None:
        prof = make_profile()
        desired = Desired(prof, IssuerId("b"))
        leaf = ObservedLeaf(
            issuer=IssuerId("b"),
            not_before=datetime(2026, 1, 1, tzinfo=timezone.utc),
            not_after=datetime(2026, 1, 10, tzinfo=timezone.utc),
            fingerprint="fp1",
            identity_digest=prof.identity_digest(),
        )
        obs = Observed(
            leaf=leaf,
            trust_bundle=(IssuerId("a"), IssuerId("b")),
            activated_fingerprint="fp-old",
        )
        action = next_action(
            desired, obs, datetime(2026, 1, 2, tzinfo=timezone.utc)
        )
        self.assertEqual(
            action, AwaitActivation("fp1", timedelta(seconds=15))
        )

    def test_activation_stale_issuer_matching_fingerprint(self) -> None:
        prof = make_profile()
        desired = Desired(prof, IssuerId("b"))
        leaf = ObservedLeaf(
            issuer=IssuerId("b"),
            not_before=datetime(2026, 1, 1, tzinfo=timezone.utc),
            not_after=datetime(2026, 1, 10, tzinfo=timezone.utc),
            fingerprint="fp1",
            identity_digest=prof.identity_digest(),
        )
        obs = Observed(
            leaf=leaf,
            trust_bundle=(IssuerId("a"), IssuerId("b")),
            activated_fingerprint="fp1",
        )
        action = next_action(
            desired, obs, datetime(2026, 1, 2, tzinfo=timezone.utc)
        )
        self.assertEqual(action, RetireIssuers((IssuerId("a"),)))

    def test_activation_retired_issuers_order_and_exclusion(self) -> None:
        prof = make_profile()
        desired = Desired(prof, IssuerId("desired"))
        leaf = ObservedLeaf(
            issuer=IssuerId("desired"),
            not_before=datetime(2026, 1, 1, tzinfo=timezone.utc),
            not_after=datetime(2026, 1, 10, tzinfo=timezone.utc),
            fingerprint="fp1",
            identity_digest=prof.identity_digest(),
        )
        obs = Observed(
            leaf=leaf,
            trust_bundle=(
                IssuerId("old2"),
                IssuerId("desired"),
                IssuerId("old1"),
            ),
            activated_fingerprint="fp1",
        )
        action = next_action(
            desired, obs, datetime(2026, 1, 2, tzinfo=timezone.utc)
        )
        expected = RetireIssuers((IssuerId("old2"), IssuerId("old1")))
        self.assertEqual(action, expected)
        self.assertNotIn(IssuerId("desired"), action.issuers)

    # --- Boundaries (4) ---
    def test_boundary_exact_not_after_is_expired(self) -> None:
        prof = make_profile()
        desired = Desired(prof, IssuerId("a"))
        t_after = datetime(2026, 1, 2, 12, 0, 0, tzinfo=timezone.utc)
        leaf = ObservedLeaf(
            issuer=IssuerId("a"),
            not_before=datetime(2026, 1, 1, tzinfo=timezone.utc),
            not_after=t_after,
            fingerprint="fp1",
            identity_digest=prof.identity_digest(),
        )
        obs = Observed(leaf=leaf, trust_bundle=(IssuerId("a"),))
        action = next_action(desired, obs, t_after)
        self.assertEqual(action, Issue(IssuerId("a"), IssueReason.EXPIRED))

    def test_boundary_one_microsecond_before_not_after_not_expired(
        self,
    ) -> None:
        prof = make_profile(renew_before_secs=3600)
        desired = Desired(prof, IssuerId("a"))
        t_after = datetime(2026, 1, 2, 12, 0, 0, tzinfo=timezone.utc)
        leaf = ObservedLeaf(
            issuer=IssuerId("a"),
            not_before=datetime(2026, 1, 1, tzinfo=timezone.utc),
            not_after=t_after,
            fingerprint="fp1",
            identity_digest=prof.identity_digest(),
        )
        obs = Observed(leaf=leaf, trust_bundle=(IssuerId("a"),))
        t_due = t_after - timedelta(seconds=3600)
        # Check at t_due - 1us
        now = t_due - timedelta(microseconds=1)
        action = next_action(desired, obs, now)
        self.assertEqual(action, Wait(t_due))

    def test_boundary_exact_due_is_renewal(self) -> None:
        prof = make_profile(renew_before_secs=3600, renew_jitter_secs=0)
        desired = Desired(prof, IssuerId("a"))
        t_after = datetime(2026, 1, 2, 12, 0, 0, tzinfo=timezone.utc)
        leaf = ObservedLeaf(
            issuer=IssuerId("a"),
            not_before=datetime(2026, 1, 1, tzinfo=timezone.utc),
            not_after=t_after,
            fingerprint="fp1",
            identity_digest=prof.identity_digest(),
        )
        obs = Observed(leaf=leaf, trust_bundle=(IssuerId("a"),))
        t_due = renew_at(prof, leaf)
        action = next_action(desired, obs, t_due)
        self.assertEqual(action, Issue(IssuerId("a"), IssueReason.RENEWAL))

    def test_boundary_one_microsecond_before_due_is_wait(self) -> None:
        prof = make_profile(renew_before_secs=3600, renew_jitter_secs=0)
        desired = Desired(prof, IssuerId("a"))
        t_after = datetime(2026, 1, 2, 12, 0, 0, tzinfo=timezone.utc)
        leaf = ObservedLeaf(
            issuer=IssuerId("a"),
            not_before=datetime(2026, 1, 1, tzinfo=timezone.utc),
            not_after=t_after,
            fingerprint="fp1",
            identity_digest=prof.identity_digest(),
        )
        obs = Observed(leaf=leaf, trust_bundle=(IssuerId("a"),))
        t_due = renew_at(prof, leaf)
        now = t_due - timedelta(microseconds=1)
        action = next_action(desired, obs, now)
        self.assertEqual(action, Wait(until=t_due))

    # --- renew_at (5) ---
    def test_renew_at_zero_jitter(self) -> None:
        prof = make_profile(renew_before_secs=3600, renew_jitter_secs=0)
        t_after = datetime(2026, 1, 2, 12, 0, 0, tzinfo=timezone.utc)
        leaf = ObservedLeaf(
            issuer=IssuerId("a"),
            not_before=datetime(2026, 1, 1, tzinfo=timezone.utc),
            not_after=t_after,
            fingerprint="fp1",
            identity_digest=prof.identity_digest(),
        )
        expected = datetime(2026, 1, 2, 11, 0, 0, tzinfo=timezone.utc)
        self.assertEqual(renew_at(prof, leaf), expected)

    def test_renew_at_with_jitter_within_bounds(self) -> None:
        prof = make_profile(renew_before_secs=3600, renew_jitter_secs=300)
        t_after = datetime(2026, 1, 2, 12, 0, 0, tzinfo=timezone.utc)
        leaf = ObservedLeaf(
            issuer=IssuerId("a"),
            not_before=datetime(2026, 1, 1, tzinfo=timezone.utc),
            not_after=t_after,
            fingerprint="fp1",
            identity_digest=prof.identity_digest(),
        )
        base = t_after - timedelta(seconds=3600)
        due = renew_at(prof, leaf)
        self.assertGreaterEqual(due, base)
        self.assertLess(due, base + timedelta(seconds=300))

    def test_renew_at_deterministic(self) -> None:
        prof = make_profile(renew_before_secs=3600, renew_jitter_secs=300)
        leaf = ObservedLeaf(
            issuer=IssuerId("a"),
            not_before=datetime(2026, 1, 1, tzinfo=timezone.utc),
            not_after=datetime(2026, 1, 2, 12, 0, 0, tzinfo=timezone.utc),
            fingerprint="sample-fingerprint-12345",
            identity_digest=prof.identity_digest(),
        )
        d1 = renew_at(prof, leaf)
        d2 = renew_at(prof, leaf)
        self.assertEqual(d1, d2)

    def test_renew_at_spread_different_leaves(self) -> None:
        prof = make_profile(renew_before_secs=3600, renew_jitter_secs=300)
        t_after = datetime(2026, 1, 2, 12, 0, 0, tzinfo=timezone.utc)
        fps = ("fp_alpha_1", "fp_beta_2", "fp_gamma_3", "fp_delta_4")
        dues = set()
        for fp in fps:
            leaf = ObservedLeaf(
                issuer=IssuerId("a"),
                not_before=datetime(2026, 1, 1, tzinfo=timezone.utc),
                not_after=t_after,
                fingerprint=fp,
                identity_digest=prof.identity_digest(),
            )
            dues.add(renew_at(prof, leaf))
        self.assertGreater(len(dues), 1)

    def test_renew_at_timezone_aware(self) -> None:
        prof = make_profile(renew_before_secs=3600, renew_jitter_secs=100)
        leaf = ObservedLeaf(
            issuer=IssuerId("a"),
            not_before=datetime(2026, 1, 1, tzinfo=timezone.utc),
            not_after=datetime(2026, 1, 2, 12, 0, 0, tzinfo=timezone.utc),
            fingerprint="fp1",
            identity_digest=prof.identity_digest(),
        )
        due = renew_at(prof, leaf)
        self.assertIsNotNone(due.tzinfo)
        self.assertEqual(due.tzinfo, timezone.utc)

    # --- retry_after (9 table tests) ---
    def test_retry_after_0(self) -> None:
        self.assertEqual(retry_after(0), timedelta(seconds=5))

    def test_retry_after_1(self) -> None:
        self.assertEqual(retry_after(1), timedelta(seconds=10))

    def test_retry_after_2(self) -> None:
        self.assertEqual(retry_after(2), timedelta(seconds=20))

    def test_retry_after_3(self) -> None:
        self.assertEqual(retry_after(3), timedelta(seconds=40))

    def test_retry_after_4(self) -> None:
        self.assertEqual(retry_after(4), timedelta(seconds=80))

    def test_retry_after_5(self) -> None:
        self.assertEqual(retry_after(5), timedelta(seconds=160))

    def test_retry_after_6(self) -> None:
        self.assertEqual(retry_after(6), timedelta(seconds=300))

    def test_retry_after_7(self) -> None:
        self.assertEqual(retry_after(7), timedelta(seconds=300))

    def test_retry_after_40(self) -> None:
        self.assertEqual(retry_after(40), timedelta(seconds=300))

    # --- No memory (2) ---
    def test_no_memory_consecutive_failures_ignored(self) -> None:
        prof = make_profile()
        desired = Desired(prof, IssuerId("a"))
        obs0 = Observed(trust_bundle=(IssuerId("a"),), consecutive_failures=0)
        obs99 = Observed(trust_bundle=(IssuerId("a"),), consecutive_failures=99)
        now = datetime(2026, 1, 1, tzinfo=timezone.utc)
        self.assertEqual(
            next_action(desired, obs0, now), next_action(desired, obs99, now)
        )

    def test_no_memory_reconcile_idempotence(self) -> None:
        prof = make_profile()
        desired = Desired(prof, IssuerId("a"))
        obs = Observed(trust_bundle=(IssuerId("a"),))
        now = datetime(2026, 1, 1, tzinfo=timezone.utc)
        self.assertEqual(
            next_action(desired, obs, now), next_action(desired, obs, now)
        )

    # --- Closed action set (1) ---
    def test_closed_action_set_no_removal_names(self) -> None:
        action_classes = (
            PublishTrustBundle,
            Issue,
            AwaitActivation,
            RetireIssuers,
            Wait,
        )
        forbidden_substrings = ("remove", "revoke", "delete", "clear", "reset")
        for cls in action_classes:
            name_lower = cls.__name__.lower()
            for sub in forbidden_substrings:
                self.assertNotIn(sub, name_lower)


if __name__ == "__main__":
    unittest.main()
