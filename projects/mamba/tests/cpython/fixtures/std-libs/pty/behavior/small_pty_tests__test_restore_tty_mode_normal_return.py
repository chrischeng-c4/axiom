# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pty"
# dimension = "behavior"
# case = "small_pty_tests__test_restore_tty_mode_normal_return"
# subject = "cpython.test_pty.SmallPtyTests.test__restore_tty_mode_normal_return"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pty.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pty.py::SmallPtyTests::test__restore_tty_mode_normal_return
"""Auto-ported test: SmallPtyTests::test__restore_tty_mode_normal_return (CPython 3.12 oracle)."""

import importlib
import unittest


try:
    module = importlib.import_module("test.test_pty")
except unittest.SkipTest as exc:
    assert str(exc), "expected pty dependency skip reason"
else:
    case = module.SmallPtyTests("test__restore_tty_mode_normal_return")
    result = unittest.TestResult()
    case.run(result)
    assert result.wasSuccessful(), result
    assert not result.failures, result.failures
    assert not result.errors, result.errors

print("SmallPtyTests::test__restore_tty_mode_normal_return mocked tty boundary: ok")
