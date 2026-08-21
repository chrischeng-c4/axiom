# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "misc"
# dimension = "behavior"
# case = "test_pending_calls__test_pendingcalls_non_threaded"
# subject = "cpython.test_misc.TestPendingCalls.test_pendingcalls_non_threaded"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_misc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_misc
_suite = unittest.defaultTestLoader.loadTestsFromName("TestPendingCalls.test_pendingcalls_non_threaded", test_misc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestPendingCalls.test_pendingcalls_non_threaded did not pass"
print("TestPendingCalls::test_pendingcalls_non_threaded: ok")
