# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pty"
# dimension = "behavior"
# case = "pty_test__test_spawn_doesnt_hang"
# subject = "cpython.test_pty.PtyTest.test_spawn_doesnt_hang"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pty.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pty.py::PtyTest::test_spawn_doesnt_hang
"""Auto-ported test: PtyTest::test_spawn_doesnt_hang (CPython 3.12 oracle)."""

import importlib
import unittest


try:
    module = importlib.import_module("test.test_pty")
except unittest.SkipTest as exc:
    assert str(exc), "expected pty dependency skip reason"
else:
    case = module.PtyTest("test_spawn_doesnt_hang")
    result = unittest.TestResult()
    case.run(result)
    assert result.wasSuccessful(), result
    assert not result.failures, result.failures
    assert not result.errors, result.errors

print("PtyTest::test_spawn_doesnt_hang child pty boundary: ok")
