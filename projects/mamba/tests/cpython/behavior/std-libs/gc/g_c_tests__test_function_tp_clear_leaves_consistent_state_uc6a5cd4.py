# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "behavior"
# case = "g_c_tests__test_function_tp_clear_leaves_consistent_state_uc6a5cd4"
# subject = "cpython.test_gc.GCTests.test_function_tp_clear_leaves_consistent_state"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_gc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_gc
_suite = unittest.defaultTestLoader.loadTestsFromName("GCTests.test_function_tp_clear_leaves_consistent_state", test_gc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython GCTests.test_function_tp_clear_leaves_consistent_state did not pass"
print("GCTests::test_function_tp_clear_leaves_consistent_state: ok")
