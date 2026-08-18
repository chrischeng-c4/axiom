"""Unit tests for the SIGKILL durability contract's assertions.

These do not start a server. They feed `check_recovery` and
`check_kill_preconditions` synthetic observations, one per failure mode each
claims to catch, and assert it goes red. An oracle that has only ever been
observed green is not known to work -- it is only known to be quiet, and those
two states look identical right up until the run where it matters.

`check_kill_preconditions` is here specifically because the guard it replaced
was dead code that no test could have caught: it lived inside `verify()`,
which nothing exercised without a real server, and it read a flag the kill
thread set unconditionally.

Its refusal branch had the same disease one layer up. It aborted the run on any
non-2xx, which made `refused` empty by construction, which made
`check_recovery`'s `recovered ∩ refused = ∅` assertion unreachable in every real
run -- a guard whose own effect was to guarantee that the thing it guarded could
never fire. It now grades vacuity in both directions, and both directions are
asserted below.

Written against stdlib `unittest` rather than pytest on purpose: `aw ec check`
runs this directory with `unittest discover` in a bare `uv` environment, so a
third-party test dependency here would make the contract unrunnable by the very
gate that is supposed to run it.
"""

from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SRC = Path(__file__).resolve().parents[2] / "src"


def _load(name: str):
    spec = importlib.util.spec_from_file_location(
        name.replace("-", "_").removesuffix(".py"), SRC / name
    )
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


durability = _load("ec-3052-durability-under-sigkill.py")
ContractFailure = durability.ContractFailure
THRESHOLD = durability.KILL_AFTER_ACKS
PROBES = durability.REFUSAL_PROBES


def event(key: str, offset: int, n: int | None = None) -> dict:
    return {
        "key": key,
        "offset": offset,
        "payload": {"n": n if n is not None else int(key[1:])},
    }


class CheckRecoveryTest(unittest.TestCase):
    def test_accepts_a_clean_recovery(self):
        sent = {"k0", "k1", "k2", "k3"}
        acked = {"k0": 0, "k1": 1, "k2": 2}
        events = [event("k0", 0), event("k1", 1), event("k2", 2)]

        facts = durability.check_recovery(sent, acked, set(), events)

        self.assertEqual(facts["acknowledged"], 3)
        self.assertEqual(facts["recovered"], 3)
        self.assertEqual(facts["refused"], 0)
        self.assertEqual(facts["offset_range"], [0, 2])

    def test_accepts_recovery_beyond_what_was_acknowledged(self):
        """A record made durable but killed before its response is legal.

        This is the case #3052's AC4 wording would forbid. It is what a correct
        implementation does, so the contract must count it rather than fail it.
        """
        sent = {"k0", "k1", "k2"}
        acked = {"k0": 0, "k1": 1}
        events = [event("k0", 0), event("k1", 1), event("k2", 2)]

        facts = durability.check_recovery(sent, acked, set(), events)

        self.assertEqual(facts["recovered_beyond_acknowledged"], 1)

    def test_rejects_a_lost_acknowledged_append(self):
        sent = {"k0", "k1", "k2"}
        acked = {"k0": 0, "k1": 1, "k2": 2}
        events = [event("k0", 0), event("k1", 1)]

        with self.assertRaisesRegex(ContractFailure, "did not survive SIGKILL"):
            durability.check_recovery(sent, acked, set(), events)

    def test_rejects_a_refused_write_that_came_back(self):
        """The achievable half of AC4's second clause.

        `k1` was answered non-2xx -- a 507 from degraded read-only, or a 500
        from a failed persist. It is not an unknown outcome; the server said
        no. Recovering it anyway is the product contradicting itself, and
        `recovered ⊆ sent` alone would wave it through, because `sent` contains
        every key the client ever tried.
        """
        sent = {"k0", "k1"}
        acked = {"k0": 0}
        events = [event("k0", 0), event("k1", 1)]

        with self.assertRaisesRegex(ContractFailure, "explicitly refused"):
            durability.check_recovery(sent, acked, {"k1"}, events)

    def test_rejects_a_key_replayed_more_than_once(self):
        """Double-apply on recovery: the canonical WAL defect.

        The server assigns offsets fresh on each apply, so the duplicates come
        back distinct and contiguous and every offset check passes. Indexing
        the recovered events by key -- which the rest of this function does --
        collapses them silently. Only an explicit count catches it.
        """
        sent = {"k0", "k1", "k2"}
        acked = {"k0": 0, "k1": 1, "k2": 2}
        events = [
            event("k0", 0),
            event("k1", 1),
            event("k2", 2),
            event("k0", 3),
            event("k1", 4),
            event("k2", 5),
        ]

        with self.assertRaisesRegex(ContractFailure, "applied more than once"):
            durability.check_recovery(sent, acked, set(), events)

    def test_rejects_a_phantom_record(self):
        sent = {"k0", "k1"}
        acked = {"k0": 0}
        events = [event("k0", 0), event("k1", 1), event("k9", 2, n=9)]

        with self.assertRaisesRegex(ContractFailure, "never submitted"):
            durability.check_recovery(sent, acked, set(), events)

    def test_rejects_duplicate_offsets(self):
        sent = {"k0", "k1"}
        acked = {"k0": 0, "k1": 1}
        events = [event("k0", 0), event("k1", 0)]

        with self.assertRaisesRegex(ContractFailure, "duplicate offsets"):
            durability.check_recovery(sent, acked, set(), events)

    def test_rejects_a_hole_in_the_offset_range(self):
        sent = {"k0", "k1", "k2"}
        acked = {"k0": 0, "k2": 2}
        events = [event("k0", 0), event("k2", 5)]

        with self.assertRaisesRegex(ContractFailure, "not contiguous"):
            durability.check_recovery(sent, acked, set(), events)

    def test_rejects_a_payload_that_does_not_match_the_acknowledgement(self):
        sent = {"k0", "k1"}
        acked = {"k0": 0, "k1": 1}
        events = [event("k0", 0), event("k1", 1, n=99)]

        with self.assertRaisesRegex(ContractFailure, "do not match"):
            durability.check_recovery(sent, acked, set(), events)

    def test_an_empty_recovery_still_fails_when_something_was_acknowledged(self):
        """The degenerate case: total data loss must not read as a vacuous pass."""
        with self.assertRaisesRegex(ContractFailure, "did not survive SIGKILL"):
            durability.check_recovery({"k0"}, {"k0": 0}, set(), [])


