# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_hierarchy"
# dimension = "behavior"
# case = "attributes_test__test_errno_translation"
# subject = "cpython.test_exception_hierarchy.AttributesTest.test_errno_translation"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exception_hierarchy.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: AttributesTest::test_errno_translation (CPython 3.12 oracle)."""

import os
import unittest
from test.test_exception_hierarchy import AttributesTest


case = AttributesTest("test_errno_translation")
result = unittest.TestResult()
case.run(result)

if os.name == "nt":
    assert result.wasSuccessful(), result
    assert result.skipped == [], result.skipped
else:
    assert len(result.skipped) == 1, result.skipped
    assert result.skipped[0][0] is case
    assert result.skipped[0][1] == "Windows-specific test"
    assert result.failures == [], result.failures
    assert result.errors == [], result.errors

print("AttributesTest::test_errno_translation boundary: ok")
