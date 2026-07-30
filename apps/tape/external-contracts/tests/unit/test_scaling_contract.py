"""Unit tests for the throughput-scaling contract's threshold logic.

The interesting cases here are the degenerate ones. A ratio test is easy to
write in a way that passes on nonsense input -- a zero baseline divides by
zero, and a baseline that somehow came back negative produces a ratio with the
wrong sign that can sail past a `>=` check. Both are asserted to go red, along
with the two cases a bare ratio cannot see at all: a ratio won by starving the
single writer rather than by amortising the barrier, and a ratio won by not
being durable in the first place.

Check ordering is itself under test. The ceiling has to be evaluated before the
ratio, because a barrier-eliding build satisfies the ratio handsomely; a test
that only feeds it numbers which fail the ratio too would pass whether the
ordering is right or wrong.

Stdlib `unittest`, not pytest: see the note in `test_recovery_contract.py`.
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


scaling = _load("ec-3052-durable-append-scaling.py")
ContractFailure = scaling.ContractFailure

# The barrier actually measured on the authoring host: F_FULLFSYNC, 5.07 ms.
# The bounds derived from it are a floor of 0.25 * 197.0 = 49.25 ops/s and a
# ceiling of 3.0 * connections * 197.0 -- 591 at 1 connection, 9456 at 16.
BARRIER = {
    "primitive": "F_FULLFSYNC",
    "samples": 40,
    "median_ms": 5.066,
    "barrier_hz": 197.0,
}

# Every 1-connection rate the unchanged whole-file path has been observed at,
# each with the barrier measured beside it in the same run. The floor claims to
# be a no-regression bound, so all of these must be green on it. At 0.4 that
# claim was false and, worse, unstable: fractions 0.310 / 0.327 / 0.393 / 0.397
# / 0.437 straddle it, so the same binary was red four times and green once.
INCUMBENT_OBSERVATIONS = [
    (61.0, 83.5, 197.0),
    (73.3, 70.2, 184.6),
    (65.8, 68.7, 167.3),
    (85.5, 93.8, 195.7),
    (63.0, 87.6, 192.6),
]


def level(connections: int, ops: float) -> dict:
    return {
        "connections": connections,
        "events": 600,
        "elapsed_s": round(600 / ops, 3) if ops else 0.0,
        "ops_per_s": ops,
        "survived_restart": 620,
    }


class CheckScalingTest(unittest.TestCase):
    def test_accepts_throughput_that_rises_with_concurrency(self):
        facts = scaling.check_scaling(level(1, 177.0), level(16, 2100.0), BARRIER)

        self.assertAlmostEqual(facts["ratio"], 11.86, places=2)
        self.assertEqual(facts["required_ratio"], scaling.REQUIRED_RATIO)
        self.assertAlmostEqual(facts["lone_writer_barrier_fraction"], 0.898, places=3)

    def test_rejects_the_flat_line_this_contract_exists_for(self):
        """61 -> 83.5 ops/s across 1 and 16 connections: the measured status quo."""
        with self.assertRaisesRegex(ContractFailure, "did not rise with concurrency"):
            scaling.check_scaling(level(1, 61.0), level(16, 83.5), BARRIER)

    def test_rejects_an_improvement_that_falls_short_of_the_threshold(self):
        """WAL without group commit is ~2x. Real, and not what was asked for."""
        with self.assertRaisesRegex(ContractFailure, "did not rise with concurrency"):
            scaling.check_scaling(level(1, 89.0), level(16, 178.0), BARRIER)

    def test_rejects_a_ratio_bought_by_starving_the_lone_writer(self):
        """Fixed linger, no early flush: the defect a bare ratio cannot see.

        20 ops/s at 1 connection and 320 at 16 is a ratio of 16 -- four times
        the requirement, and green on the ratio alone -- while single-writer
        durable append has regressed threefold below today's 61 ops/s.
        """
        with self.assertRaisesRegex(ContractFailure, "starving the lone writer"):
            scaling.check_scaling(level(1, 20.0), level(16, 320.0), BARRIER)

    def test_rejects_a_build_that_removed_the_barrier_instead_of_sharing_it(self):
        """The other way to win a ratio: stop being durable.

        2000 ops/s at 1 connection is ten records per F_FULLFSYNC with nobody
        to share one with -- arithmetically impossible for an append that waits
        for its barrier. Note this input is green on the ratio (10.0) and green
        on the floor, so nothing else in this contract would object, and the
        SIGKILL recovery check in `measure()` would not either: unsynced writes
        survive a process kill.

        These are not invented numbers. A real tape started without a store
        never issues a barrier and measures 4786.8 ops/s at 1 connection
        against a 594.3 ceiling -- 8x over. 2000 is the conservative version of
        the same input.
        """
        with self.assertRaisesRegex(ContractFailure, "did not wait for a barrier"):
            scaling.check_scaling(level(1, 2000.0), level(16, 20000.0), BARRIER)

    def test_the_ceiling_is_evaluated_before_the_ratio(self):
        """Ordering, asserted directly rather than left to luck.

        These numbers fail both checks. If the ratio were evaluated first the
        report would blame concurrency, and the far more serious finding -- that
        acknowledgements are not waiting for a barrier -- would be buried under
        a performance complaint that is beside the point.
        """
        with self.assertRaisesRegex(ContractFailure, "did not wait for a barrier"):
            scaling.check_scaling(level(1, 5000.0), level(16, 5200.0), BARRIER)

    def test_the_ceiling_widens_with_connection_count(self):
        """At N connections N records may legally share one barrier.

        5000 ops/s is over the ceiling at 1 connection and comfortably under it
        at 16. A ceiling that did not scale with N would reject exactly the
        group commit this contract is asking for.
        """
        facts = scaling.check_scaling(level(1, 400.0), level(16, 5000.0), BARRIER)

        self.assertEqual(facts["ceiling_ops_per_s"]["1"], 591.0)
        self.assertEqual(facts["ceiling_ops_per_s"]["16"], 9456.0)

    def test_rejects_a_scaled_level_over_its_own_ceiling(self):
        """The scaled level is bounded too, though it does not bind today.

        A real zero-durability tape measures 4785.9 ops/s at 16 connections
        against a 9508.8 ceiling -- silent, because that build is HTTP bound at
        every concurrency. The check is kept for the build this host cannot
        exhibit: one that elides the barrier *and* scales, whose baseline would
        look plausible while its scaled level ran past the bound.
        """
        with self.assertRaisesRegex(ContractFailure, "did not wait for a barrier"):
            scaling.check_scaling(level(1, 400.0), level(16, 12000.0), BARRIER)

    def test_todays_incumbent_clears_the_floor_it_is_supposed_to_calibrate(self):
        """The floor claims to be a no-regression bound. Hold it to that.

        Each of these is a real measurement of the unchanged whole-file path
        with its own in-run barrier. Every one must be red on the ratio -- that
        is the point of the contract -- and green on the floor, or the floor is
        not measuring what it says. The assertion reads the fraction out of the
        failure's facts rather than relying on which check fires first, because
        at 0.4 three of the four were red on the floor and the ratio check,
        firing earlier, hid it.
        """
        for base_rate, scaled_rate, barrier_hz in INCUMBENT_OBSERVATIONS:
            with self.subTest(baseline=base_rate, barrier_hz=barrier_hz):
                barrier = dict(BARRIER, barrier_hz=barrier_hz)

                with self.assertRaises(ContractFailure) as caught:
                    scaling.check_scaling(
                        level(1, base_rate), level(16, scaled_rate), barrier
                    )

                self.assertIn("did not rise with concurrency", str(caught.exception))
                self.assertGreaterEqual(
                    caught.exception.facts["lone_writer_barrier_fraction"],
                    scaling.LONE_WRITER_BARRIER_FRACTION,
                )
                self.assertLessEqual(
                    base_rate, caught.exception.facts["ceiling_ops_per_s"]["1"]
                )

    def test_accepts_exactly_the_threshold(self):
        facts = scaling.check_scaling(level(1, 100.0), level(16, 400.0), BARRIER)

        self.assertEqual(facts["ratio"], scaling.REQUIRED_RATIO)

    def test_accepts_a_lone_writer_at_exactly_the_barrier_floor(self):
        floor = scaling.LONE_WRITER_BARRIER_FRACTION * BARRIER["barrier_hz"]

        facts = scaling.check_scaling(level(1, floor), level(16, floor * 5), BARRIER)

        self.assertEqual(facts["lone_writer_floor_ops_per_s"], round(floor, 1))

    def test_the_floor_scales_with_the_measured_barrier_not_a_constant(self):
        """A slower disk must not make the floor unreachable.

        The same 40 ops/s baseline is a pass on a 100/s barrier and a failure on
        a 197/s one. That is the whole point of calibrating in-run: the floor is
        a statement about how many barriers an append costs, not about ops/s.
        """
        slow = dict(BARRIER, barrier_hz=100.0, median_ms=10.0)

        facts = scaling.check_scaling(level(1, 40.0), level(16, 400.0), slow)
        self.assertEqual(facts["lone_writer_floor_ops_per_s"], 25.0)

        with self.assertRaisesRegex(ContractFailure, "starving the lone writer"):
            scaling.check_scaling(level(1, 40.0), level(16, 400.0), BARRIER)

    def test_rejects_a_zero_baseline_instead_of_dividing_by_it(self):
        with self.assertRaisesRegex(ContractFailure, "not positive"):
            scaling.check_scaling(level(1, 0.0), level(16, 2100.0), BARRIER)

    def test_rejects_a_zero_scaled_rate(self):
        with self.assertRaisesRegex(ContractFailure, "not positive"):
            scaling.check_scaling(level(1, 89.0), level(16, 0.0), BARRIER)

    def test_rejects_a_negative_rate_rather_than_producing_a_passing_ratio(self):
        with self.assertRaisesRegex(ContractFailure, "not positive"):
            scaling.check_scaling(level(1, -100.0), level(16, -2100.0), BARRIER)

    def test_rejects_a_non_positive_barrier_measurement(self):
        """An unmeasurable barrier must not silently disable the floor."""
        broken = dict(BARRIER, barrier_hz=0.0)

        with self.assertRaisesRegex(ContractFailure, "barrier rate was not positive"):
            scaling.check_scaling(level(1, 177.0), level(16, 2100.0), broken)


if __name__ == "__main__":
    unittest.main()
