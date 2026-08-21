# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tix"
# dimension = "behavior"
# case = "test_tix__test_tix_available"
# subject = "cpython.test_tix.TestTix.test_tix_available"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tix.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: TestTix::test_tix_available (CPython 3.12 oracle)."""

import importlib
import unittest


try:
    module = importlib.import_module("test.test_tix")
except unittest.SkipTest as exc:
    assert str(exc) == "No module named '_tkinter'", str(exc)
else:
    case = module.TestTix("test_tix_available")
    result = unittest.TestResult()
    case.run(result)
    assert result.wasSuccessful(), result
    assert not result.failures, result.failures
    assert not result.errors, result.errors

print("TestTix::test_tix_available dependency boundary: ok")
