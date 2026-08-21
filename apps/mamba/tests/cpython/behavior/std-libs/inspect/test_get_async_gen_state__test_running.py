# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "test_get_async_gen_state__test_running"
# subject = "cpython.test_inspect.TestGetAsyncGenState.test_running"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_inspect/test_inspect.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_inspect import test_inspect
_suite = unittest.defaultTestLoader.loadTestsFromName("TestGetAsyncGenState.test_running", test_inspect)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestGetAsyncGenState.test_running did not pass"
print("TestGetAsyncGenState::test_running: ok")
