# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "waitfor"
# dimension = "behavior"
# case = "asyncio_wait_for_test__test_cancel_blocking_wait_for_uc5e3298"
# subject = "cpython.test_waitfor.AsyncioWaitForTest.test_cancel_blocking_wait_for"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_waitfor.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_waitfor
_suite = unittest.defaultTestLoader.loadTestsFromName("AsyncioWaitForTest.test_cancel_blocking_wait_for", test_waitfor)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AsyncioWaitForTest.test_cancel_blocking_wait_for did not pass"
print("AsyncioWaitForTest::test_cancel_blocking_wait_for: ok")
