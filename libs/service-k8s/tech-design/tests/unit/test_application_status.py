from __future__ import annotations

import sys
import unittest
from datetime import datetime, timezone

sys.path.insert(0, __file__.rsplit("/", 3)[0] + "/src")

from service_k8s.application.rotation import (
    ACTIVATION_RECHECK,
    AwaitActivation,
    Issue,
    IssueReason,
    IssuerId,
    PublishTrustBundle,
    RetireIssuers,
    Wait,
)
from service_k8s.application.status import (
    READY_CONDITION,
    ROTATING_CONDITION,
    CertificateFacts,
    condition_prefix,
    redact,
    rotation_detail,
    rotation_reason,
    short_fingerprint,
)
from service_k8s.domain.condition import ConditionStatus
from service_k8s.domain.purpose import Purpose


class TestApplicationStatus(unittest.TestCase):
    # --- 7.1 redact (14) ---
    def test_redact_pem_private_key_block_replaced(self) -> None:
        raw = "issued -----BEGIN PRIVATE KEY-----\nMIIEvQIBADANBg\n-----END PRIVATE KEY----- ok"
        res = redact(raw)
        self.assertIn("[redacted pem]", res)
        self.assertNotIn("MIIEvQIBADANBg", res)
        self.assertEqual(res, "issued [redacted pem] ok")

    def test_redact_pem_surrounding_text_survives(self) -> None:
        raw = "before -----BEGIN PRIVATE KEY-----\nsecret\n-----END PRIVATE KEY----- after"
        res = redact(raw)
        self.assertTrue(res.startswith("before "))
        self.assertTrue(res.endswith(" after"))

    def test_redact_pem_two_blocks_replaced(self) -> None:
        raw = "-----BEGIN CERT-----\nA\n-----END CERT----- middle -----BEGIN CERT-----\nB\n-----END CERT-----"
        res = redact(raw)
        self.assertEqual(res.count("[redacted pem]"), 2)

    def test_redact_pem_unterminated_begin_drops_tail(self) -> None:
        raw = "prefix -----BEGIN PRIVATE KEY-----\nsecret payload with no end"
        res = redact(raw)
        self.assertEqual(res, "prefix [redacted pem]")

    def test_redact_plain_text_whitespace_normalized(self) -> None:
        raw = "hello   world"
        self.assertEqual(redact(raw), "hello world")

    def test_redact_bearer_token(self) -> None:
        raw = "Authorization: Bearer abc123def"
        self.assertEqual(redact(raw), "Authorization: Bearer [redacted]")

    def test_redact_bearer_token_keeps_trailing_text(self) -> None:
        raw = "GET failed: Bearer abc123def456 status=401"
        self.assertEqual(
            redact(raw), "GET failed: Bearer [redacted] status=401"
        )

    def test_redact_bearer_token_multiple(self) -> None:
        raw = "Bearer t1 and Bearer t2"
        self.assertEqual(redact(raw), "Bearer [redacted] and Bearer [redacted]")

    def test_redact_jwt_token(self) -> None:
        raw = "token eyJhbGciOi.eyJzdWIiOi.SflKxwRJSM done"
        self.assertEqual(redact(raw), "token [redacted token] done")

    def test_redact_jwt_two_part_word_not_redacted(self) -> None:
        raw = "aaaaaaaa.bbbbbbbb"
        self.assertEqual(redact(raw), "aaaaaaaa.bbbbbbbb")

    def test_redact_jwt_four_part_word_not_redacted(self) -> None:
        raw = "aaaaaaaa.bbbbbbbb.cccccccc.dddddddd"
        self.assertEqual(redact(raw), "aaaaaaaa.bbbbbbbb.cccccccc.dddddddd")

    def test_redact_jwt_short_segment_not_redacted(self) -> None:
        raw = "aa.bbbbbbbb.cccccccc"
        self.assertEqual(redact(raw), "aa.bbbbbbbb.cccccccc")

    def test_redact_jwt_invalid_base64url_chars_not_redacted(self) -> None:
        raw = "aaaaaaaa.bbbb+bbb.cccccccc"
        self.assertEqual(redact(raw), "aaaaaaaa.bbbb+bbb.cccccccc")

    def test_redact_whitespace_normalization(self) -> None:
        raw = "a\n\nb   c"
        self.assertEqual(redact(raw), "a b c")

    # --- 7.2 short_fingerprint (2) ---
    def test_short_fingerprint_truncates_64_to_16(self) -> None:
        fp64 = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        self.assertEqual(short_fingerprint(fp64), "0123456789abcdef")

    def test_short_fingerprint_short_input_unchanged(self) -> None:
        self.assertEqual(short_fingerprint("abcd"), "abcd")

    # --- 7.3 from_action (6) ---
    def test_from_action_issue_sets_rotating(self) -> None:
        act = Issue(IssuerId("iss"), IssueReason.RENEWAL)
        facts = CertificateFacts.from_action(
            Purpose.SERVING,
            IssuerId("iss"),
            datetime(2026, 1, 1, tzinfo=timezone.utc),
            "fp64byteslong12345678901234567890123456789012345678901234567890",
            (IssuerId("iss"),),
            0,
            act,
        )
        self.assertEqual(facts.rotating, IssueReason.RENEWAL)

    def test_from_action_publish_trust_bundle_clears_rotating(self) -> None:
        act = PublishTrustBundle((IssuerId("iss"),))
        facts = CertificateFacts.from_action(
            Purpose.SERVING, None, None, None, (), 0, act
        )
        self.assertIsNone(facts.rotating)

    def test_from_action_await_activation_clears_rotating(self) -> None:
        act = AwaitActivation("fp", ACTIVATION_RECHECK)
        facts = CertificateFacts.from_action(
            Purpose.SERVING, None, None, None, (), 0, act
        )
        self.assertIsNone(facts.rotating)

    def test_from_action_retire_issuers_clears_rotating(self) -> None:
        act = RetireIssuers((IssuerId("old"),))
        facts = CertificateFacts.from_action(
            Purpose.SERVING, None, None, None, (), 0, act
        )
        self.assertIsNone(facts.rotating)

    def test_from_action_wait_clears_rotating(self) -> None:
        act = Wait(datetime(2026, 1, 1, tzinfo=timezone.utc))
        facts = CertificateFacts.from_action(
            Purpose.SERVING, None, None, None, (), 0, act
        )
        self.assertIsNone(facts.rotating)

    def test_from_action_shortens_fingerprint(self) -> None:
        act = Wait(datetime(2026, 1, 1, tzinfo=timezone.utc))
        facts = CertificateFacts.from_action(
            Purpose.SERVING,
            IssuerId("iss"),
            datetime(2026, 1, 1, tzinfo=timezone.utc),
            "0123456789abcdefEXTRA",
            (),
            0,
            act,
        )
        self.assertEqual(facts.fingerprint, "0123456789abcdef")

    # --- 7.4 conditions (12) ---
    def test_conditions_returns_two_facts_ready_first(self) -> None:
        facts = CertificateFacts(purpose=Purpose.PEER)
        conds = facts.conditions()
        self.assertEqual(len(conds), 2)
        self.assertEqual(conds[0].type_, "PeerCertificateReady")
        self.assertEqual(conds[1].type_, "PeerCertificateRotating")

    def test_conditions_ready_true_when_issuer_and_not_after_present(
        self,
    ) -> None:
        facts = CertificateFacts(
            purpose=Purpose.SERVING,
            issuer=IssuerId("pool-a"),
            not_after=datetime(2026, 8, 1, tzinfo=timezone.utc),
        )
        ready_fact, _ = facts.conditions()
        self.assertEqual(ready_fact.status, ConditionStatus.TRUE)
        self.assertEqual(ready_fact.reason, "Issued")

    def test_conditions_ready_false_when_not_after_missing(self) -> None:
        facts = CertificateFacts(
            purpose=Purpose.SERVING,
            issuer=IssuerId("pool-a"),
            not_after=None,
        )
        ready_fact, _ = facts.conditions()
        self.assertEqual(ready_fact.status, ConditionStatus.FALSE)
        self.assertEqual(ready_fact.reason, "Pending")

    def test_conditions_ready_false_when_issuer_missing(self) -> None:
        facts = CertificateFacts(
            purpose=Purpose.SERVING,
            issuer=None,
            not_after=datetime(2026, 8, 1, tzinfo=timezone.utc),
        )
        ready_fact, _ = facts.conditions()
        self.assertEqual(ready_fact.status, ConditionStatus.FALSE)
        self.assertEqual(ready_fact.reason, "Pending")

    def test_conditions_ready_false_zero_failures_pending(self) -> None:
        facts = CertificateFacts(purpose=Purpose.PEER, consecutive_failures=0)
        ready_fact, _ = facts.conditions()
        self.assertEqual(ready_fact.status, ConditionStatus.FALSE)
        self.assertEqual(ready_fact.reason, "Pending")
        self.assertEqual(ready_fact.message, "no peer certificate projected yet")

    def test_conditions_ready_false_failures_issuance_failing(self) -> None:
        facts = CertificateFacts(purpose=Purpose.PEER, consecutive_failures=3)
        ready_fact, _ = facts.conditions()
        self.assertEqual(ready_fact.status, ConditionStatus.FALSE)
        self.assertEqual(ready_fact.reason, "IssuanceFailing")
        self.assertEqual(
            ready_fact.message,
            "no peer certificate projected after 3 consecutive attempts",
        )

    def test_conditions_namespaced_by_purpose(self) -> None:
        f_peer = CertificateFacts(purpose=Purpose.PEER)
        c_peer_r, c_peer_rot = f_peer.conditions()
        self.assertEqual(c_peer_r.type_, "PeerCertificateReady")
        self.assertEqual(c_peer_rot.type_, "PeerCertificateRotating")

        f_srv = CertificateFacts(purpose=Purpose.SERVING)
        c_srv_r, c_srv_rot = f_srv.conditions()
        self.assertEqual(c_srv_r.type_, "ServingCertificateReady")
        self.assertEqual(c_srv_rot.type_, "ServingCertificateRotating")

    def test_conditions_ready_message_exact_format(self) -> None:
        facts = CertificateFacts(
            purpose=Purpose.PEER,
            issuer=IssuerId("pool-a"),
            fingerprint="0123456789abcdef",
            not_after=datetime(2026, 8, 1, 0, 0, 0, tzinfo=timezone.utc),
            trust_bundle=(IssuerId("pool-a"), IssuerId("pool-b")),
        )
        ready_fact, _ = facts.conditions()
        expected_msg = (
            "issuer pool-a; leaf 0123456789abcdef; expires"
            " 2026-08-01T00:00:00+00:00; trusting pool-a, pool-b"
        )
        self.assertEqual(ready_fact.message, expected_msg)

    def test_conditions_ready_message_omits_trust_bundle_part(self) -> None:
        facts = CertificateFacts(
            purpose=Purpose.PEER,
            issuer=IssuerId("pool-a"),
            fingerprint="0123456789abcdef",
            not_after=datetime(2026, 8, 1, 0, 0, 0, tzinfo=timezone.utc),
            trust_bundle=(),
        )
        ready_fact, _ = facts.conditions()
        expected_msg = (
            "issuer pool-a; leaf 0123456789abcdef; expires"
            " 2026-08-01T00:00:00+00:00"
        )
        self.assertEqual(ready_fact.message, expected_msg)

    def test_conditions_rotating_none_is_stable(self) -> None:
        facts = CertificateFacts(purpose=Purpose.SERVING, rotating=None)
        _, rot_fact = facts.conditions()
        self.assertEqual(rot_fact.status, ConditionStatus.FALSE)
        self.assertEqual(rot_fact.reason, "Stable")
        self.assertEqual(rot_fact.message, "")

    def test_conditions_rotating_all_reasons(self) -> None:
        expected_table = {
            IssueReason.BOOTSTRAP: ("Bootstrap", "no material has been issued yet"),
            IssueReason.RENEWAL: ("Renewal", "the renewal window has opened"),
            IssueReason.EXPIRED: ("Expired", "the projected leaf is past its notAfter"),
            IssueReason.IDENTITY_CHANGED: (
                "IdentityChanged",
                "the requested names no longer match the leaf",
            ),
            IssueReason.ISSUER_ROTATION: (
                "IssuerRotation",
                "the configured issuer changed",
            ),
        }
        for reason_enum, (exp_reason, exp_detail) in expected_table.items():
            facts = CertificateFacts(purpose=Purpose.SERVING, rotating=reason_enum)
            _, rot_fact = facts.conditions()
            self.assertEqual(rot_fact.status, ConditionStatus.TRUE)
            self.assertEqual(rot_fact.reason, exp_reason)
            expected_msg = f"issuing a new serving certificate: {exp_detail}"
            self.assertEqual(rot_fact.message, expected_msg)

    def test_conditions_rotating_does_not_make_ready_false(self) -> None:
        facts = CertificateFacts(
            purpose=Purpose.SERVING,
            issuer=IssuerId("pool-a"),
            not_after=datetime(2026, 8, 1, tzinfo=timezone.utc),
            rotating=IssueReason.RENEWAL,
        )
        ready_fact, rot_fact = facts.conditions()
        self.assertEqual(ready_fact.status, ConditionStatus.TRUE)
        self.assertEqual(rot_fact.status, ConditionStatus.TRUE)


if __name__ == "__main__":
    unittest.main()
