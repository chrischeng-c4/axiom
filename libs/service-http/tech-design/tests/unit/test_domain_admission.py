from __future__ import annotations

import sys
import unittest
from dataclasses import fields
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_http.domain.admission import (
    DEFAULT_MAX_KEYS,
    AdmissionPolicy,
    Event,
    Outcome,
    default_refill_window_ns,
    is_valid_policy,
    max_credits,
    observed_fields,
    policy_problem,
    request_cost,
)


class TestDomainAdmission(unittest.TestCase):
    def test_policy_problem_valid(self) -> None:
        policy = AdmissionPolicy(2, 10_000_000_000, 8)
        self.assertIsNone(policy_problem(policy))
        self.assertTrue(is_valid_policy(policy))

    def test_policy_problem_capacity_first(self) -> None:
        policy = AdmissionPolicy(0, 10_000_000_000, 0)
        self.assertEqual(policy_problem(policy), "capacity must be positive")

    def test_policy_problem_refill_window(self) -> None:
        policy = AdmissionPolicy(2, 0, 0)
        self.assertEqual(
            policy_problem(policy), "refill window must be positive"
        )

    def test_policy_problem_max_keys(self) -> None:
        policy = AdmissionPolicy(2, 10_000_000_000, 0)
        self.assertEqual(policy_problem(policy), "max keys must be positive")

    def test_max_credits_and_request_cost(self) -> None:
        policy = AdmissionPolicy(2, 10_000_000_000, 8)
        self.assertEqual(max_credits(policy), 20_000_000_000)
        self.assertEqual(request_cost(policy), 10_000_000_000)

    def test_default_refill_window_ns(self) -> None:
        self.assertEqual(default_refill_window_ns(), 60_000_000_000)

    def test_constants(self) -> None:
        self.assertEqual(DEFAULT_MAX_KEYS, 1024)

    def test_observed_fields_without_retry_after(self) -> None:
        event = Event("read", Outcome.ALLOW, None)
        obs = observed_fields(event)
        self.assertEqual(obs, {"class": "read", "outcome": "allow"})
        self.assertNotIn("retryAfterMs", obs)

    def test_observed_fields_with_retry_after(self) -> None:
        event = Event("write", Outcome.DENY, 5000)
        obs = observed_fields(event)
        self.assertEqual(
            obs, {"class": "write", "outcome": "deny", "retryAfterMs": 5000}
        )

    def test_event_field_names(self) -> None:
        names = tuple(f.name for f in fields(Event))
        self.assertEqual(
            names, ("route_class", "outcome", "retry_after_ms")
        )
