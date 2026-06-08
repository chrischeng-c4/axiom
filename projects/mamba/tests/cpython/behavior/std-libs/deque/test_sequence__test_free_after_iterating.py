# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "deque"
# dimension = "behavior"
# case = "test_sequence__test_free_after_iterating"
# subject = "cpython.test_deque.TestSequence.test_free_after_iterating"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_deque.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: TestSequence::test_free_after_iterating (CPython 3.12 oracle)."""

import unittest
from test.test_deque import TestSequence


case = TestSequence("test_free_after_iterating")
result = unittest.TestResult()
case.run(result)

assert result.wasSuccessful(), result
assert len(result.skipped) == 1, result.skipped
assert result.skipped[0][0] is case
assert result.skipped[0][1] == "Exhausted deque iterator doesn't free a deque"

print("TestSequence::test_free_after_iterating skip boundary: ok")
