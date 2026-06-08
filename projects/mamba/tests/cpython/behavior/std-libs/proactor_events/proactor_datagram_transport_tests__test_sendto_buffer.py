# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "proactor_events"
# dimension = "behavior"
# case = "proactor_datagram_transport_tests__test_sendto_buffer"
# subject = "cpython.test_proactor_events.ProactorDatagramTransportTests.test_sendto_buffer"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_proactor_events.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_proactor_events
_suite = unittest.defaultTestLoader.loadTestsFromName("ProactorDatagramTransportTests.test_sendto_buffer", test_proactor_events)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ProactorDatagramTransportTests.test_sendto_buffer did not pass"
print("ProactorDatagramTransportTests::test_sendto_buffer: ok")
