# sentinel_assertion_fail.py — #2539 unittest assertion sentinel (failing variant)
#
# Mamba-authored sentinel, not vendored from CPython. Pair with
# `sentinel_assertion_pass.py` (passing variant).
#
# The failing variant exists so a future change cannot "fix" the sentinel
# pair by no-op'ing the runner: if assertions are bypassed, both files
# classify as `Stub`; if assertions are dispatched correctly, this file
# must classify as `Fail` (the deliberate `1 == 2` mismatch should raise).
#
# Today on mamba: `unittest.main()` is a stub, so this stays `Stub`. The
# moment #2540 (AssertionPass outcome) + #2545 (minimal unittest dispatch)
# land, the baseline for this file must flip to `Fail` in the same commit.

import unittest


class SentinelFail(unittest.TestCase):
    def test_deliberate_mismatch(self):
        # This MUST fail under a working runner. If the runner reports
        # success, the sentinel pair has caught a no-op dispatcher.
        self.assertEqual(1, 2, "sentinel: working runner should not pass this")


if __name__ == "__main__":
    unittest.main()
