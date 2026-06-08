# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "proactor_events"
# dimension = "behavior"
# case = "proactor_event_loop_unix_sock_sendfile_tests__test_sock_sendfile_not_a_file"
# subject = "cpython.test_proactor_events.ProactorEventLoopUnixSockSendfileTests.test_sock_sendfile_not_a_file"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_proactor_events.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_proactor_events
_suite = unittest.defaultTestLoader.loadTestsFromName("ProactorEventLoopUnixSockSendfileTests.test_sock_sendfile_not_a_file", test_proactor_events)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ProactorEventLoopUnixSockSendfileTests.test_sock_sendfile_not_a_file did not pass"
print("ProactorEventLoopUnixSockSendfileTests::test_sock_sendfile_not_a_file: ok")
