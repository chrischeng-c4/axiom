# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unix_events"
# dimension = "behavior"
# case = "selector_event_loop_unix_sock_sendfile_tests__test_sock_sendfile_not_available"
# subject = "cpython.test_unix_events.SelectorEventLoopUnixSockSendfileTests.test_sock_sendfile_not_available"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_unix_events.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_unix_events
_suite = unittest.defaultTestLoader.loadTestsFromName("SelectorEventLoopUnixSockSendfileTests.test_sock_sendfile_not_available", test_unix_events)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SelectorEventLoopUnixSockSendfileTests.test_sock_sendfile_not_available did not pass"
print("SelectorEventLoopUnixSockSendfileTests::test_sock_sendfile_not_available: ok")
