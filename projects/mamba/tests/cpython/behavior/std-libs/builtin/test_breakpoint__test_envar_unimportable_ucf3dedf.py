# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtin"
# dimension = "behavior"
# case = "test_breakpoint__test_envar_unimportable_ucf3dedf"
# subject = "cpython.test_builtin.TestBreakpoint.test_envar_unimportable"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_builtin
_suite = unittest.defaultTestLoader.loadTestsFromName("TestBreakpoint.test_envar_unimportable", test_builtin)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestBreakpoint.test_envar_unimportable did not pass"
print("TestBreakpoint::test_envar_unimportable: ok")
