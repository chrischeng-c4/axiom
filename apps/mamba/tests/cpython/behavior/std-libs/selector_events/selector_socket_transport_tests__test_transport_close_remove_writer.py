# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selector_events"
# dimension = "behavior"
# case = "selector_socket_transport_tests__test_transport_close_remove_writer"
# subject = "cpython.test_selector_events.SelectorSocketTransportTests.test_transport_close_remove_writer"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_selector_events.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_selector_events
_suite = unittest.defaultTestLoader.loadTestsFromName("SelectorSocketTransportTests.test_transport_close_remove_writer", test_selector_events)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SelectorSocketTransportTests.test_transport_close_remove_writer did not pass"
print("SelectorSocketTransportTests::test_transport_close_remove_writer: ok")
