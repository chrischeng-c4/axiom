# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unix_events"
# dimension = "behavior"
# case = "unix_read_pipe_transport_tests__test_ctor"
# subject = "cpython.test_unix_events.UnixReadPipeTransportTests.test_ctor"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_unix_events.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_unix_events
_suite = unittest.defaultTestLoader.loadTestsFromName("UnixReadPipeTransportTests.test_ctor", test_unix_events)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython UnixReadPipeTransportTests.test_ctor did not pass"
print("UnixReadPipeTransportTests::test_ctor: ok")
