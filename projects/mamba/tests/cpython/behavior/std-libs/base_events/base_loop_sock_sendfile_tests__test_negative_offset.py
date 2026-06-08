# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base_events"
# dimension = "behavior"
# case = "base_loop_sock_sendfile_tests__test_negative_offset"
# subject = "cpython.test_base_events.BaseLoopSockSendfileTests.test_negative_offset"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_base_events.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_base_events
_suite = unittest.defaultTestLoader.loadTestsFromName("BaseLoopSockSendfileTests.test_negative_offset", test_base_events)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BaseLoopSockSendfileTests.test_negative_offset did not pass"
print("BaseLoopSockSendfileTests::test_negative_offset: ok")
