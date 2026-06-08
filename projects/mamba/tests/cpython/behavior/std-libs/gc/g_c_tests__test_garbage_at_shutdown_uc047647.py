# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "behavior"
# case = "g_c_tests__test_garbage_at_shutdown_uc047647"
# subject = "cpython.test_gc.GCTests.test_garbage_at_shutdown"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_gc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_gc
_suite = unittest.defaultTestLoader.loadTestsFromName("GCTests.test_garbage_at_shutdown", test_gc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython GCTests.test_garbage_at_shutdown did not pass"
print("GCTests::test_garbage_at_shutdown: ok")
