# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tix"
# dimension = "behavior"
# case = "test_tix_module_dependency_skip"
# subject = "cpython.test_tix.module_dependency_gate"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tix.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""test_tix: missing _tkinter raises CPython module-level SkipTest."""
import importlib
import unittest

try:
    importlib.import_module("test.test_tix")
except unittest.SkipTest as exc:
    assert str(exc) == "No module named '_tkinter'", str(exc)
else:
    raise AssertionError("test.test_tix imported despite missing _tkinter gate")

print("test_tix_module_dependency_skip OK")
