# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base_events"
# dimension = "behavior"
# case = "base_event_tests__test_ipaddr_info_no_inet_pton"
# subject = "cpython.test_base_events.BaseEventTests.test_ipaddr_info_no_inet_pton"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_base_events.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_base_events
_suite = unittest.defaultTestLoader.loadTestsFromName("BaseEventTests.test_ipaddr_info_no_inet_pton", test_base_events)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BaseEventTests.test_ipaddr_info_no_inet_pton did not pass"
print("BaseEventTests::test_ipaddr_info_no_inet_pton: ok")
