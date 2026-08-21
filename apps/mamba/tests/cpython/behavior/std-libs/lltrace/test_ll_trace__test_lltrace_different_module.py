# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lltrace"
# dimension = "behavior"
# case = "test_ll_trace__test_lltrace_different_module"
# subject = "cpython.test_lltrace.TestLLTrace.test_lltrace_different_module"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_lltrace.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_lltrace.py::TestLLTrace::test_lltrace_different_module
"""Auto-ported test: TestLLTrace::test_lltrace_different_module (CPython 3.12 oracle)."""

import unittest
from test.test_lltrace import TestLLTrace


case = TestLLTrace("test_lltrace_different_module")
result = unittest.TestResult()
case.run(result)
assert result.wasSuccessful(), result
assert not result.failures, result.failures
assert not result.errors, result.errors
if result.skipped:
    assert len(result.skipped) == 1, result.skipped
    skipped_case, reason = result.skipped[0]
    assert skipped_case is case, result.skipped
    assert reason == "lltrace requires Py_DEBUG", reason

print("TestLLTrace::test_lltrace_different_module Py_DEBUG boundary: ok")
