# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "behavior"
# case = "g_c_callback_tests__test_collect_garbage_uc975768"
# subject = "cpython.test_gc.GCCallbackTests.test_collect_garbage"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_gc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_gc
_suite = unittest.defaultTestLoader.loadTestsFromName("GCCallbackTests.test_collect_garbage", test_gc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython GCCallbackTests.test_collect_garbage did not pass"
print("GCCallbackTests::test_collect_garbage: ok")
