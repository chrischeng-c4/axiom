# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "normalization_test__test_normalization"
# subject = "cpython.test_unicodedata.NormalizationTest.test_normalization"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_unicodedata.py::NormalizationTest::test_normalization
"""Auto-ported test: NormalizationTest::test_normalization (CPython 3.12 oracle)."""

import unittest
from test.test_unicodedata import NormalizationTest


case = NormalizationTest("test_normalization")
result = unittest.TestResult()
case.run(result)
assert result.wasSuccessful(), result
assert not result.failures, result.failures
assert not result.errors, result.errors
if result.skipped:
    assert len(result.skipped) == 1, result.skipped
    skipped_case, reason = result.skipped[0]
    assert skipped_case is case, result.skipped
    assert "resource" in reason or "download" in reason or "Permission error" in reason, reason

print("NormalizationTest::test_normalization resource boundary: ok")
