# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selector_events"
# dimension = "behavior"
# case = "selector_socket_transport_buffered_protocol_tests__test_read_ready_eof_keep_open"
# subject = "cpython.test_selector_events.SelectorSocketTransportBufferedProtocolTests.test_read_ready_eof_keep_open"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_selector_events.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_selector_events
_suite = unittest.defaultTestLoader.loadTestsFromName("SelectorSocketTransportBufferedProtocolTests.test_read_ready_eof_keep_open", test_selector_events)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SelectorSocketTransportBufferedProtocolTests.test_read_ready_eof_keep_open did not pass"
print("SelectorSocketTransportBufferedProtocolTests::test_read_ready_eof_keep_open: ok")
