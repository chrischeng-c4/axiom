# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ucn"
# dimension = "behavior"
# case = "unicode_names_test__test_issue16335"
# subject = "cpython.test_ucn.UnicodeNamesTest.test_issue16335"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ucn.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: UnicodeNamesTest::test_issue16335 (CPython 3.12 oracle)."""

import unittest
from test import support
from test.test_ucn import UnicodeNamesTest


original_real_max_memuse = support.real_max_memuse
support.real_max_memuse = 0
try:
    case = UnicodeNamesTest("test_issue16335")
    result = unittest.TestResult()
    case.run(result)
finally:
    support.real_max_memuse = original_real_max_memuse

assert result.wasSuccessful(), result
assert len(result.skipped) == 1, result.skipped
assert result.skipped[0][0] is case
assert result.skipped[0][1].startswith("not enough memory:"), result.skipped
assert "minimum needed" in result.skipped[0][1]

print("UnicodeNamesTest::test_issue16335 bigmem boundary: ok")
