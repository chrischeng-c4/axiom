# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selector_events"
# dimension = "behavior"
# case = "selector_datagram_transport_tests__test_sendto_buffer"
# subject = "cpython.test_selector_events.SelectorDatagramTransportTests.test_sendto_buffer"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_selector_events.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_selector_events
_suite = unittest.defaultTestLoader.loadTestsFromName("SelectorDatagramTransportTests.test_sendto_buffer", test_selector_events)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SelectorDatagramTransportTests.test_sendto_buffer did not pass"
print("SelectorDatagramTransportTests::test_sendto_buffer: ok")
