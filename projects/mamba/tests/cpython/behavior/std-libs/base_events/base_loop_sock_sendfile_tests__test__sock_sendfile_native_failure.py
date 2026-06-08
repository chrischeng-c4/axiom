# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base_events"
# dimension = "behavior"
# case = "base_loop_sock_sendfile_tests__test__sock_sendfile_native_failure"
# subject = "cpython.test_base_events.BaseLoopSockSendfileTests.test__sock_sendfile_native_failure"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_base_events.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_base_events
_suite = unittest.defaultTestLoader.loadTestsFromName("BaseLoopSockSendfileTests.test__sock_sendfile_native_failure", test_base_events)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BaseLoopSockSendfileTests.test__sock_sendfile_native_failure did not pass"
print("BaseLoopSockSendfileTests::test__sock_sendfile_native_failure: ok")
