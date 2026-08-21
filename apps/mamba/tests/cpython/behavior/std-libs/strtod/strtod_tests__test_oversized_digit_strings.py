# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strtod"
# dimension = "behavior"
# case = "strtod_tests__test_oversized_digit_strings"
# subject = "cpython.test_strtod.StrtodTests.test_oversized_digit_strings"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_strtod.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_strtod.py::StrtodTests::test_oversized_digit_strings
"""Auto-ported test: StrtodTests::test_oversized_digit_strings (CPython 3.12 oracle)."""

import unittest
from test import support
from test.test_strtod import StrtodTests


case = StrtodTests("test_oversized_digit_strings")
result = unittest.TestResult()
old_real_max_memuse = support.real_max_memuse
try:
    support.real_max_memuse = 0
    case.run(result)
finally:
    support.real_max_memuse = old_real_max_memuse

assert result.wasSuccessful(), result
assert not result.failures, result.failures
assert not result.errors, result.errors
assert len(result.skipped) == 1, result.skipped
skipped_case, reason = result.skipped[0]
assert skipped_case is case, result.skipped
assert reason.startswith("not enough memory:"), reason
assert "minimum needed" in reason, reason

print("StrtodTests::test_oversized_digit_strings bigmem skip boundary: ok")
