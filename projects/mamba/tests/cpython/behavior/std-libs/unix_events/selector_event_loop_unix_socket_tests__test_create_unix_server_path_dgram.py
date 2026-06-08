# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unix_events"
# dimension = "behavior"
# case = "selector_event_loop_unix_socket_tests__test_create_unix_server_path_dgram"
# subject = "cpython.test_unix_events.SelectorEventLoopUnixSocketTests.test_create_unix_server_path_dgram"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_unix_events.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_unix_events
_suite = unittest.defaultTestLoader.loadTestsFromName("SelectorEventLoopUnixSocketTests.test_create_unix_server_path_dgram", test_unix_events)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SelectorEventLoopUnixSocketTests.test_create_unix_server_path_dgram did not pass"
print("SelectorEventLoopUnixSocketTests::test_create_unix_server_path_dgram: ok")
