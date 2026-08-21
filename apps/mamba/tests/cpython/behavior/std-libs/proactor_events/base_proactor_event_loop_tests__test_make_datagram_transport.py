# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "proactor_events"
# dimension = "behavior"
# case = "base_proactor_event_loop_tests__test_make_datagram_transport"
# subject = "cpython.test_proactor_events.BaseProactorEventLoopTests.test_make_datagram_transport"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_proactor_events.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_proactor_events
_suite = unittest.defaultTestLoader.loadTestsFromName("BaseProactorEventLoopTests.test_make_datagram_transport", test_proactor_events)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BaseProactorEventLoopTests.test_make_datagram_transport did not pass"
print("BaseProactorEventLoopTests::test_make_datagram_transport: ok")
