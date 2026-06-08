# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "waitfor"
# dimension = "behavior"
# case = "asyncio_wait_for_test__test_wait_for_race_condition_uc8b5ddf"
# subject = "cpython.test_waitfor.AsyncioWaitForTest.test_wait_for_race_condition"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_waitfor.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_waitfor
_suite = unittest.defaultTestLoader.loadTestsFromName("AsyncioWaitForTest.test_wait_for_race_condition", test_waitfor)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AsyncioWaitForTest.test_wait_for_race_condition did not pass"
print("AsyncioWaitForTest::test_wait_for_race_condition: ok")
