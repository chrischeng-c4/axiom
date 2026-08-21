# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unix_events"
# dimension = "behavior"
# case = "unix_write_pipe_transport_tests__test_write_buffer"
# subject = "cpython.test_unix_events.UnixWritePipeTransportTests.test_write_buffer"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_unix_events.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_unix_events
_suite = unittest.defaultTestLoader.loadTestsFromName("UnixWritePipeTransportTests.test_write_buffer", test_unix_events)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython UnixWritePipeTransportTests.test_write_buffer did not pass"
print("UnixWritePipeTransportTests::test_write_buffer: ok")
