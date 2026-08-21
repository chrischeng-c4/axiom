# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "crashers"
# dimension = "security"
# case = "crasher_test__test_crashers_crash_is_skipped"
# subject = "cpython.test_crashers.CrasherTest.test_crashers_crash"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_crashers.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""CrasherTest.test_crashers_crash: CPython keeps fragile crashers skipped."""
import io
import unittest

import test.test_crashers as test_crashers

suite = unittest.defaultTestLoader.loadTestsFromTestCase(test_crashers.CrasherTest)
result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(suite)

assert result.testsRun == 1, result.testsRun
assert len(result.skipped) == 1, result.skipped
assert result.skipped[0][1] == "these tests are too fragile", result.skipped
assert not result.failures, result.failures
assert not result.errors, result.errors

print("CrasherTest::test_crashers_crash skipped: ok")