class CheckKillPreconditionsTest(unittest.TestCase):
    def test_accepts_a_run_that_reached_the_threshold_cleanly(self):
        self.assertIsNone(
            durability.check_kill_preconditions(True, THRESHOLD, PROBES, 0)
        )

    def test_rejects_a_kill_that_fired_on_the_timeout(self):
        """The failure the previous inline guard could never report.

        One append acknowledged out of 400, the kill fired because the deadline
        expired rather than because anything was durable. `check_recovery` would
        happily grade this green: one acked key, one recovered key, no phantom.
        """
        with self.assertRaisesRegex(ContractFailure, "fired on a timeout"):
            durability.check_kill_preconditions(False, 1, PROBES, 0)

    def test_rejects_an_acknowledgement_count_below_the_threshold(self):
        """Unreachable from `verify()`, asserted anyway.

        The flag and the count come from the same counter there, so a `True`
        flag implies the count. This pins the guard against the refactor that
        stops making that true.
        """
        with self.assertRaisesRegex(ContractFailure, "below the"):
            durability.check_kill_preconditions(True, THRESHOLD - 1, PROBES, 0)

    def test_rejects_a_run_where_the_server_refused_a_well_formed_write(self):
        """A healthy server answering non-2xx is a defect, not a measurement."""
        with self.assertRaisesRegex(ContractFailure, "well-formed appends"):
            durability.check_kill_preconditions(True, THRESHOLD, PROBES, 7)

    def test_rejects_a_run_where_the_refusal_probes_were_not_refused(self):
        """The vacuity this guard exists for, in the opposite direction.

        If the malformed appends came back 2xx, `refused` is empty, and
        `check_recovery`'s `recovered ∩ refused = ∅` assertion has no members to
        reject -- it passes on any input whatsoever. That is precisely the state
        this contract shipped in for two revisions while reporting green, so it
        is now a graded precondition rather than an assumption.
        """
        with self.assertRaisesRegex(ContractFailure, "would be vacuous"):
            durability.check_kill_preconditions(True, THRESHOLD, 0, 0)

    def test_rejects_a_partially_refused_probe_batch(self):
        """All of them, not most. The probes are serial against a healthy
        server before any load, so a short count means something changed about
        the refusal path -- which is the thing being relied on."""
        with self.assertRaisesRegex(ContractFailure, "would be vacuous"):
            durability.check_kill_preconditions(True, THRESHOLD, PROBES - 1, 0)


class MalformedBodyTest(unittest.TestCase):
    """The probe body has two jobs and they pull in opposite directions."""

    def test_does_not_parse(self):
        import json

        with self.assertRaises(json.JSONDecodeError):
            json.loads(durability.malformed_body("m000001"))

    def test_carries_the_key_in_its_bytes(self):
        """Otherwise a recovered probe key would prove nothing.

        The assertion the probe feeds is "the server did not store a request it
        refused". If the key were not in the bytes, the server could not have
        stored it under that key even in principle, and the assertion would be
        unfalsifiable rather than merely unfired.
        """
        self.assertIn(b"m000001", durability.malformed_body("m000001"))


if __name__ == "__main__":
    unittest.main()
