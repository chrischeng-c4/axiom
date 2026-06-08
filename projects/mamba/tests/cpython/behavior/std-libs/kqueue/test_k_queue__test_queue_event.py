# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "kqueue"
# dimension = "behavior"
# case = "test_k_queue__test_queue_event"
# subject = "cpython.test_kqueue.TestKQueue.test_queue_event"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_kqueue.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_kqueue.py::TestKQueue::test_queue_event
"""Auto-ported test: TestKQueue::test_queue_event (CPython 3.12 oracle)."""

import importlib
import unittest


try:
    module = importlib.import_module("test.test_kqueue")
except unittest.SkipTest as exc:
    assert str(exc) == "test works only on BSD", str(exc)
else:
    case = module.TestKQueue("test_queue_event")
    result = unittest.TestResult()
    case.run(result)
    assert result.wasSuccessful(), result
    assert not result.failures, result.failures
    assert not result.errors, result.errors

print("TestKQueue::test_queue_event local socket queue boundary: ok")
