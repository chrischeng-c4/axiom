from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_http.application.admission import (
    AdmissionLedger,
    decision_event,
)
from service_http.domain.admission import (
    AdmissionPolicy,
    Decision,
    Event,
    Outcome,
)


class TestApplicationAdmission(unittest.TestCase):
    def test_admit_at_worked_example_capacity_2(self) -> None:
        p2 = AdmissionPolicy(2, 10_000_000_000, 8)
        ledger = AdmissionLedger({"write": p2})

        d1 = ledger.admit_at("write", "f", 0)
        self.assertEqual(d1, Decision(Outcome.ALLOW, None))

        d2 = ledger.admit_at("write", "f", 0)
        self.assertEqual(d2, Decision(Outcome.ALLOW, None))

        d3 = ledger.admit_at("write", "f", 0)
        self.assertEqual(d3, Decision(Outcome.DENY, 5_000_000_000))

        d4 = ledger.admit_at("write", "f", 5_000_000_000)
        self.assertEqual(d4, Decision(Outcome.ALLOW, None))

    def test_admit_at_capacity_1(self) -> None:
        p1 = AdmissionPolicy(1, 10_000_000_000, 8)
        ledger = AdmissionLedger({"write": p1})

        d1 = ledger.admit_at("write", "f", 0)
        self.assertEqual(d1, Decision(Outcome.ALLOW, None))

        d2 = ledger.admit_at("write", "f", 0)
        self.assertEqual(d2, Decision(Outcome.DENY, 10_000_000_000))

    def test_bypass_leaves_sequence_and_keys_zero(self) -> None:
        p2 = AdmissionPolicy(2, 10_000_000_000, 8)
        ledger = AdmissionLedger({"write": p2})

        d = ledger.admit_at("unknown", "f", 0)
        self.assertEqual(d, Decision(Outcome.BYPASS, None))
        self.assertEqual(ledger.sequence(), 0)
        self.assertEqual(ledger.total_keys(), 0)

    def test_max_keys_eviction_same_class_only(self) -> None:
        p_write = AdmissionPolicy(2, 10_000_000_000, 2)
        p_read = AdmissionPolicy(2, 10_000_000_000, 5)
        ledger = AdmissionLedger({"write": p_write, "read": p_read})

        ledger.admit_at("read", "r1", 0)
        self.assertEqual(ledger.tracked_keys("read"), 1)

        ledger.admit_at("write", "w1", 1)
        ledger.admit_at("write", "w2", 2)
        self.assertEqual(ledger.tracked_keys("write"), 2)

        ledger.admit_at("write", "w3", 3)
        self.assertEqual(ledger.tracked_keys("write"), 2)
        self.assertEqual(ledger.tracked_keys("read"), 1)

    def test_clamped_refill_after_long_elapsed_time(self) -> None:
        p = AdmissionPolicy(2, 10_000_000_000, 8)
        ledger = AdmissionLedger({"write": p})

        ledger.admit_at("write", "f", 0)
        d_long = ledger.admit_at("write", "f", 10**15)
        self.assertEqual(d_long, Decision(Outcome.ALLOW, None))

    def test_decision_event_floors_retry(self) -> None:
        e_allow = decision_event("write", Decision(Outcome.ALLOW, None))
        self.assertEqual(e_allow, Event("write", Outcome.ALLOW, None))

        e_deny_5s = decision_event("write", Decision(Outcome.DENY, 5_000_000_000))
        self.assertEqual(e_deny_5s, Event("write", Outcome.DENY, 5000))

        e_deny_floor = decision_event("r", Decision(Outcome.DENY, 1_500_000))
        self.assertEqual(e_deny_floor.retry_after_ms, 1)
