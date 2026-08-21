# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "poll"
# dimension = "behavior"
# case = "poll_tests__test_poll2"
# subject = "cpython.test_poll.PollTests.test_poll2"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_poll.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_poll.py::PollTests::test_poll2
"""Auto-ported test: PollTests::test_poll2 (CPython 3.12 oracle)."""

import importlib
import unittest


try:
    module = importlib.import_module("test.test_poll")
except unittest.SkipTest as exc:
    assert str(exc), "expected select.poll availability skip reason"
else:
    case = module.PollTests("test_poll2")
    result = unittest.TestResult()
    case.run(result)
    assert result.wasSuccessful(), result
    assert not result.failures, result.failures
    assert not result.errors, result.errors

print("PollTests::test_poll2 subprocess pipe boundary: ok")
