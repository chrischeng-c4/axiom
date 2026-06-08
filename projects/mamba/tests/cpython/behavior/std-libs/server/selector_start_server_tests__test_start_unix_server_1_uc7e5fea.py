# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "server"
# dimension = "behavior"
# case = "selector_start_server_tests__test_start_unix_server_1_uc7e5fea"
# subject = "cpython.test_server.SelectorStartServerTests.test_start_unix_server_1"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_server.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_server
_suite = unittest.defaultTestLoader.loadTestsFromName("SelectorStartServerTests.test_start_unix_server_1", test_server)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SelectorStartServerTests.test_start_unix_server_1 did not pass"
print("SelectorStartServerTests::test_start_unix_server_1: ok")
