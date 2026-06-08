# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selector_events"
# dimension = "behavior"
# case = "selector_socket_transport_tests__test_writelines_pauses_protocol"
# subject = "cpython.test_selector_events.SelectorSocketTransportTests.test_writelines_pauses_protocol"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_selector_events.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_selector_events
_suite = unittest.defaultTestLoader.loadTestsFromName("SelectorSocketTransportTests.test_writelines_pauses_protocol", test_selector_events)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SelectorSocketTransportTests.test_writelines_pauses_protocol did not pass"
print("SelectorSocketTransportTests::test_writelines_pauses_protocol: ok")
