from __future__ import annotations

from dataclasses import FrozenInstanceError
import sys
import unittest

sys.path.insert(0, __file__.rsplit("/", 3)[0] + "/src")

from service_k8s.domain.condition import (
    Condition,
    ConditionFact,
    ConditionStatus,
    project,
)


class TestDomainCondition(unittest.TestCase):
    def test_status_tokens(self) -> None:
        self.assertEqual(ConditionStatus.TRUE.token, "True")
        self.assertEqual(ConditionStatus.FALSE.token, "False")
        self.assertEqual(ConditionStatus.UNKNOWN.token, "Unknown")

    def test_status_from_bool(self) -> None:
        self.assertEqual(ConditionStatus.from_bool(True), ConditionStatus.TRUE)
        self.assertEqual(ConditionStatus.from_bool(False), ConditionStatus.FALSE)
        self.assertNotIn(
            ConditionStatus.UNKNOWN,
            (ConditionStatus.from_bool(True), ConditionStatus.from_bool(False)),
        )

    def test_status_closed_enumeration(self) -> None:
        self.assertEqual(len(tuple(ConditionStatus)), 3)

    def test_unchanged_status_keeps_instant_and_refreshes_reason_and_message(
        self,
    ) -> None:
        prior = (
            Condition(
                type_="Ready",
                status="True",
                reason="OldReason",
                message="OldMessage",
                last_transition_time="2026-01-01T00:00:00Z",
                observed_generation=1,
            ),
        )
        facts = (
            ConditionFact(
                type_="Ready",
                status=ConditionStatus.TRUE,
                reason="NewReason",
                message="NewMessage",
            ),
        )
        res = project(
            prior, facts, observed_generation=7, now="2026-06-01T00:00:00Z"
        )
        self.assertEqual(len(res), 1)
        c = res[0]
        self.assertEqual(c.last_transition_time, "2026-01-01T00:00:00Z")
        self.assertEqual(c.reason, "NewReason")
        self.assertEqual(c.message, "NewMessage")
        self.assertEqual(c.observed_generation, 7)

    def test_status_flip_takes_injected_instant(self) -> None:
        prior = (
            Condition(
                type_="Ready",
                status="True",
                reason="ReadyReason",
                message="All green",
                last_transition_time="2026-01-01T00:00:00Z",
                observed_generation=1,
            ),
        )
        facts = (
            ConditionFact(
                type_="Ready",
                status=ConditionStatus.FALSE,
                reason="NotReadyReason",
                message="Degraded",
            ),
        )
        res = project(
            prior, facts, observed_generation=2, now="2026-06-01T00:00:00Z"
        )
        self.assertEqual(res[0].last_transition_time, "2026-06-01T00:00:00Z")
        self.assertEqual(res[0].status, "False")

    def test_first_sighting_takes_injected_instant(self) -> None:
        facts = (
            ConditionFact(
                type_="Ready",
                status=ConditionStatus.TRUE,
                reason="InitialReady",
                message="First check",
            ),
        )
        res = project(
            (), facts, observed_generation=1, now="2026-01-01T00:00:00Z"
        )
        self.assertEqual(res[0].last_transition_time, "2026-01-01T00:00:00Z")

    def test_resurrection_is_new_condition(self) -> None:
        fact_ready = ConditionFact("Ready", ConditionStatus.TRUE, "R1", "M1")
        res1 = project((), (fact_ready,), 1, "2026-01-01T00:00:00Z")
        self.assertEqual(res1[0].last_transition_time, "2026-01-01T00:00:00Z")

        res2 = project(res1, (), 2, "2026-02-01T00:00:00Z")
        self.assertEqual(len(res2), 0)

        res3 = project(res2, (fact_ready,), 3, "2026-03-01T00:00:00Z")
        self.assertEqual(res3[0].last_transition_time, "2026-03-01T00:00:00Z")

    def test_dropping_absent_facts(self) -> None:
        prior = (
            Condition("Ready", "True", "R", "M", "T1", 1),
            Condition("Rotating", "True", "R", "M", "T1", 1),
        )
        facts = (ConditionFact("Ready", ConditionStatus.TRUE, "R", "M"),)
        res = project(prior, facts, 2, "T2")
        self.assertEqual(len(res), 1)
        self.assertEqual(res[0].type_, "Ready")

    def test_preserves_fact_order(self) -> None:
        facts = (
            ConditionFact("Ready", ConditionStatus.TRUE, "R1"),
            ConditionFact("Rotating", ConditionStatus.FALSE, "R2"),
            ConditionFact("Available", ConditionStatus.TRUE, "R3"),
        )
        res = project((), facts, 1, "T1")
        types = tuple(c.type_ for c in res)
        self.assertEqual(types, ("Ready", "Rotating", "Available"))

    def test_determinism(self) -> None:
        prior = (Condition("Ready", "True", "R", "M", "T1", 1),)
        facts = (ConditionFact("Ready", ConditionStatus.TRUE, "R2", "M2"),)
        res1 = project(prior, facts, 2, "T2")
        res2 = project(prior, facts, 2, "T2")
        self.assertEqual(res1, res2)

    def test_observed_generation_refreshed_on_carried_instant(self) -> None:
        prior = (Condition("Ready", "True", "R1", "M1", "T1", 1),)
        facts = (ConditionFact("Ready", ConditionStatus.TRUE, "R1", "M1"),)
        res = project(prior, facts, 7, "T2")
        self.assertEqual(res[0].last_transition_time, "T1")
        self.assertEqual(res[0].observed_generation, 7)

    def test_to_json_full_keys_order_and_content(self) -> None:
        c = Condition("Ready", "True", "Reason", "Msg", "T1", 5)
        d = c.to_json()
        self.assertEqual(
            list(d.keys()),
            [
                "type",
                "status",
                "reason",
                "message",
                "lastTransitionTime",
                "observedGeneration",
            ],
        )
        expected = {
            "type": "Ready",
            "status": "True",
            "reason": "Reason",
            "message": "Msg",
            "lastTransitionTime": "T1",
            "observedGeneration": 5,
        }
        self.assertEqual(d, expected)

    def test_to_json_omits_observed_generation_when_none(self) -> None:
        c = Condition("Ready", "True", "Reason", "Msg", "T1", None)
        d = c.to_json()
        self.assertNotIn("observedGeneration", d)

    def test_to_json_keeps_empty_message(self) -> None:
        c = Condition("Ready", "True", "Reason", "", "T1", 1)
        d = c.to_json()
        self.assertEqual(d["message"], "")

    def test_to_json_key_is_type_not_type_underscore(self) -> None:
        c = Condition("Ready", "True", "Reason", "Msg", "T1", 1)
        d = c.to_json()
        self.assertIn("type", d)
        self.assertNotIn("type_", d)

    def test_project_returns_tuple_and_dataclasses_frozen(self) -> None:
        facts = (ConditionFact("Ready", ConditionStatus.TRUE, "R"),)
        res = project((), facts, 1, "T1")
        self.assertIsInstance(res, tuple)
        with self.assertRaises(FrozenInstanceError):
            res[0].reason = "Other"  # type: ignore[misc]

    def test_prior_same_type_different_status_does_not_donate_instant(
        self,
    ) -> None:
        prior = (Condition("Ready", "False", "OldReason", "OldMsg", "T1", 1),)
        facts = (
            ConditionFact(
                "Ready", ConditionStatus.TRUE, "NewReason", "NewMsg"
            ),
        )
        res = project(prior, facts, 2, "T2")
        self.assertEqual(res[0].last_transition_time, "T2")


if __name__ == "__main__":
    unittest.main()
