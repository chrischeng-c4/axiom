# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "proactor_events"
# dimension = "behavior"
# case = "proactor_socket_transport_tests__test_loop_reading_aborted_is_fatal"
# subject = "cpython.test_proactor_events.ProactorSocketTransportTests.test_loop_reading_aborted_is_fatal"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_proactor_events.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_proactor_events
_suite = unittest.defaultTestLoader.loadTestsFromName("ProactorSocketTransportTests.test_loop_reading_aborted_is_fatal", test_proactor_events)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ProactorSocketTransportTests.test_loop_reading_aborted_is_fatal did not pass"
print("ProactorSocketTransportTests::test_loop_reading_aborted_is_fatal: ok")
