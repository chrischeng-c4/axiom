# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "coroutines"
# dimension = "behavior"
# case = "coroutine_test__test_for_assign_raising_stop_async_iteration_uc123ba5"
# subject = "cpython.test_coroutines.CoroutineTest.test_for_assign_raising_stop_async_iteration"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_coroutines.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_coroutines
_suite = unittest.defaultTestLoader.loadTestsFromName("CoroutineTest.test_for_assign_raising_stop_async_iteration", test_coroutines)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CoroutineTest.test_for_assign_raising_stop_async_iteration did not pass"
print("CoroutineTest::test_for_assign_raising_stop_async_iteration: ok")
