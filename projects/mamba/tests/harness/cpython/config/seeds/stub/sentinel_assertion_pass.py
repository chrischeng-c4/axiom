# sentinel_assertion_pass.py — #2539 unittest assertion sentinel (passing variant)
#
# Mamba-authored sentinel, not vendored from CPython. The `sentinel_*`
# filename prefix differentiates it from `test_*` seeds copied verbatim
# from CPython Lib/test/.
#
# Pair with `sentinel_assertion_fail.py` (failing variant). Together they
# answer one question: did the unittest runner actually execute assertions?
#
#   - Today: both run through `unittest.main()`, which is a stub on mamba,
#     so both classify as `Stub`. The runner never reached the asserts.
#   - When #2540 (AssertionPass outcome) + #2545 (minimal unittest dispatch)
#     land, this seed should flip to `AssertionPass` and the failing variant
#     must flip to `Fail`. If both stay `Stub` after dispatch is wired, the
#     dispatcher is bypassing assertions and the sentinel has done its job.

import unittest


class SentinelPass(unittest.TestCase):
    def test_arithmetic_holds(self):
        # A real assertion the runner must execute to declare success.
        self.assertEqual(1 + 1, 2)

    def test_truthiness(self):
        self.assertTrue(True)
        self.assertFalse(False)


if __name__ == "__main__":
    unittest.main()
