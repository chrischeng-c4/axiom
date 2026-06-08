# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "proactor_events"
# dimension = "behavior"
# case = "proactor_event_loop_unix_sock_sendfile_tests__test_sock_sendfile_iobuffer"
# subject = "cpython.test_proactor_events.ProactorEventLoopUnixSockSendfileTests.test_sock_sendfile_iobuffer"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_proactor_events.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_proactor_events
_suite = unittest.defaultTestLoader.loadTestsFromName("ProactorEventLoopUnixSockSendfileTests.test_sock_sendfile_iobuffer", test_proactor_events)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ProactorEventLoopUnixSockSendfileTests.test_sock_sendfile_iobuffer did not pass"
print("ProactorEventLoopUnixSockSendfileTests::test_sock_sendfile_iobuffer: ok")
