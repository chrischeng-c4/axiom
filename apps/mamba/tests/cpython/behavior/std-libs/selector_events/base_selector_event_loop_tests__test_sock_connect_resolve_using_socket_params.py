# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selector_events"
# dimension = "behavior"
# case = "base_selector_event_loop_tests__test_sock_connect_resolve_using_socket_params"
# subject = "cpython.test_selector_events.BaseSelectorEventLoopTests.test_sock_connect_resolve_using_socket_params"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_selector_events.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_selector_events
_suite = unittest.defaultTestLoader.loadTestsFromName("BaseSelectorEventLoopTests.test_sock_connect_resolve_using_socket_params", test_selector_events)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BaseSelectorEventLoopTests.test_sock_connect_resolve_using_socket_params did not pass"
print("BaseSelectorEventLoopTests::test_sock_connect_resolve_using_socket_params: ok")
