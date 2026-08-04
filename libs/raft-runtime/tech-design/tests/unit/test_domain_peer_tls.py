from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from raft_runtime.domain.peer_tls import (
    HandshakeOutcome,
    PeerCertificate,
    TrustBundle,
    is_accepted,
    is_trusted,
    matches_identity,
    validity_problem,
    verify_peer,
)


class TestDomainPeerTls(unittest.TestCase):
    def setUp(self) -> None:
        self.trust = TrustBundle(issuers=("ca1.raft.svc", "ca2.raft.svc"))
        self.valid_cert = PeerCertificate(
            subject="raft-0.raft.svc",
            issuer="ca1.raft.svc",
            dns_names=("raft-0.raft.svc", "node0.raft.svc"),
            not_before_ms=1000,
            not_after_ms=2000,
        )

    def test_is_trusted_returns_true_when_issuer_in_bundle(self) -> None:
        self.assertTrue(is_trusted(self.valid_cert, self.trust))

    def test_is_trusted_returns_false_when_issuer_not_in_bundle(self) -> None:
        untrusted_cert = PeerCertificate(
            subject="raft-0.raft.svc",
            issuer="rogue-ca.svc",
            dns_names=("raft-0.raft.svc",),
            not_before_ms=1000,
            not_after_ms=2000,
        )
        self.assertFalse(is_trusted(untrusted_cert, self.trust))

    def test_validity_problem_before_valid_window(self) -> None:
        self.assertEqual(
            validity_problem(self.valid_cert, 999),
            HandshakeOutcome.NOT_YET_VALID,
        )

    def test_validity_problem_inclusive_not_before_boundary(self) -> None:
        self.assertIsNone(validity_problem(self.valid_cert, 1000))

    def test_validity_problem_valid_range(self) -> None:
        self.assertIsNone(validity_problem(self.valid_cert, 1999))

    def test_validity_problem_exclusive_not_after_boundary(self) -> None:
        self.assertEqual(
            validity_problem(self.valid_cert, 2000),
            HandshakeOutcome.EXPIRED,
        )

    def test_validity_problem_after_expiry(self) -> None:
        self.assertEqual(
            validity_problem(self.valid_cert, 2500),
            HandshakeOutcome.EXPIRED,
        )

    def test_matches_identity_exact_and_case_sensitive(self) -> None:
        self.assertTrue(matches_identity(self.valid_cert, "raft-0.raft.svc"))
        self.assertFalse(matches_identity(self.valid_cert, "RAFT-0.RAFT.SVC"))
        self.assertFalse(matches_identity(self.valid_cert, "*.raft.svc"))

    def test_matches_identity_absent(self) -> None:
        self.assertFalse(matches_identity(self.valid_cert, "raft-1.raft.svc"))

    def test_matches_identity_empty_dns_names(self) -> None:
        empty_cert = PeerCertificate(
            subject="raft-0.raft.svc",
            issuer="ca1.raft.svc",
            dns_names=(),
            not_before_ms=1000,
            not_after_ms=2000,
        )
        self.assertFalse(matches_identity(empty_cert, "raft-0.raft.svc"))

    def test_verify_peer_precedence_untrusted_first(self) -> None:
        bad_cert = PeerCertificate(
            subject="raft-0.raft.svc",
            issuer="unknown-ca",
            dns_names=("wrong-name",),
            not_before_ms=1000,
            not_after_ms=2000,
        )
        # untrusted AND expired AND misnamed
        outcome = verify_peer(
            bad_cert, self.trust, "expected-name", now_ms=3000
        )
        self.assertEqual(outcome, HandshakeOutcome.UNTRUSTED_ISSUER)

    def test_verify_peer_precedence_validity_before_identity(self) -> None:
        expired_misnamed_cert = PeerCertificate(
            subject="raft-0.raft.svc",
            issuer="ca1.raft.svc",
            dns_names=("wrong-name",),
            not_before_ms=1000,
            not_after_ms=2000,
        )
        # trusted, but expired AND misnamed
        outcome = verify_peer(
            expired_misnamed_cert, self.trust, "expected-name", now_ms=2000
        )
        self.assertEqual(outcome, HandshakeOutcome.EXPIRED)

    def test_verify_peer_precedence_hostname_mismatch(self) -> None:
        misnamed_cert = PeerCertificate(
            subject="raft-0.raft.svc",
            issuer="ca1.raft.svc",
            dns_names=("wrong-name",),
            not_before_ms=1000,
            not_after_ms=2000,
        )
        # trusted AND valid, but misnamed
        outcome = verify_peer(
            misnamed_cert, self.trust, "expected-name", now_ms=1500
        )
        self.assertEqual(outcome, HandshakeOutcome.HOSTNAME_MISMATCH)

    def test_verify_peer_accepts_when_all_valid(self) -> None:
        outcome = verify_peer(
            self.valid_cert, self.trust, "raft-0.raft.svc", now_ms=1500
        )
        self.assertEqual(outcome, HandshakeOutcome.ACCEPTED)

    def test_verify_peer_expected_identity_none_skips_identity_check(
        self,
    ) -> None:
        outcome = verify_peer(self.valid_cert, self.trust, None, now_ms=1500)
        self.assertEqual(outcome, HandshakeOutcome.ACCEPTED)

    def test_is_accepted_predicate(self) -> None:
        self.assertTrue(is_accepted(HandshakeOutcome.ACCEPTED))
        self.assertFalse(is_accepted(HandshakeOutcome.UNTRUSTED_ISSUER))
        self.assertFalse(is_accepted(HandshakeOutcome.NOT_YET_VALID))
        self.assertFalse(is_accepted(HandshakeOutcome.EXPIRED))
        self.assertFalse(is_accepted(HandshakeOutcome.HOSTNAME_MISMATCH))

    def test_enum_members_distinct_wire_values(self) -> None:
        outcomes = [
            HandshakeOutcome.ACCEPTED,
            HandshakeOutcome.UNTRUSTED_ISSUER,
            HandshakeOutcome.NOT_YET_VALID,
            HandshakeOutcome.EXPIRED,
            HandshakeOutcome.HOSTNAME_MISMATCH,
        ]
        values = [o.value for o in outcomes]
        self.assertEqual(len(set(values)), 5)
        self.assertEqual(
            values,
            [
                "accepted",
                "untrusted-issuer",
                "not-yet-valid",
                "expired",
                "hostname-mismatch",
            ],
        )


if __name__ == "__main__":
    unittest.main()
