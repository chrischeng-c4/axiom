# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "launcher"
# dimension = "behavior"
# case = "test_launcher_non_windows_module_skip"
# subject = "cpython.test_launcher.module_platform_gate"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_launcher.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""test_launcher: non-Windows imports raise the CPython module-level SkipTest."""
import importlib
import sys
import unittest

if sys.platform == "win32":
    raise SystemExit("test_launcher_non_windows_module_skip only covers non-Windows")

try:
    importlib.import_module("test.test_launcher")
except unittest.SkipTest as exc:
    assert str(exc) == "test only applies to Windows", str(exc)
else:
    raise AssertionError("test.test_launcher did not raise SkipTest on non-Windows")

print("test_launcher_non_windows_module_skip OK")
