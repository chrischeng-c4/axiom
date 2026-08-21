# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "behavior"
# case = "g_c_toggling_tests__test_bug1055820d_ucf3296d"
# subject = "cpython.test_gc.GCTogglingTests.test_bug1055820d"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_gc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_gc
_suite = unittest.defaultTestLoader.loadTestsFromName("GCTogglingTests.test_bug1055820d", test_gc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython GCTogglingTests.test_bug1055820d did not pass"
print("GCTogglingTests::test_bug1055820d: ok")
